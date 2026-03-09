use super::{TechniqueFlags, TechniquePropagator, TechniqueRule};
use crate::core::SolvePath;
use std::collections::VecDeque;

/// Alternating Inference Chain (AIC) Technique.
///
/// An AIC is a continuous chain of alternating strong and weak links between candidates.
/// If the first candidate is false, the chain implies the last candidate is true. Therefore,
/// any candidate that sees both the first and last candidate in the chain must be false.
///
/// - Strong link: Two candidates are the ONLY two possibilities in a unit (If A is false, B is true).
/// - Weak link: Two candidates cannot BOTH be true (If A is true, B is false).
pub struct AlternatingInferenceChain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    r: usize,
    c: usize,
    val: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkType {
    Strong,
    Weak,
}

#[derive(Debug, Clone)]
struct ChainPath {
    nodes: Vec<Node>,
    last_link: LinkType,
}

impl AlternatingInferenceChain {
    /// Check if two nodes have a weak link (they see each other and cannot both be true).
    fn is_weak_link(n1: &Node, n2: &Node) -> bool {
        (n1.r == n2.r || n1.c == n2.c || (n1.r / 3 == n2.r / 3 && n1.c / 3 == n2.c / 3))
            && n1.val == n2.val
    }

    /// Check if two nodes have a strong link.
    /// A strong link exists if they are the ONLY two cells in a unit (row, col, box) that
    /// can hold the given value, OR if they are the ONLY two candidates in a single cell (bivalue).
    fn is_strong_link(prop: &TechniquePropagator, n1: &Node, n2: &Node) -> bool {
        // Condition 1: Bivalue cell (n1 and n2 are in the same cell, different values)
        if n1.r == n2.r && n1.c == n2.c && n1.val != n2.val {
            let mask = prop.candidates.get(n1.r, n1.c);
            return mask.count_ones() == 2;
        }

        // Condition 2: Bilocal unit (n1 and n2 are in different cells, same value, only 2 places in unit)
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

    /// Finds all candidates that can be the next step in the chain
    fn find_next_nodes(prop: &TechniquePropagator, current: &Node, need_strong: bool) -> Vec<Node> {
        let mut next_nodes = Vec::new();

        // Check same cell (different values)
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
                    next_nodes.push(next); // Any two candidates in a cell are weakly linked
                }
            }
        }

        // Check peers (same value)
        let val_mask = 1 << (current.val - 1);

        // Row
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
                    next_nodes.push(next); // Any two identical candidates in a row are weakly linked
                }
            }
        }

        // Col
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

        // Box
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

    /// Try to find a target node that sees both ends of a valid chain
    fn find_eliminations(
        prop: &mut TechniquePropagator,
        path: &mut SolvePath,
        chain: &ChainPath,
    ) -> bool {
        if chain.nodes.len() < 4 || !chain.nodes.len().is_multiple_of(2) {
            return false; // Valid AICs have an even number of nodes (start/end are strong linked to their neighbors)
        }
        // The chain starts with a strong link, alternates, and ends with a strong link.
        // Therefore, if the first node is false, the last node MUST be true.
        // Any node that sees BOTH the first and last node MUST be false.

        let start = &chain.nodes[0];
        let end = chain.nodes.last().unwrap();

        // They must be different nodes but have the SAME value to eliminate that value from peers.
        // (If they are different values, it's a completely different type of deduction, like a grouped chain).
        // Let's stick to single-digit eliminations (X-Chains) or cross-digit eliminations (XY-Chains).
        // If start and end have the SAME value, we eliminate that value from mutual peers.
        // If start and end are in the SAME cell, we can safely say the true value MUST be one of them, effectively eliminating all OTHER candidates in that cell.
        // If start and end have DIFFERENT values in DIFFERENT cells, we can only eliminate a candidate if it sees BOTH ends AND equals the respective end values (rare).

        let mut progress = false;

        if start.val == end.val {
            let val = start.val;
            let val_mask = 1 << (val - 1);

            // Find mutual peers
            for r in 0..9 {
                for c in 0..9 {
                    if (r == start.r && c == start.c) || (r == end.r && c == end.c) {
                        continue;
                    }

                    if (prop.candidates.get(r, c) & val_mask) != 0 {
                        let target = Node { r, c, val };
                        // Target must be weakly linked (see) both start and end
                        // Note: Technically for different values, target just needs to "see" start and end.
                        // But since start and end are the same value, "seeing" is equivalent to is_weak_link.
                        if Self::is_weak_link(start, &target) && Self::is_weak_link(end, &target)
                            && prop.eliminate_candidate(
                                r,
                                c,
                                val_mask,
                                TechniqueFlags::ALTERNATING_INFERENCE_CHAIN,
                                path,
                            ) {
                                progress = true;
                            }
                    }
                }
            }
        } else if start.r == end.r && start.c == end.c {
            // Discontinuous Nice Loop type: start and end are the same cell, different values.
            // This means one of these two MUST be true, so all other candidates in this cell are false.
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
                ) {
                    progress = true;
                }
        }

        progress
    }
}

impl TechniqueRule for AlternatingInferenceChain {
    fn apply(&self, prop: &mut TechniquePropagator, path: &mut SolvePath) -> bool {
        // Collect all possible starting nodes (candidates)
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

        let max_depth = 14; // Prevent infinite loops / too long chains

        for start in starts {
            let mut queue = VecDeque::new();
            queue.push_back(ChainPath {
                nodes: vec![start],
                last_link: LinkType::Weak, // First link outbound will be Strong
            });

            while let Some(current_path) = queue.pop_front() {
                if current_path.nodes.len() >= max_depth {
                    continue;
                }

                let current_node = current_path.nodes.last().unwrap();
                let need_strong = current_path.last_link == LinkType::Weak;

                let next_nodes =
                    AlternatingInferenceChain::find_next_nodes(prop, current_node, need_strong);

                for next in next_nodes {
                    // Prevent loops (don't visit nodes already in the chain)
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

                    // If we just added a strong link, we can check for eliminations
                    // (AICs must start and end with strong links, hence even number of nodes)
                    if new_path.last_link == LinkType::Strong && new_path.nodes.len() >= 4
                        && AlternatingInferenceChain::find_eliminations(prop, path, &new_path) {
                            return true; // We made an elimination, return to let propagator restart
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
