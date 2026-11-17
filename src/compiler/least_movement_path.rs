//! The cheapest way through a sequence of choices: a shortest path through a
//! layered graph where each layer is one thing to play (in written order),
//! each of its nodes one way to play it, and each edge the movement between
//! two consecutive ways. A Viterbi-style dynamic program finds the global
//! optimum in O(n · k²). Shared by the guitar chord view (one voicing per
//! chord) and the harmonica-tab view (one hole per note).

use itertools::Itertools;

/// What a path costs: `node` for picking the node at an index of its layer,
/// `movement` for going from one node to the next.
pub(crate) struct PathCosts<Node, Movement> {
    pub(crate) node: Node,
    pub(crate) movement: Movement,
}

/// For each layer, the index of its node on the cheapest path. Every layer
/// must be non-empty. Ties go to the node earliest in its layer, so the
/// result is stable.
pub(crate) fn least_movement_path<T>(
    layers: &[Vec<T>],
    costs: &PathCosts<impl Fn(usize) -> u32, impl Fn(&T, &T) -> u32>,
) -> Vec<usize> {
    let Some(first) = layers.first() else {
        return vec![];
    };
    // `path_costs[j]`: the cheapest path ending on the current layer's node
    // `j`; `back_pointers[i][j]`: that path's node on layer `i`.
    let (path_costs, back_pointers) = layers.iter().tuple_windows().fold(
        (
            (0..first.len()).map(&costs.node).collect_vec(),
            Vec::<Vec<usize>>::new(),
        ),
        |(path_costs, mut back_pointers), (previous, layer)| {
            let (next_costs, pointers): (Vec<u32>, Vec<usize>) = layer
                .iter()
                .enumerate()
                .map(|(index, node)| {
                    let (from, cost) = previous
                        .iter()
                        .zip(&path_costs)
                        .map(|(from, cost)| cost + (costs.movement)(from, node))
                        .enumerate()
                        .min_by_key(|&(_, cost)| cost)
                        .unwrap_or_default();
                    (cost + (costs.node)(index), from)
                })
                .unzip();
            back_pointers.push(pointers);
            (next_costs, back_pointers)
        },
    );
    let last = path_costs.iter().position_min().unwrap_or_default();
    let mut path = back_pointers
        .iter()
        .rev()
        .scan(last, |current, pointers| {
            *current = pointers.get(*current).copied().unwrap_or_default();
            Some(*current)
        })
        .collect_vec();
    path.reverse();
    path.push(last);
    path
}
