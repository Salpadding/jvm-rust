mod attr;
mod class_file;
mod cp;
mod entry;
mod io_utils;

#[macro_use]
extern crate err;

pub use crate::attr::*;
pub use crate::class_file::*;
pub use crate::cp::*;
pub use crate::entry::{new_entry, Entry};

trait ReadFrom: Sized {
    fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> Self;

    fn read_vec_from(p: &mut ClassFileParser, cp: &ConstantPool) -> Vec<Self> {
        let n = p.u16() as usize;
        let mut v: Vec<Self> = Vec::with_capacity(n);
        for _ in 0..n {
            v.push(Self::read_from(p, cp));
        }
        v
    }
}

impl ReadFrom for u16 {
    fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> Self {
        p.u16()
    }
}

pub(crate) struct ClassFileParser {
    bin: Vec<u8>,
    off: usize,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
