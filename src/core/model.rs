pub struct MergeFile {
    pub blocks: Vec<Block>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Block {
    Regular(Vec<String>),
    Conflict(Conflict),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConflictSegment {
    pub tag: Option<String>,
    pub lines: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Conflict {
    pub ours: ConflictSegment,
    pub base: Option<ConflictSegment>,
    pub theirs: ConflictSegment,
    pub resolution: Option<Resolution>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Resolution {
    Ours,
    Theirs,
    TheirsBeforeOurs,
    OursBeforeTheirs,
    Edited(Vec<String>),
}
