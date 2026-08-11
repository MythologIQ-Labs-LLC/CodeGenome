use crate::graph::edge::{Edge, Relation};
use crate::identity::UorAddress;

/// Find all ancestors of a given address via MutatedFrom edges.
/// Follows child→parent direction. Returns oldest first.
pub fn ancestors(addr: UorAddress, edges: &[Edge]) -> Vec<UorAddress> {
    let mut chain = Vec::new();
    let mut current = addr;
    loop {
        let parent = edges
            .iter()
            .find(|e| e.source == current && e.relation == Relation::MutatedFrom);
        match parent {
            Some(e) => {
                chain.push(e.target);
                current = e.target;
            }
            None => break,
        }
    }
    chain.reverse();
    chain
}

/// Find all descendants of a given address.
/// Follows parent←child direction. Returns newest last.
pub fn descendants(addr: UorAddress, edges: &[Edge]) -> Vec<UorAddress> {
    let mut chain = Vec::new();
    let mut current = addr;
    loop {
        let child = edges
            .iter()
            .find(|e| e.target == current && e.relation == Relation::MutatedFrom);
        match child {
            Some(e) => {
                chain.push(e.source);
                current = e.source;
            }
            None => break,
        }
    }
    chain
}

/// Full lineage chain: ancestors + self + descendants.
pub fn lineage_chain(addr: UorAddress, edges: &[Edge]) -> Vec<UorAddress> {
    let mut chain = ancestors(addr, edges);
    chain.push(addr);
    chain.extend(descendants(addr, edges));
    chain
}
