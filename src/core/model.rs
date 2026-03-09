pub struct MergeFile {
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Block {
    Regular(Vec<String>),
    Conflict(Conflict),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Conflict {
    pub ours: Vec<String>,
    pub base: Option<Vec<String>>,
    pub theirs: Vec<String>,
    pub resolution: Option<Resolution>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Resolution {
    Ours,
    Theirs,
    Base,
    Edited(Vec<String>),
}
