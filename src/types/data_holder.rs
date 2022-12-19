use super::Error;

#[derive(Debug)]
pub enum DataHolder {
    A,
    B,
    Const(i8),
    AAddr,
    ConstAddr(i8),
}

impl DataHolder {
    pub fn encode_arg1(&self) -> Result<u8, Error> {
        match *self {
            Self::A => Ok(0),
            Self::B => Ok(1),
            _ => Err(Error("Argument arg1 non applicable".to_owned())),
        }
    }

    pub fn encode_dest(&self) -> Result<u8, Error> {
        match *self {
            Self::A => Ok(0),
            Self::B => Ok(1),
            _ => Err(Error("Argument dest non applicable".to_owned())),
        }
    }

    pub fn encode_arg2(&self) -> Result<u8, Error> {
        match *self {
            Self::A => Ok(0),
            Self::Const(_) => Ok(1),
            _ => Err(Error("Argument arg2 non applicable".to_owned())),
        }
    }
}
