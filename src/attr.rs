use crate::cp::ClassFileParser;
use crate::cp::ConstantPool;

#[derive(Debug)]
pub enum AttrInfo {
    Code,
    ConstantValue,
    Deprecated,
    Exceptions,
    LineNumberTable,
    LocalVariableTable,
    SourceFile,
    Synthetic,
    Unparsed { name: String, len: usize, info: Vec<u8> },
}

impl AttrInfo {
    pub(crate) fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> Self {
        let name_i = p.u16() as usize;
        let attr_len = p.u32() as usize;

        match cp.utf8(name_i) {
            _ => Self::Unparsed { name: cp.utf8(name_i).to_string(), len: attr_len, info: p.bytes(attr_len).to_vec() } 
        }
    }
}