pub mod header;
pub mod whoami;

pub mod address;
pub mod var_uint;
pub mod var_str;

pub mod states;

pub trait ByteSize {
    fn byte_size(&self) -> usize;
}
