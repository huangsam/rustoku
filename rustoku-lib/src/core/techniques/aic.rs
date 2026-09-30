use super::{TechniqueFlags, TechniquePropagator, TechniqueRule};
use crate::core::SolvePath;
use std::collections::VecDeque;

/// Alternating Inference Chain (AIC) Technique.
///
/// An AIC is a continuous chain of alternating strong and weak inferences between candidate propositions.
/// Because the chain begins and ends with strong links:
/// - If the starting candidate is false, the ending candidate must be true.
/// - Therefore, at least one of the two endpoints must be true.
/// - Any candidate that sees both endpoints can be safely eliminated.
///
/// ### Link Types
/// - **Strong link** ($\neg A \implies B$): If $A$ is false, $B$ must be true. Arises in bivalue cells
///   (only 2 candidates in a cell) or bilocal units (a digit appears in only 2 cells of a row, col, or box).
/// - **Weak link** ($A \implies \neg B$): If $A$ is true, $B$ must be false. Arises between any two candidates
///   in the same cell, or identical candidates in cells that see each other.
///
/// ### Elimination Types
/// 1. **Mutual Peer Elimination** (X-Chain / XY-Chain): When endpoints share the same candidate value in
///    different cells, any cell seeing both endpoints cannot hold that candidate.
/// 2. **Discontinuous Nice Loop** (DNL): When endpoints occupy the same cell with different candidate values,
///    one of the two must be true, eliminating all other candidates from that cell.
///
/// ### Example
/// If `(0, 0)=5 == (0, 4)=5 -- (3, 4)=5 == (3, 7)=5`, then either `(0, 0)=5` or `(3, 7)=5` must be true.
/// Candidate 5 can be eliminated from any cell seeing both `(0, 0)` and `(3, 7)` (such as `(0, 7)` or `(3, 0)`).
///
/// See: <https://hodoku.sourceforge.net/en/tech_chains.php>
pub struct AlternatingInferenceChain;

/// A candidate proposition in the inference chain, representing digit `val` at cell `(r, c)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    r: usize,
    c: usize,
    val: u8,
}

/// Inference strength connecting two candidate nodes in the chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkType {
    /// Strong inference ($\neg A \implies B$). If preceding node is false, following node is true.
    Strong,
    /// Weak inference ($A \implies \neg B$). If preceding node is true, following node is false.
    Weak,
}

/// Tracks an active alternating inference path during breadth-first search.
#[derive(Debug, Clone)]
struct ChainPath {
    nodes: Vec<Node>,
    last_link: LinkType,
}

impl AlternatingInferenceChain {
    /// Checks if two nodes share a weak link (cells see each other and share the same candidate value).
    fn is_weak_link(n1: &Node, n2: &Node) -> bool {
        (n1.r == n2.r || n1.c == n2.c || (n1.r / 3 == n2.r / 3 && n1.c / 3 == n2.c / 3))
            && n1.val == n2.val
    }

    /// Checks if two candidate nodes share a strong link.
    ///
    /// A strong link exists in two cases:
    /// 1. **Bivalue cell**: Both nodes share the same cell with different candidate values,
    ///    and the cell has exactly 2 candidates.
    /// 2. **Bilocal unit**: Both nodes share the same candidate value in different cells,
    ///    and the candidate appears in exactly 2 cells of their shared row, column, or 3x3 box.
    fn is_strong_link(prop: &TechniquePropagator, n1: &Node, n2: &Node) -> bool {
        // Condition 1: Bivalue cell (same cell, different values)
        if n1.r == n2.r && n1.c == n2.c && n1.val != n2.val {
            let mask = prop.candidates.get(n1.r, n1.c);
            return mask.count_ones() == 2;
        }

        // Condition 2: Bilocal unit (different cells, same value, only 2 places in unit)
        if n1.val == n2.val && (n1.r != n2.r || n1.c != n2.c) {
            let val_mask = 1 << (n1.val - 1);

            // Check Row
            if n1.r == n2.r {
                let mut count = 0;
                for c in 0..9 {
                    if (prop.candidates.get(n1.r, c) & val_mask) != 0 {
                        count += 1;
                    }
                }
                if count == 2 {
                    return true;
                }
            }

            // Check Col
            if n1.c == n2.c {
                let mut count = 0;
                for r in 0..9 {
                    if (prop.candidates.get(r, n1.c) & val_mask) != 0 {
                        count += 1;
                    }
                }
                if count == 2 {
                    return true;
                }
            }

            // Check Box
            let b1 = (n1.r / 3) * 3 + n1.c / 3;
            let b2 = (n2.r / 3) * 3 + n2.c / 3;
            if b1 == b2 {
                let br = (n1.r / 3) * 3;
                let bc = (n1.c / 3) * 3;
                let mut count = 0;
                for r in br..br + 3 {
                    for c in bc..bc + 3 {
                        if (prop.candidates.get(r, c) & val_mask) != 0 {
                            count += 1;
                        }
                    }
                }
                if count == 2 {
                    return true;
                }
            }
        }

        false
    }

    /// Finds all candidate nodes reachable from `current` via the requested link type (strong or weak).
    fn find_next_nodes(prop: &TechniquePropagator, current: &Node, need_strong: bool) -> Vec<Node> {
        let mut next_nodes = Vec::new();

        // Check same cell (different candidate values)
        let mask = prop.candidates.get(current.r, current.c);
        for v in 1..=9 {
            if v != current.val && (mask & (1 << (v - 1))) != 0 {
                let next = Node {
                    r: current.r,
                    c: current.c,
                    val: v,
                };
                if need_strong {
                    if Self::is_strong_link(prop, current, &next) {
                        next_nodes.push(next);
                    }
                } else {
                    next_nodes.push(next); // Any two candidates in the same cell are weakly linked
                }
            }
        }

        // Check peers (same candidate value across row, column, or box)
        let val_mask = 1 << (current.val - 1);

        // Row peers
        for c in 0..9 {
            if c != current.c && (prop.candidates.get(current.r, c) & val_mask) != 0 {
                let next = Node {
                    r: current.r,
                    c,
                    val: current.val,
                };
                if need_strong {
                    if Self::is_strong_link(prop, current, &next) {
                        next_nodes.push(next);
                    }
                } else {
                    next_nodes.push(next);
                }
            }
        }

        // Column peers
        for r in 0..9 {
            if r != current.r && (prop.candidates.get(r, current.c) & val_mask) != 0 {
                let next = Node {
                    r,
                    c: current.c,
                    val: current.val,
                };
                if need_strong {
                    if Self::is_strong_link(prop, current, &next) {
                        next_nodes.push(next);
                    }
                } else if !next_nodes.contains(&next) {
                    next_nodes.push(next);
                }
            }
        }

        // Box peers
        let br = (current.r / 3) * 3;
        let bc = (current.c / 3) * 3;
        for r in br..br + 3 {
            for c in bc..bc + 3 {
                if (r != current.r || c != current.c) && (prop.candidates.get(r, c) & val_mask) != 0
                {
                    let next = Node {
                        r,
                        c,
                        val: current.val,
                    };
                    if need_strong {
                        if Self::is_strong_link(prop, current, &next) {
                            next_nodes.push(next);
                        }
                    } else if !next_nodes.contains(&next) {
                        next_nodes.push(next);
                    }
                }
            }
        }

        next_nodes
    }

    /// Tests whether the endpoints of a valid even-length chain permit candidate eliminations.
    fn find_eliminations(
        prop: &mut TechniquePropagator,
        path: &mut SolvePath,
        chain: &ChainPath,
    ) -> bool {
        // Valid AICs must have an even number of nodes (start and end are strong-linked to their neighbors)
        if chain.nodes.len() < 4 || !chain.nodes.len().is_multiple_of(2) {
            return false;
        }

        let start = &chain.nodes[0];
        let end = chain
            .nodes
            .last()
            .expect("chain should have at least 4 nodes");

        let mut progress = false;

        // Case 1: Same candidate value in different cells (X-Chain / XY-Chain).
        // Candidate val is eliminated from all mutual peers seeing both start and end.
        if start.val == end.val {
            let val = start.val;
            let val_mask = 1 << (val - 1);

            for r in 0..9 {
                for c in 0..9 {
                    if (r == start.r && c == start.c) || (r == end.r && c == end.c) {
                        continue;
                    }

                    if (prop.candidates.get(r, c) & val_mask) != 0 {
                        let target = Node { r, c, val };
                        // Target must see both start and end cells
                        if Self::is_weak_link(start, &target)
                            && Self::is_weak_link(end, &target)
                            && prop.eliminate_candidate(
                                r,
                                c,
                                val_mask,
                                TechniqueFlags::ALTERNATING_INFERENCE_CHAIN,
                                path,
                            )
                        {
                            progress = true;
                        }
                    }
                }
            }
        } else if start.r == end.r && start.c == end.c {
            // Case 2: Discontinuous Nice Loop (DNL).
            // Start and end are in the same cell with different values: one of them MUST be true,
            // so all other candidate values in this cell can be eliminated.
            let mask = prop.candidates.get(start.r, start.c);
            let keep_mask = (1 << (start.val - 1)) | (1 << (end.val - 1));
            let remove_mask = mask & !keep_mask;

            if remove_mask != 0
                && prop.eliminate_multiple_candidates(
                    start.r,
                    start.c,
                    remove_mask,
                    TechniqueFlags::ALTERNATING_INFERENCE_CHAIN,
                    path,
                )
            {
                progress = true;
            }
        }

        progress
    }
}

impl TechniqueRule for AlternatingInferenceChain {
    /// Applies the AIC technique by performing breadth-first search from every candidate.
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        // Step 1: Collect all candidate nodes as potential chain starting points
        let mut starts = Vec::new();
        for r in 0..9 {
            for c in 0..9 {
                let mask = prop.candidates.get(r, c);
                for v in 1..=9 {
                    if (mask & (1 << (v - 1))) != 0 {
                        starts.push(Node { r, c, val: v });
                    }
                }
            }
        }

        let max_depth = 14; // Bounded search depth to ensure fast execution

        // Step 2: Perform BFS search starting from each candidate node
        for start in starts {
            let mut queue = VecDeque::new();
            queue.push_back(ChainPath {
                nodes: vec![start],
                last_link: LinkType::Weak, // First outbound link must be Strong
            });

            while let Some(current_path) = queue.pop_front() {
                if current_path.nodes.len() >= max_depth {
                    continue;
                }

                let current_node = current_path
                    .nodes
                    .last()
                    .expect("path nodes should never be empty");
                let need_strong = current_path.last_link == LinkType::Weak;

                let next_nodes =
                    AlternatingInferenceChain::find_next_nodes(prop, current_node, need_strong);

                // Step 3: Explore valid link extensions
                for next in next_nodes {
                    // Prevent cycles (do not revisit nodes already present in this chain)
                    if current_path.nodes.contains(&next) {
                        continue;
                    }

                    let mut new_path = current_path.clone();
                    new_path.nodes.push(next);
                    new_path.last_link = if need_strong {
                        LinkType::Strong
                    } else {
                        LinkType::Weak
                    };

                    // Step 4: When a strong link completes an even-length chain of length >= 4,
                    // test for candidate eliminations.
                    if new_path.last_link == LinkType::Strong
                        && new_path.nodes.len() >= 4
                        && AlternatingInferenceChain::find_eliminations(prop, path, &new_path)
                    {
                        return true; // Elimination made, return to let propagator restart
                    }

                    queue.push_back(new_path);
                }
            }
        }

        false
    }

    fn flags(&self) -> TechniqueFlags {
        TechniqueFlags::ALTERNATING_INFERENCE_CHAIN
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{Rustoku, SolvePath, SolveStep, TechniqueFlags};

    #[test]
    fn test_aic_eliminates_candidates_on_x_chain_puzzle() {
        // X-Chain
        let s = "3.4.2..8...6.......5..7.3.....68..2.....34....6.15.7...1.........9....6...8217..5";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::ALTERNATING_INFERENCE_CHAIN);
        let mut path = SolvePath::default();
        rustoku.techniques_make_valid_changes(&mut path);

        let eliminations: Vec<_> = path
            .steps
            .iter()
            .filter_map(|step| match step {
                SolveStep::CandidateElimination {
                    row,
                    col,
                    value,
                    flags,
                    ..
                } if flags.contains(TechniqueFlags::ALTERNATING_INFERENCE_CHAIN) => {
                    Some((*row, *col, *value))
                }
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "AIC should produce at least one candidate elimination on this X-Chain puzzle"
        );
    }

    #[test]
    fn test_aic_eliminates_candidates_on_xy_chain_puzzle() {
        // XY-Chain
        let s = "3...4.52858.........2..........74....1....35..5.6...4..78.....21..2......39..68..";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::ALTERNATING_INFERENCE_CHAIN);
        let mut path = SolvePath::default();
        rustoku.techniques_make_valid_changes(&mut path);

        let eliminations: Vec<_> = path
            .steps
            .iter()
            .filter_map(|step| match step {
                SolveStep::CandidateElimination {
                    row,
                    col,
                    value,
                    flags,
                    ..
                } if flags.contains(TechniqueFlags::ALTERNATING_INFERENCE_CHAIN) => {
                    Some((*row, *col, *value))
                }
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "AIC should produce at least one candidate elimination on this XY-Chain puzzle"
        );
    }

    #[test]
    fn test_aic_eliminates_candidates_on_dnl_puzzle() {
        // Discontinuous Nice Loop
        let s = "....8.2....5....4..2...5........7......21..971.4....3...........973..52...8.5136.";
        let mut rustoku = Rustoku::new_from_str(s)
            .unwrap()
            .with_techniques(TechniqueFlags::ALTERNATING_INFERENCE_CHAIN);
        let mut path = SolvePath::default();
        rustoku.techniques_make_valid_changes(&mut path);

        let eliminations: Vec<_> = path
            .steps
            .iter()
            .filter_map(|step| match step {
                SolveStep::CandidateElimination {
                    row,
                    col,
                    value,
                    flags,
                    ..
                } if flags.contains(TechniqueFlags::ALTERNATING_INFERENCE_CHAIN) => {
                    Some((*row, *col, *value))
                }
                _ => None,
            })
            .collect();

        assert!(
            !eliminations.is_empty(),
            "AIC should produce at least one candidate elimination on this Discontinuous Nice Loop puzzle"
        );
    }
}
