use crate::belief::create::{create_belief, BeliefSpec};
use crate::belief::store::{load_beliefs, persist_beliefs, try_load_beliefs};
use codegenome_identity::graph::node::NodeKind;
use codegenome_identity::graph::overlay::OverlayKind;
use codegenome_identity::identity::address_of;
use codegenome_identity::store::backend::StoreBackend;
use codegenome_identity::store::ondisk::OnDiskStore;

fn addr(name: &str) -> codegenome_identity::identity::UorAddress {
    address_of(name.as_bytes())
}

fn temp_store(suffix: &str) -> (OnDiskStore, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "cg_belief_persist_{}_{:?}",
        suffix,
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    (OnDiskStore::new(&dir), dir)
}

#[test]
fn persisted_belief_appears_in_loaded() {
    let (store, dir) = temp_store("load");
    let belief = create_belief(&BeliefSpec {
        claim: "fn_x is dead".into(),
        subject: addr("fn_x"),
        confidence: 0.8,
        supporting_evidence: vec![],
        contradicting_evidence: vec![],
        actor: "agent".into(),
    });
    persist_beliefs(&store, &[belief]).unwrap();
    let (nodes, _edges) = load_beliefs(&store).unwrap();
    assert!(nodes.iter().any(|n| n.kind == NodeKind::Belief));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn beliefs_survive_fused_overlay_rewrite() {
    let (store, dir) = temp_store("rewrite");
    // Persist a belief
    let belief = create_belief(&BeliefSpec {
        claim: "survives reindex".into(),
        subject: addr("fn_y"),
        confidence: 0.7,
        supporting_evidence: vec![],
        contradicting_evidence: vec![],
        actor: "agent".into(),
    });
    persist_beliefs(&store, &[belief]).unwrap();

    // Rewrite the fused overlay (simulating reindex)
    store
        .write_overlay(&OverlayKind::Custom("fused".into()), &[], &[])
        .unwrap();

    // Beliefs still loadable
    let (nodes, _) = load_beliefs(&store).unwrap();
    assert!(nodes.iter().any(|n| n.kind == NodeKind::Belief));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn try_load_returns_empty_when_none_persisted() {
    let (store, dir) = temp_store("empty");
    let (nodes, edges) = try_load_beliefs(&store);
    assert!(nodes.is_empty());
    assert!(edges.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
