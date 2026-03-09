pub struct MergeFile {
    pub blocks: Vec<Block>,
}

pub enum Block {
    Regular(Vec<String>),
    Conflict(Conflict),
}

pub struct Conflict {
    pub ours: Vec<String>,
    pub base: Option<Vec<String>>,
    pub theirs: Vec<String>,
    pub resolution: Option<Resolution>,
}

pub enum Resolution {
    Ours,
    Theirs,
    Base,
    Edited(Vec<String>),
}
