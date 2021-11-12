use crate::cp::ClassFileParser;
use crate::cp::ConstantPool;
use crate::cp::ReadFrom;

#[derive(Debug)]
pub enum AttrInfo {
    Code(Code),
    // refers to constant pool constant long, constant float, constant double, constant integer, constant string
    ConstantValue(u16),
    Deprecated,
    // class names
    Exceptions(Vec<String>),
    // start pc -> line number
    LineNumberTable(Vec<LineNumber>),
    LocalVariableTable(Vec<LocalVariable>),
    SourceFile(String),
    Synthetic,
    Unparsed { name: String, len: usize, info: Vec<u8> },
}

#[derive(Debug, Default)]
pub struct LineNumber {
    start_pc: u16,
    line_number: u16,
}

impl ReadFrom for LineNumber {
    fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> Self {
        Self { start_pc: p.u16(), line_number: p.u16()}
    }
}

#[derive(Debug, Default)]
pub struct LocalVariable {
    start_pc: u16,
    length: u16,
    name_i: u16,
    desc_i: u16,
    index: u16,
    name: String,
    desc: String,
}

impl ReadFrom for LocalVariable {
    fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> Self {
        let mut r = Self::default();
        r.start_pc = p.u16();
        r.length = p.u16();
        r.name_i = p.u16();
        r.desc_i = p.u16();
        r.index = p.u16();
        r.name = cp.utf8(r.name_i as usize).to_string();
        r.desc = cp.utf8(r.desc_i as usize).to_string();
        r
    }
}

#[derive(Debug, Default)]
pub struct Exception {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

impl ReadFrom for Exception {
    fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> Self {
        Exception {
            start_pc: p.u16(),
            end_pc: p.u16(),
            handler_pc: p.u16(),
            catch_type: p.u16(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Code {
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    exceptions: Vec<Exception>,
    attrs: Vec<AttrInfo>,
}

impl ReadFrom for Code {
    fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> Self {
        let ms = p.u16();
        let ml = p.u16();
        let code_len = p.u32() as usize;
        let code = p.bytes(code_len).to_vec();

        Self {
            max_stack: ms,
            max_locals: ml,
            code: code,
            exceptions: Exception::read_vec_from(p, cp),
            attrs: AttrInfo::read_vec_from(p, cp),
        }
    }
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
            "Code" => Self::Code(Code::read_from(p, cp)),
            "LineNumberTable" => Self::LineNumberTable(LineNumber::read_vec_from(p, cp)),
            "LocalVariableTable" => Self::LocalVariableTable(LocalVariable::read_vec_from(p, cp)),
            // constant pool index refers to class info
            "Exceptions" => Self::Exceptions(u16::read_vec_from(p, cp).into_iter().map(|x| cp.class(x as usize).to_string()).collect()),
            _ => Self::Unparsed { name: cp.utf8(name_i).to_string(), len: attr_len, info: p.bytes(attr_len).to_vec() } 
        }
    }
}