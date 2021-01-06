// Some variables in the messages structures are
// special structures, defined here.
use std::convert::TryFrom;
use std::convert::TryInto;

use super::ByteSize;

#[derive(Debug, PartialEq)]
pub enum VarUint {
    Small(u8),
    Median(u16),
    Large(u32),
    Big(u64),
}

impl VarUint {
    pub fn new(num: u64) -> Self {
        if num <= 252 {
            VarUint::Small(num.try_into().unwrap())
        } else if num <= 0xFFFF {
            VarUint::Median(num.try_into().unwrap())
        } else if num <= 0xFFFFFFFF {
            VarUint::Large(num.try_into().unwrap())
        } else {
            VarUint::Big(num)
        }
    }

    pub fn value(&self) -> u64 {
        match self {
            Self::Small(num) => *num as u64,
            Self::Median(num) => *num as u64,
            Self::Large(num) => *num as u64,
            Self::Big(num) => *num,
        }
    }
}

impl ByteSize for VarUint {
    fn byte_size(&self) -> usize {
        match self {
            Self::Small(_) => 1,
            Self::Median(_) => 3,
            Self::Large(_) => 5,
            Self::Big(_) => 9,
        }
    }
}

impl TryFrom<&[u8]> for VarUint {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() == 0 {
            return Err("Empty slice");
        }

        let (first_byte, rest) = bytes.split_at(1);
        let first_byte = first_byte[0];
        match first_byte {
            token if token == 0xFD => {
                if rest.len() < 2 {
                    Err("Slice not big enough")
                } else {
                    let (num, _) = rest.split_at(2);
                    let num = u16::from_be_bytes(num.try_into().unwrap());
                    Ok(VarUint::Median(num))
                }
            },
            token if token == 0xFE => {
                if rest.len() < 4 {
                    Err("Slice not big enough")
                } else {
                    let (num, _) = rest.split_at(4);
                    let num = u32::from_be_bytes(num.try_into().unwrap());
                    Ok(VarUint::Large(num))
                }
            },
            token if token == 0xFF => {
                if rest.len() < 8 {
                    Err("Slice not big enough")
                } else {
                    let (num, _) = rest.split_at(8);
                    let num = u64::from_be_bytes(num.try_into().unwrap());
                    Ok(VarUint::Big(num))
                }
            },
            num => Ok(VarUint::Small(num))
        }
    }
}

impl From<VarUint> for Vec<u8> {
    fn from(var: VarUint) -> Self {
        match var {
            VarUint::Small(num) => u8::to_be_bytes(num).into(),
            VarUint::Median(num) => {
                let mut bytes: Vec<u8> = vec![0xFD];
                bytes.extend(&u16::to_be_bytes(num));
                bytes
            },
            VarUint::Large(num) => {
                let mut bytes = vec![0xFE];
                bytes.extend(&u32::to_be_bytes(num));
                bytes
            },
            VarUint::Big(num) => {
                let mut bytes = vec![0xFF];
                bytes.extend(&u64::to_be_bytes(num));
                bytes
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_varuint() {
        let pos: u64 = 8;
        assert_eq!(VarUint::new(pos), VarUint::Small(8));

        let pos: u64 = 255;
        assert_eq!(VarUint::new(pos), VarUint::Median(255));

        let pos: u64 = 1024;
        assert_eq!(VarUint::new(pos), VarUint::Median(1024));
    }

    #[test]
    fn test_convert_varuint() {
        let var = VarUint::new(25433);
        let bytes = Vec::<u8>::from(var);
        assert_eq!(VarUint::try_from(bytes.as_slice()), Ok(VarUint::new(25433)));
    }
}
