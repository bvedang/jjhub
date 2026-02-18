use rift_core::{ChangeId, Revision, RevisionId, describe};

fn main() {
    let revision_id = RevisionId(String::from("123"));
    let change_id = ChangeId(String::from("234"));
    let revision = Revision{
        revision_id,
        change_id,
        tree_hash: String::from("abc"),
        delta_hash: String::from("bcd"),
        author: String::from("Vednag"),
        description: String::from("Init commit"),
        timestamp: String::from("345")
    };
    describe(revision);
}
