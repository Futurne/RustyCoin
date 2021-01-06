pub mod header;

pub mod states;

pub mod var_uint;
pub mod var_str;

trait ByteSize {
    fn byte_size(&self) -> usize;
}
