use super::DataHolder;

#[derive(Debug)]
pub enum Operation {
    None(DataHolder),
    Add(DataHolder, DataHolder),
    Sub(DataHolder, DataHolder),
    And(DataHolder, DataHolder),
    Or(DataHolder, DataHolder),
    Xor(DataHolder, DataHolder),
    LShiftRight(DataHolder),
    Not(DataHolder),
}
