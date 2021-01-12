use std::convert::TryFrom;
use std::convert::TryInto;
use std::net::{IpAddr, Ipv6Addr};

use super::ByteSize;

#[derive(Debug, PartialEq, Clone)]
pub struct Address {
    timestamp: u64,
    addr: Ipv6Addr,
    port: u16,
}

pub const ADDRESS_SIZE: usize = 8 + 16 + 2;

impl Address {
    pub fn new(timestamp: u64, addr: IpAddr, port: u16) -> Self {
        let addr: Ipv6Addr = match addr {
            IpAddr::V4(addr) => addr.to_ipv6_compatible(),
            IpAddr::V6(addr) => addr,
        };

        Address {
            timestamp,
            addr,
            port,
        }
    }
}

impl ByteSize for Address {
    fn byte_size(&self) -> usize {
        8 + 16 + 2
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < ADDRESS_SIZE {
            return Err("Slice is not big enough");
        }

        let (timestamp, bytes) = bytes.split_at(8);
        let timestamp = u64::from_be_bytes(timestamp.try_into().unwrap());

        let (addr, bytes) = bytes.split_at(16);
        let addr: [u8; 16] = addr.try_into().unwrap();
        let addr = Ipv6Addr::from(addr);

        let (port, _) = bytes.split_at(2);
        let port = u16::from_be_bytes(port.try_into().unwrap());

        Ok(Address {
            timestamp,
            addr,
            port,
        })
    }
}

impl From<Address> for Vec<u8> {
    fn from(addr: Address) -> Self {
        let mut bytes = Vec::new();
        bytes.extend(&u64::to_be_bytes(addr.timestamp));
        bytes.extend(&addr.addr.octets());
        bytes.extend(&u16::to_be_bytes(addr.port));
        bytes
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_address() {
        let var = Address::new(431, "127.0.0.1".parse().unwrap(), 10);
        let bytes = Vec::<u8>::from(var);
        assert_eq!(Address::try_from(bytes.as_slice()),
            Ok(Address::new(431, "127.0.0.1".parse().unwrap(), 10)));
    }
}
