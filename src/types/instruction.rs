use std::collections::BTreeMap;

use crate::types::{DataHolder, Error, JRCond, JumpTarget, Operation};

#[derive(Debug)]
pub enum Instruction<'a> {
    Assignment { op: Operation, dest: DataHolder },
    JA(JumpTarget<'a>),
    JR(JRCond, JumpTarget<'a>),
    Check(DataHolder, DataHolder),
    RetI,
}

pub enum BinaryInstruction {
    SingleByte([u8; 1]),
    DoubleByte([u8; 2]),
}
impl<'a> Instruction<'a> {
    pub fn to_binary(
        &self,
        current_addr: u8,
        label_table: &BTreeMap<&str, u8>,
    ) -> Result<BinaryInstruction, Error> {
        use Instruction::*;
        match *self {
            JA(ref target) => {
                let instr = match *target {
                    JumpTarget::Const(instr) => instr,
                    JumpTarget::Label(label) => *label_table
                        .get(label)
                        .ok_or_else(|| Error(format!("Undefined label : {}", label)))?,
                };
                Ok(BinaryInstruction::DoubleByte([0b01111000, instr]))
            }
            JR(ref cond, ref target) => {
                let cond: u8 = match cond {
                    JRCond::True => 0b00,
                    JRCond::IfZ => 0b01,
                    JRCond::IfC => 0b10,
                    JRCond::IfN => 0b11,
                };
                let instr = match *target {
                    JumpTarget::Const(instr) => {
                        let instr = instr as i8;
                        if !(-16..=15).contains(&instr) {
                            return Err(Error(format!("Saut relatif trop grand : {}", instr)));
                        }
                        instr
                    }
                    JumpTarget::Label(label) => {
                        let instr = *label_table
                            .get(label)
                            .ok_or_else(|| Error(format!("Undefined label : {}", label)))?;
                        let instr = (instr as i16 - current_addr as i16) as i8;
                        if !(-16..=15).contains(&instr) {
                            return Err(Error(format!(
                                "Saut relatif vers un label trop éloigné : {}",
                                label
                            )));
                        }
                        instr
                    }
                };
                if !(-16..=15).contains(&instr) {
                    return Err(Error("Les sauts relatifs doivent être des entiers signés sur 5 bits (entre -16 et 15 inclus)".to_owned()));
                }
                let mut result: u8 = 1 << 7;
                result |= cond << 5;
                result |= (0b11111 & instr) as u8;

                Ok(BinaryInstruction::SingleByte([result]))
            }
            Check(ref arg1, ref arg2) => {
                let arg1_encode = arg1.encode_arg1()?;
                let arg2_encode = arg2.encode_arg2()?;

                let mut instr = 0b00110 << 3;
                instr |= arg2_encode << 2;
                instr |= arg1_encode << 1;

                match *arg2 {
                    DataHolder::A => Ok(BinaryInstruction::SingleByte([instr])),
                    DataHolder::Const(cst) => Ok(BinaryInstruction::DoubleByte([instr, cst as u8])),
                    _ => unreachable!(),
                }
            }
            Assignment { ref op, ref dest } => match op {
                Operation::Add(arg1, arg2)
                | Operation::Sub(arg1, arg2)
                | Operation::And(arg1, arg2)
                | Operation::Or(arg1, arg2)
                | Operation::Xor(arg1, arg2) => {
                    let arg1_encode = arg1.encode_arg1()?;
                    let arg2_encode = arg2.encode_arg2()?;
                    let dest_encode = dest.encode_dest()?;

                    let mut instr = match op {
                        Operation::Add(..) => 0b0000,
                        Operation::Sub(..) => 0b0001,
                        Operation::And(..) => 0b0010,
                        Operation::Or(..) => 0b0011,
                        Operation::Xor(..) => 0b0100,
                        _ => unreachable!(),
                    } << 3;

                    instr |= arg2_encode << 2;
                    instr |= arg1_encode << 1;
                    instr |= dest_encode;

                    match *arg2 {
                        DataHolder::A => Ok(BinaryInstruction::SingleByte([instr])),
                        DataHolder::Const(cst) => {
                            Ok(BinaryInstruction::DoubleByte([instr, cst as u8]))
                        }
                        _ => unreachable!(),
                    }
                }
                Operation::LShiftRight(arg1) | Operation::Not(arg1) => {
                    let arg1_encode = arg1.encode_arg1()?;
                    let dest_encode = dest.encode_dest()?;
                    let mut instr: u8 = match op {
                        Operation::LShiftRight(..) => 0b0101,
                        Operation::Not(..) => 0b1000,
                        _ => unreachable!(),
                    } << 3;

                    if let Operation::Not(..) = op {
                        instr |= 1 << 2;
                    }

                    instr |= arg1_encode << 1;
                    instr |= dest_encode;

                    Ok(BinaryInstruction::SingleByte([instr]))
                }
                Operation::None(arg) => {
                    if let Ok(dest_encode) = dest.encode_dest() {
                        match *arg {
                            DataHolder::A | DataHolder::Const(_) => {
                                let mut instr = 0b1001 << 3;
                                instr |= arg.encode_arg2()? << 2;
                                instr |= dest_encode;

                                match *arg {
                                    DataHolder::A => Ok(BinaryInstruction::SingleByte([instr])),
                                    DataHolder::Const(cst) => {
                                        Ok(BinaryInstruction::DoubleByte([instr, cst as u8]))
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            DataHolder::AAddr | DataHolder::ConstAddr(_) => {
                                let mut instr = 0b1101 << 3;
                                instr |= match *arg {
                                    DataHolder::AAddr => 0,
                                    DataHolder::ConstAddr(_) => 1,
                                    _ => unreachable!(),
                                } << 2;
                                instr |= dest_encode;

                                match *arg {
                                    DataHolder::AAddr => Ok(BinaryInstruction::SingleByte([instr])),
                                    DataHolder::ConstAddr(cst) => {
                                        Ok(BinaryInstruction::DoubleByte([instr, cst as u8]))
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            DataHolder::B => {
                                let mut instr = 0b1000 << 3;
                                instr |= arg.encode_arg1()? << 1;
                                instr |= dest_encode;

                                Ok(BinaryInstruction::SingleByte([instr]))
                            }
                        }
                    } else {
                        match *dest {
                            DataHolder::AAddr | DataHolder::ConstAddr(_) => {
                                let arg2_encode: u8 = match *dest {
                                    DataHolder::AAddr => 0,
                                    DataHolder::ConstAddr(_) => 1,
                                    _ => unreachable!(),
                                };
                                let mut instr = 0b1110 << 3;
                                instr |= arg2_encode << 2;
                                instr |= arg.encode_arg1()? << 1;

                                match *dest {
                                    DataHolder::AAddr => Ok(BinaryInstruction::SingleByte([instr])),
                                    DataHolder::ConstAddr(cst) => {
                                        Ok(BinaryInstruction::DoubleByte([instr, cst as u8]))
                                    }
                                    _ => unreachable!(),
                                }
                            }
                            _ => Err(Error("Argument dest ou *arg2 non applicable".to_owned())),
                        }
                    }
                }
            },
            RetI => Ok(BinaryInstruction::SingleByte([0b01011000])),
        }
    }

    pub fn get_byte_size(&self) -> u8 {
        use DataHolder::*;
        use Instruction::*;
        use Operation::*;
        match self {
            JA(_) => 2,
            Check(_, Const(_)) => 2,
            Assignment { op, dest } => {
                if let ConstAddr(_) = dest {
                    2
                } else {
                    match op {
                        Add(_, Const(_))
                        | Sub(_, Const(_))
                        | And(_, Const(_))
                        | Or(_, Const(_))
                        | Xor(_, Const(_))
                        | None(Const(_))
                        | None(ConstAddr(_)) => 2,
                        _ => 1,
                    }
                }
            }
            _ => 1,
        }
    }
}
