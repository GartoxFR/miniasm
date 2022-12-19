#[derive(Debug)]
pub enum JRCond {
    True,
    IfZ,
    IfC,
    IfN,
}

#[derive(Debug)]
pub enum JumpTarget<'a> {
    Const(u8),
    Label(&'a str),
}
