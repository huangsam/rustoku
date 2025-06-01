use super::{TechniquePropagator, TechniqueRule};

/// Naked singles technique implementation.
pub struct NakedSingles;

impl TechniqueRule for NakedSingles {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut Vec<(usize, usize, u8)>) -> bool {
        let mut placements_made = false;

        for r in 0..9 {
            for c in 0..9 {
                if prop.board.is_empty(r, c) {
                    let cand_mask = prop.candidates_cache.get(r, c);
                    if cand_mask.count_ones() == 1 {
                        let num = cand_mask.trailing_zeros() as u8 + 1;
                        prop.place_and_update(r, c, num, path);
                        placements_made = true;
                    }
                }
            }
        }
        placements_made
    }
}
