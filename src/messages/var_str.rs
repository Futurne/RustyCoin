use std::convert::TryFrom;

use super::var_uint::VarUint;
use super::ByteSize;

#[derive(Debug, PartialEq)]
struct VarStr {
    length: VarUint,
    string_value: String,
}

impl VarStr {
    pub fn new(string_value: String) -> Self {
        VarStr {
            length: VarUint::new(string_value.len() as u64),
            string_value,
        }
    }
}

impl ByteSize for VarStr {
    fn byte_size(&self) -> usize {
        self.length.byte_size() + self.length.value() as usize
    }
}

impl TryFrom<&[u8]> for VarStr {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let length = VarUint::try_from(bytes)?;
        if bytes.len() < length.value() as usize {
            return Err("Slice is not big enough");
        }
        // Remove the bytes used by the VarUint
        let (_, bytes) = bytes.split_at(length.byte_size());

        let (string_value, _) = bytes.split_at(length.value() as usize);
        let string_value = match String::from_utf8(string_value.into()) {
            Ok(val) if val.is_ascii() => val,
            _ => return Err("Non-ascii characters !"),
        };

        Ok(VarStr{length, string_value})
    }
}

impl From<VarStr> for Vec<u8> {
    fn from(var: VarStr) -> Self {
        let mut bytes: Vec<u8> = Vec::<u8>::from(var.length);
        bytes.extend(var.string_value.as_bytes());
        bytes
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_varstr() {
        let var = VarStr::new("Oui".to_string());
        let bytes = Vec::<u8>::from(var);
        assert_eq!(VarStr::try_from(bytes.as_slice()), Ok(VarStr::new("Oui".to_string())));
    }
}
