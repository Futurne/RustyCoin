use std::convert::TryFrom;
use std::convert::TryInto;

/// Header of a message.
/// A node sending a message should always start his message with this structure.
///
/// Note: The `msg_type` is a field representing 12 ascii characters.
struct Header {
    magic: u32,
    msg_type: String,  // Chars can only be ascii 8-bit characters.
    length: u64
}

impl Header {
    pub fn new(magic: u32, msg_type: String, length: u64)
        -> Result<Self, &'static str> {
        if !msg_type.is_ascii() {
            return Err("Message type can only contains ascii characters !");
        }

        if msg_type.len() > 12 {
            return Err("Message type can not be greater than 12 characters !");
        }

        Ok(Header{magic, msg_type, length})
    }
}

impl TryFrom<Vec<u8>> for Header {
    type Error = &'static str;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.len() < 4 + 12 + 8 {
            return Err("Vec is not big enough to be parsed into Header");
        }

        let (magic, bytes) = bytes.split_at(4);
        let magic = u32::from_be_bytes(magic.try_into().unwrap());

        let (msg_type, length) = bytes.split_at(12);
        let msg_type: Vec<u8> = msg_type.iter()
                                        .filter(|&&el| el != 0) // Removes the empty characters to avoid later errors
                                        .map(|&el| el)
                                        .collect();
        let msg_type = match String::from_utf8(msg_type.into()) {
            Ok(val) if val.is_ascii() => val,
            _ => return Err("Non-ascii characters !"),
        };

        let length = u64::from_be_bytes(length.try_into().unwrap());
        Ok(Header{magic, msg_type, length})
    }
}

impl From<Header> for Vec<u8> {
    fn from(header: Header) -> Self {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.extend(&u32::to_be_bytes(header.magic));

        bytes.extend(header.msg_type.as_bytes());
        bytes.extend(vec![0; 4 + 12 - bytes.len()]);  // Fill the rest of the msg_type with empty chars

        bytes.extend(&u64::to_be_bytes(header.length));

        bytes
    }
}
