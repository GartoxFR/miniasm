use std::collections::BTreeMap;

use crate::types::{BinaryInstruction, Error, Line};

pub fn generate_label_table<'a>(program: &'a [Line<'a>]) -> Result<BTreeMap<&'a str, u8>, Error> {
    let mut table = BTreeMap::default();
    let mut addr: u8 = 0;

    for (label, instr) in program {
        if let Some(label) = label.as_deref() {
            if label.eq("isr") {
                addr = 0xA0;
            }

            if table.insert(label, addr).is_some() {
                return Err(Error(format!("Label {} is defined multiple times", label)));
            }
        }

        addr = addr
            .checked_add(instr.get_byte_size())
            .expect("Overflowing address (Program too large)");
    }

    Ok(table)
}

pub fn compile(program: &[Line]) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];
    let mut addr: u8 = 0;

    let label_table = generate_label_table(program).unwrap();

    for (label, instr) in program {
        match *label {
            Some(label) if label.eq("isr") => bytes.resize(0xA0, 0),
            _ => (),
        }

        let bin_instr = instr.to_binary(addr, &label_table).unwrap();

        match bin_instr {
            BinaryInstruction::SingleByte(arr) => {
                bytes.extend_from_slice(&arr);
                addr += 1;
            }
            BinaryInstruction::DoubleByte(arr) => {
                bytes.extend_from_slice(&arr);
                addr += 2;
            }
        }
    }

    bytes
}
