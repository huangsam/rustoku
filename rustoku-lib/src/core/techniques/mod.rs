use crate::core::{SolvePath, SolveStep};

use super::board::Board;
use super::candidates::Candidates;
use super::masks::Masks;

pub mod flags;
mod hidden_pairs;
mod hidden_quads;
mod hidden_singles;
mod hidden_triples;
mod locked_candidates;
mod naked_pairs;
mod naked_quads;
mod naked_singles;
mod naked_triples;
mod swordfish;
pub mod units;
mod w_wing;
mod x_wing;
mod xy_wing;
mod xyz_wing;

use flags::TechniqueFlags;
use hidden_pairs::HiddenPairs;
use hidden_quads::HiddenQuads;
use hidden_singles::HiddenSingles;
use hidden_triples::HiddenTriples;
use locked_candidates::LockedCandidates;
use naked_pairs::NakedPairs;
use naked_quads::NakedQuads;
use naked_singles::NakedSingles;
use naked_triples::NakedTriples;
use swordfish::Swordfish;
use w_wing::WWing;
use x_wing::XWing;
use xy_wing::XYWing;
use xyz_wing::XyzWing;

/// Propagates constraints via zero or more techniques.
///
/// The techniques are toggled via bitflags. Most of the data in struct comes
/// from the Rustoku instance, which has a longer lifetime than this struct - since
/// it is only used at the start, before any backtracking occurs.
///
/// Some examples of techniques employed including Naked Singles and X-Wings.
/// If we want to add more techniques, extend the existing logic and bitflags
/// in this module.
///
/// This class acts as the Mediator object between `Rustoku` and the `TechniqueRule`
/// implementations out there. To learn about the Mediator design pattern, please
/// consult [this link](https://refactoring.guru/design-patterns/mediator)
/// for more details.
pub struct TechniquePropagator<'a> {
    board: &'a mut Board,
    masks: &'a mut Masks,
    candidates: &'a mut Candidates,
    techniques_enabled: TechniqueFlags,
}

impl<'a> TechniquePropagator<'a> {
    pub fn new(
        board: &'a mut Board,
        masks: &'a mut Masks,
        candidates: &'a mut Candidates,
        techniques_enabled: TechniqueFlags,
    ) -> Self {
        Self {
            board,
            masks,
            candidates,
            techniques_enabled,
        }
    }

    /// Helper to place a number and update caches.
    fn place_and_update(
        &mut self,
        r: usize,
        c: usize,
        num: u8,
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) {
        self.board.set(r, c, num);
        self.masks.add_number(r, c, num);

        // Count candidates eliminated by this placement
        let affected_cells_count = self.count_affected_cells(r, c, num);
        let candidates_eliminated_count = self.count_candidates_eliminated(r, c, num);

        self.candidates
            .update_affected_cells(r, c, self.masks, self.board);

        let step_number = path.steps.len() as u32;
        let difficulty_point = Self::difficulty_for_technique(flags);

        path.steps.push(SolveStep::Placement {
            row: r,
            col: c,
            value: num,
            flags,
            step_number,
            candidates_eliminated: candidates_eliminated_count,
            related_cell_count: affected_cells_count.min(255) as u8,
            difficulty_point,
        });
    }

    /// Helper to remove a number and update caches.
    fn remove_and_update(&mut self, r: usize, c: usize, num: u8) {
        self.board.set(r, c, 0);
        self.masks.remove_number(r, c, num);
        self.candidates
            .update_affected_cells(r, c, self.masks, self.board);
        // Note: For propagation, `remove_number` is mostly for backtracking, not direct technique application.
        // The `update_affected_cells` on removal will recalculate candidates for the now-empty cell.
    }

    /// Helper to eliminate a candidate and update caches.
    fn eliminate_candidate(
        &mut self,
        r: usize,
        c: usize,
        candidate_bit: u16, // Assume only one candidate is being eliminated
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let initial_mask = self.candidates.get(r, c);
        let refined_mask = initial_mask & !candidate_bit;
        self.candidates.set(r, c, refined_mask);

        let num = candidate_bit.trailing_zeros() as u8 + 1; // Convert bit to number
        let step_number = path.steps.len() as u32;
        let difficulty_point = Self::difficulty_for_technique(flags);

        path.steps.push(SolveStep::CandidateElimination {
            row: r,
            col: c,
            value: num,
            flags,
            step_number,
            candidates_eliminated: 1, // Single candidate was eliminated
            related_cell_count: 1,    // At minimum, this cell is affected
            difficulty_point,
        });

        initial_mask != refined_mask // Return true if a candidate was eliminated
    }

    /// Helper to eliminate multiple candidates and update caches
    fn eliminate_multiple_candidates(
        &mut self,
        r: usize,
        c: usize,
        elimination_mask: u16, // bits to eliminate
        flags: TechniqueFlags,
        path: &mut SolvePath,
    ) -> bool {
        let initial_mask = self.candidates.get(r, c);
        let refined_mask = initial_mask & !elimination_mask;
        self.candidates.set(r, c, refined_mask);

        // Log each eliminated candidate
        let eliminated_mask = initial_mask & elimination_mask; // what was actually eliminated
        let eliminated_count = eliminated_mask.count_ones();
        let difficulty_point = Self::difficulty_for_technique(flags);

        for candidate in 1..=9 {
            let candidate_bit = 1 << (candidate - 1);
            if (eliminated_mask & candidate_bit) != 0 {
                let step_number = path.steps.len() as u32;
                path.steps.push(SolveStep::CandidateElimination {
                    row: r,
                    col: c,
                    value: candidate,
                    flags,
                    step_number,
                    candidates_eliminated: eliminated_count,
                    related_cell_count: 1,
                    difficulty_point,
                });
            }
        }

        initial_mask != refined_mask // Return true if a candidate was eliminated
    }

    /// Counts affected cells when placing a number (cells in same row, column, or box).
    /// Deduplicates cells that appear in multiple units (row+box or col+box overlap).
    fn count_affected_cells(&self, r: usize, c: usize, _num: u8) -> u32 {
        let mut count = 0u32;
        let box_r = (r / 3) * 3;
        let box_c = (c / 3) * 3;

        // Count cells in the same row
        for col in 0..9 {
            if col != c && self.board.is_empty(r, col) {
                count += 1;
            }
        }

        // Count cells in the same column
        for row in 0..9 {
            if row != r && self.board.is_empty(row, c) {
                count += 1;
            }
        }

        // Count cells in the same 3x3 box, excluding those already counted in the row or column
        for br in box_r..box_r + 3 {
            for bc in box_c..box_c + 3 {
                if br != r && bc != c && self.board.is_empty(br, bc) {
                    count += 1;
                }
            }
        }

        count
    }

    /// Counts the number of candidates that would be eliminated by a placement.
    /// Deduplicates cells that appear in multiple units (row+box or col+box overlap).
    fn count_candidates_eliminated(&self, r: usize, c: usize, num: u8) -> u32 {
        let mut count = 0u32;
        let box_r = (r / 3) * 3;
        let box_c = (c / 3) * 3;
        let candidate_bit = 1u16 << (num - 1);

        // Count in the same row
        for col in 0..9 {
            if col != c && (self.candidates.get(r, col) & candidate_bit) != 0 {
                count += 1;
            }
        }

        // Count in the same column
        for row in 0..9 {
            if row != r && (self.candidates.get(row, c) & candidate_bit) != 0 {
                count += 1;
            }
        }

        // Count in the same 3x3 box, excluding those already counted in the row or column
        for br in box_r..box_r + 3 {
            for bc in box_c..box_c + 3 {
                if br != r && bc != c && (self.candidates.get(br, bc) & candidate_bit) != 0 {
                    count += 1;
                }
            }
        }

        count
    }

    /// Returns a difficulty metric for a given technique.
    fn difficulty_for_technique(flags: TechniqueFlags) -> u8 {
        if flags.is_empty() {
            0
        } else {
            flags.bits().trailing_zeros() as u8 + 1
        }
    }

    /// Applies deterministic constraint propagation techniques iteratively.
    pub fn propagate_constraints(&mut self, path: &mut SolvePath, initial_path_len: usize) -> bool {
        let techniques: Vec<&dyn TechniqueRule> = vec![
            &NakedSingles,
            &HiddenSingles,
            &NakedPairs,
            &HiddenPairs,
            &LockedCandidates,
            &NakedTriples,
            &HiddenTriples,
            &XWing,
            &NakedQuads,
            &HiddenQuads,
            &Swordfish,
            &WWing,
            &XYWing,
            &XyzWing,
        ];

        loop {
            let mut changed_this_iter = false;

            for technique in &techniques {
                if self.techniques_enabled.contains(technique.flags()) {
                    // Pass the propagator itself and the current path to the technique. This
                    // is an example of the Mediator pattern, where the propagator
                    // mediates the interaction between the techniques and the board
                    changed_this_iter |= technique.apply(self, path);
                    if changed_this_iter {
                        break;
                    }
                }
            }

            if (0..9).any(|r| {
                (0..9).any(|c| self.board.is_empty(r, c) && self.candidates.get(r, c) == 0)
            }) {
                while path.steps.len() > initial_path_len {
                    if let Some(step) = path.steps.pop() {
                        match step {
                            SolveStep::Placement {
                                row,
                                col,
                                value,
                                flags: _,
                                ..
                            } => {
                                self.remove_and_update(row, col, value);
                            }
                            SolveStep::CandidateElimination {
                                row,
                                col,
                                value,
                                flags: _,
                                ..
                            } => {
                                // This is a candidate elimination step, we need to restore the candidate
                                // in the candidates cache
                                let initial_mask = self.candidates.get(row, col);
                                let refined_mask = initial_mask | (1 << (value - 1));
                                self.candidates.set(row, col, refined_mask);
                            }
                        }
                    }
                }
                return false;
            }

            if !changed_this_iter {
                break;
            }
        }
        true
    }
}

/// This is the contract for all human techniques.
///
/// All techniques are expected to have a way to apply themselves to a board
/// and modify the solve path with placements and eliminations. In addition, they
/// are expected to return one flag that helps with technique attribution when
/// people want to visualize the solve path.
///
/// To get started on the intuition behind the techniques, check out
/// [SudokuWiki](https://www.sudokuwiki.org/Introduction) and
/// [HoDoKu](https://hodoku.sourceforge.net/en/tech_intro.php)
/// to understand the basic strategies and techniques used in Sudoku solving.
pub trait TechniqueRule {
    /// Applies the technique to the given propagator.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool;

    /// Returns the flags associated with this technique.
    fn flags(&self) -> TechniqueFlags;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Rustoku;

    struct TechniqueTestCase<'a> {
        name: &'a str,
        trigger_string: &'a str,
        technique_flag: TechniqueFlags,
    }

    #[test]
    fn test_each_technique_makes_valid_changes() {
        let test_cases = vec![
            // Last digit is empty, no other option exists
            TechniqueTestCase {
                name: "Naked Singles",
                trigger_string: "385421967194756328627983145571892634839645271246137589462579813918364752753218490",
                technique_flag: TechniqueFlags::NAKED_SINGLES,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=h101&tech=Hidden+Single
            TechniqueTestCase {
                name: "Hidden Singles",
                trigger_string: "008007000016083000000000051107290000000000000000046307290000000000860140000300700",
                technique_flag: TechniqueFlags::HIDDEN_SINGLES,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=n201&tech=Naked+Pair
            TechniqueTestCase {
                name: "Naked Pairs",
                trigger_string: "700009030000105006400260009002083951007000000005600000000000003100000060000004010",
                technique_flag: TechniqueFlags::NAKED_PAIRS,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=h201&tech=Hidden+Pair
            TechniqueTestCase {
                name: "Hidden Pairs",
                trigger_string: "000032000000000000007600914096000800005008000030040005050200000700000560904010000",
                // Needs easy techniques to reduce candidates first, then hidden pairs to find the pair
                technique_flag: TechniqueFlags::EASY | TechniqueFlags::HIDDEN_PAIRS,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=lc101&tech=Locked+Candidates+Type+1+%28Pointing%29
            TechniqueTestCase {
                name: "Locked Candidates",
                trigger_string: "984000000000500040000000002006097200003002000000000010005060003407051890030009700",
                technique_flag: TechniqueFlags::LOCKED_CANDIDATES,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=bf201&tech=X-Wing
            TechniqueTestCase {
                name: "X-Wing",
                trigger_string: "000000000760003002002640009403900070000004903005000020010560000370090041000000060",
                technique_flag: TechniqueFlags::X_WING,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=bf301&tech=Swordfish
            TechniqueTestCase {
                name: "Swordfish",
                trigger_string: "160540070008001030030800000700050069600902057000000000000030040000000016000164500",
                technique_flag: TechniqueFlags::EASY
                    | TechniqueFlags::MEDIUM
                    | TechniqueFlags::SWORDFISH,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=xy01&tech=XY-Wing
            TechniqueTestCase {
                name: "XY-Wing",
                trigger_string: "000060000000010863003009000904000000300000704570820000000006580690007000000040030",
                technique_flag: TechniqueFlags::EASY
                    | TechniqueFlags::MEDIUM
                    | TechniqueFlags::XY_WING,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=l302&tech=Locked+Triple
            TechniqueTestCase {
                name: "Naked Triples",
                trigger_string: "400500370320000004060000000800002030210840000000000090070090100040651000000070000",
                technique_flag: TechniqueFlags::NAKED_TRIPLES,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=h301&tech=Hidden+Triple
            TechniqueTestCase {
                name: "Hidden Triples",
                trigger_string: "200000400500000006001034080000500040000000000060790000090200600003009001000080037",
                technique_flag: TechniqueFlags::EASY | TechniqueFlags::HIDDEN_TRIPLES,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=w01&tech=W-Wing
            TechniqueTestCase {
                name: "W-Wing",
                trigger_string: "025100000000009030400708900040000800150400000000060004000000008263040000080390106",
                technique_flag: TechniqueFlags::EASY | TechniqueFlags::W_WING,
            },
            // https://hodoku.sourceforge.net/en/show_example.php?file=xyz01&tech=XYZ-Wing
            TechniqueTestCase {
                name: "XYZ-Wing",
                trigger_string: "069000000000021000000800400001530080007600050000000100000000003902080010000340205",
                technique_flag: TechniqueFlags::EASY | TechniqueFlags::XYZ_WING,
            },
        ];

        for test_case in test_cases {
            let rustoku = Rustoku::new_from_str(test_case.trigger_string)
                .unwrap_or_else(|_| panic!("Rustoku creation failed for '{}'", test_case.name));
            let mut path = SolvePath::default();
            assert!(
                rustoku
                    .with_techniques(test_case.technique_flag)
                    .techniques_make_valid_changes(&mut path),
                "Propagation should not contradict for '{}'",
                test_case.name
            );
            assert!(
                !path.steps.is_empty(),
                "Expected at least one placement or elimination for '{}'",
                test_case.name
            )
        }
    }

    #[test]
    fn test_all_techniques_produce_valid_solution() {
        // Every technique-trigger puzzle must still solve to a valid board
        let puzzles = vec![
            "385421967194756328627983145571892634839645271246137589462579813918364752753218490",
            "008007000016083000000000051107290000000000000000046307290000000000860140000300700",
            "700009030000105006400260009002083951007000000005600000000000003100000060000004010",
            "000032000000000000007600914096000800005008000030040005050200000700000560904010000",
            "984000000000500040000000002006097200003002000000000010005060003407051890030009700",
            "000000000760003002002640009403900070000004903005000020010560000370090041000000060",
            "000970081200083005600000000400000027008705000006400000905010200000000040060000103",
            "004000000030701000700000090070060100608400000000050024080009005100300080943000700",
            "003020600900305001001806400008102900700000008006708200002609500800203009005010300",
            "200000400500000006001034080000500040000000000060790000090200600003009001000080037",
            "069000000000021000000800400001530080007600050000000100000000003902080010000340205", // XYZ-Wing
        ];

        for puzzle in puzzles {
            let mut rustoku = Rustoku::new_from_str(puzzle)
                .unwrap()
                .with_techniques(TechniqueFlags::all());
            let solution = rustoku.solve_any();
            assert!(
                solution.is_some(),
                "Puzzle should be solvable with all techniques: {puzzle}"
            );
            let solved = solution.unwrap().board;
            let check = Rustoku::new(solved).unwrap();
            assert!(
                check.is_solved(),
                "Solution must be valid for puzzle: {puzzle}"
            );
        }
    }

    #[test]
    fn test_techniques_do_not_alter_given_clues() {
        // Verify that constraint propagation never overwrites already-given clues
        let puzzles = vec![
            (
                "Naked Singles",
                "385421967194756328627983145571892634839645271246137589462579813918364752753218490",
                TechniqueFlags::NAKED_SINGLES,
            ),
            (
                "Hidden Singles",
                "008007000016083000000000051107290000000000000000046307290000000000860140000300700",
                TechniqueFlags::HIDDEN_SINGLES,
            ),
            (
                "Naked Pairs",
                "700009030000105006400260009002083951007000000005600000000000003100000060000004010",
                TechniqueFlags::NAKED_PAIRS,
            ),
            (
                "Swordfish",
                "000970081200083005600000000400000027008705000006400000905010200000000040060000103",
                TechniqueFlags::EASY | TechniqueFlags::MEDIUM | TechniqueFlags::SWORDFISH,
            ),
            (
                "XY-Wing",
                "004000000030701000700000090070060100608400000000050024080009005100300080943000700",
                TechniqueFlags::EASY | TechniqueFlags::MEDIUM | TechniqueFlags::XY_WING,
            ),
            (
                "Naked Triples",
                "003020600900305001001806400008102900700000008006708200002609500800203009005010300",
                TechniqueFlags::NAKED_TRIPLES,
            ),
            (
                "Hidden Triples",
                "200000400500000006001034080000500040000000000060790000090200600003009001000080037",
                TechniqueFlags::EASY | TechniqueFlags::HIDDEN_TRIPLES,
            ),
            (
                "XYZ-Wing",
                "069000000000021000000800400001530080007600050000000100000000003902080010000340205",
                TechniqueFlags::EASY | TechniqueFlags::XYZ_WING,
            ),
        ];

        for (name, puzzle, flag) in puzzles {
            let original = Board::try_from(puzzle).unwrap();
            let mut rustoku = Rustoku::new_from_str(puzzle).unwrap().with_techniques(flag);
            let mut path = SolvePath::default();
            rustoku.techniques_make_valid_changes(&mut path);

            for r in 0..9 {
                for c in 0..9 {
                    let orig_val = original.get(r, c);
                    if orig_val != 0 {
                        assert_eq!(
                            rustoku.board.get(r, c),
                            orig_val,
                            "{name}: clue at ({r},{c}) was overwritten"
                        );
                    }
                }
            }
        }
    }
}
