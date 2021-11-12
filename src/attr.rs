use crate::cp::ClassFileParser;
use crate::cp::ConstantPool;
use crate::cp::ReadFrom;

#[derive(Debug)]
pub enum AttrInfo {
    Code,
    // refers to constant pool
    ConstantValue(u16),
    Deprecated,
    Exceptions,
    LineNumberTable,
    LocalVariableTable,
    SourceFile(String),
    Synthetic,
    Unparsed { name: String, len: usize, info: Vec<u8> },
}

impl ReadFrom for AttrInfo {
    fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> Self {
        let name_i = p.u16() as usize;
        let attr_len = p.u32() as usize;

        match cp.utf8(name_i) {
            "Deprecated" =>  Self::Deprecated,
            "Synthetic" => Self::Synthetic,
            "SourceFile" => {
                let i = p.u16();
                Self::SourceFile(cp.utf8(i as usize).to_string())
            }
            "ConstantValue" => Self::ConstantValue(p.u16()),
            _ => Self::Unparsed { name: cp.utf8(name_i).to_string(), len: attr_len, info: p.bytes(attr_len).to_vec() } 
        }
    }
}