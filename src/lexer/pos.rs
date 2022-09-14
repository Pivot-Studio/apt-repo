#[derive(Debug, PartialEq)]
pub struct Pos {
    pub line: usize,   // 1based
    pub column: usize, // 1based
    pub offset: usize, // 0based
}

#[derive(Debug, PartialEq)]
pub struct Range {
    pub start: Pos,
    pub end: Pos,
}