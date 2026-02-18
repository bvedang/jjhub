pub struct RevisionId(pub String);

pub struct ChangeId(pub String);

pub struct Revision{
    pub revision_id:RevisionId,
    pub change_id: ChangeId,
    pub tree_hash: String,
    pub delta_hash: String,
    pub author: String,
    pub description:String,
    pub timestamp: String
}

pub fn describe(revision: Revision){
    println!("Revision ID : {}",revision.revision_id.0);
    println!("Change ID : {}",revision.change_id.0);
    println!("Tree Hash : {}",revision.tree_hash);
    println!("Delta Hash : {}",revision.delta_hash);
    println!("Author : {}",revision.author);
    println!("Description : {}",revision.description);
    println!("TimeStamp : {}",revision.timestamp);

}
