use std::convert::TryFrom;
use std::convert::TryInto;

use super::address::{ADDRESS_SIZE, Address};
use super::var_uint::VarUint;
use super::var_str::VarStr;
use super::ByteSize;

#[derive(Debug, PartialEq)]
pub struct Whoami {
    version: u32,
    from: Address,
    service_count: VarUint,
    services: Vec<VarStr>,
}

impl Whoami {
    pub fn new(version: u32, from: Address, services: Vec<String>) -> Self {
        let service_count = VarUint::new(services.len() as u64);
        let services: Vec<VarStr> = services.into_iter().map(|s| VarStr::new(s)).collect();

        Whoami {
            version,
            from,
            service_count,
            services,
        }
    }
}

impl ByteSize for Whoami {
    fn byte_size(&self) -> usize {
        4 + self.from.byte_size() +
            self.service_count.byte_size() +
            self.services.iter().map(|s| s.byte_size()).sum::<usize>()
    }
}

impl TryFrom<&[u8]> for Whoami {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let (version, bytes) = bytes.split_at(4);
        let version = u32::from_be_bytes(version.try_into().unwrap());

        let (from, bytes) = bytes.split_at(ADDRESS_SIZE);
        let from = Address::try_from(from)?;

        let service_count = VarUint::try_from(bytes)?;
        let (_, mut bytes) = bytes.split_at(service_count.byte_size());

        let mut services: Vec<VarStr> = Vec::new();
        for _ in 0..service_count.value() {
            let s = VarStr::try_from(bytes)?;
            let (_, b) = bytes.split_at(s.byte_size());
            services.push(s);
            bytes = b;
        }

        Ok(Whoami{
            version,
            from,
            service_count,
            services,
        })
    }
}

impl From<Whoami> for Vec<u8> {
    fn from(whoami: Whoami) -> Self {
        let mut bytes: Vec<u8> = Vec::from(u32::to_be_bytes(whoami.version));
        bytes.extend(Vec::<u8>::from(whoami.from));
        bytes.extend(Vec::<u8>::from(whoami.service_count));
        for service in whoami.services {
            bytes.extend(Vec::<u8>::from(service));
        }

        bytes
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_whoami() {
        let addr = Address::new(431, "127.0.0.1".parse().unwrap(), 10);
        let whoami = Whoami::new(42, addr, Vec::new());
        let bytes = Vec::<u8>::from(whoami);

        let addr = Address::new(431, "127.0.0.1".parse().unwrap(), 10);
        assert_eq!(Whoami::try_from(bytes.as_slice()),
            Ok(Whoami::new(42, addr, Vec::new())));
    }

    #[test]
    fn test_convert_whoami_with_services() {
        let addr = Address::new(431, "127.0.0.1".parse().unwrap(), 10);
        let services = vec!["node".to_string(), "network".to_string()];
        let whoami = Whoami::new(42, addr, services);
        let bytes = Vec::<u8>::from(whoami);

        let addr = Address::new(431, "127.0.0.1".parse().unwrap(), 10);
        let services = vec!["node".to_string(), "network".to_string()];
        assert_eq!(Whoami::try_from(bytes.as_slice()),
            Ok(Whoami::new(42, addr, services)));
    }

    #[test] fn test_whoami_byte_size() {
        let addr = Address::new(431, "127.0.0.1".parse().unwrap(), 10);
        let services = vec!["node".to_string(), "network".to_string()];
        let whoami = Whoami::new(42, addr, services);
        let byte_size = whoami.byte_size();
        let bytes = Vec::<u8>::from(whoami);
        assert_eq!(bytes.len(), byte_size);
    }
}
