use crate::attr::AttrInfo;
use core::panic;

pub trait ReadFrom: Sized {
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

#[derive(Default, Debug)]
// java 类文件
pub struct ClassFile {
    // 魔数, CAFEBABE
    pub magic: u32,
    // 次版本号, 通常是 0
    pub minor_version: u16,
    // 主版本号, 对于 jdk8 编译出的 class 文件通常是 52
    pub major_version: u16,

    // 常量池
    pub cp: ConstantPool,
    // 类访问标志
    pub access_flags: u16,
    // 类索引, refers to cp ClassInfo
    this_class_i: u16,
    // 超类索引 refers to cp ClassInfo
    super_class_i: u16,
    // 接口索引 refers to cp ClassInfo
    pub interfaces_i: Vec<u16>,

    // fields
    pub fields: Vec<MemberInfo>,

    // methods
    pub methods: Vec<MemberInfo>,

    // class attributes
    pub attrs: Vec<AttrInfo>,
}

#[derive(Default, Debug)]
pub struct MemberInfo {
    pub access_flags: u16,
    // refers to constant pool utf8
    pub name_i: u16,
    pub desc_i: u16,
    pub name: String,
    pub desc: String,
    pub attrs: Vec<AttrInfo>,
}

impl ReadFrom for MemberInfo {
    fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> MemberInfo {
        let a = p.u16();
        let name_i = p.u16();
        let desc_i = p.u16();
        let mut m = MemberInfo {
            access_flags: a,
            name_i: name_i,
            desc_i: desc_i,
            attrs: Vec::new(),
            name: cp.utf8(name_i as usize).to_string(),
            desc: cp.utf8(desc_i as usize).to_string(),
        };
        // parse attributes
        m.attrs = AttrInfo::read_vec_from(p, cp);
        m
    }
}

#[derive(Default, Debug)]
pub struct ConstantPool {
    infos: Vec<ConstantInfo>,
}

impl ReadFrom for ConstantPool {
    fn read_from(parser: &mut ClassFileParser, cp: &ConstantPool) -> ConstantPool {
        // n of constant pool
        let n = parser.u16() as usize;

        let mut infos: Vec<ConstantInfo> = vec![ConstantInfo::Blank; n];

        let mut i = 1;
        // parse constant infos
        while i < n {
            let info = ConstantInfo::read_from(parser, cp);

            infos[i] = info;
            match infos[i] {
                ConstantInfo::Double(_) | ConstantInfo::Long(_) => i += 2,
                _ => i += 1,
            }
        }

        ConstantPool { infos }
    }
}

macro_rules! cp_member {
    ($n: ident, $t: path) => {
        // field index -> (class, name, desc)
        pub fn $n(&self, i: usize) -> (&str, &str, &str) {
            let j = match self.infos[i as usize] {
                $t {
                    class_i,
                    name_type_i,
                } => (class_i, name_type_i),
                _ => panic!("invalid index"),
            };

            let name_type = match self.infos[j.1 as usize] {
                ConstantInfo::NameAndType { name_i, desc_i } => (name_i, desc_i),
                _ => panic!("invalid name type index"),
            };

            (
                self.class(j.0 as usize),
                self.utf8(name_type.0 as usize),
                self.utf8(name_type.1 as usize),
            )
        }
    };
}

#[derive(Debug)]
pub enum Constant<'a> {
    // value, wide?
    Primitive(u64, bool),
    // class reference
    ClassRef(u16),
    String(&'a str),
}

impl ConstantPool {
    pub fn utf8(&self, i: usize) -> &str {
        match self.infos[i as usize] {
            ConstantInfo::Utf8(ref a) => &a,
            _ => panic!("invalid utf8 index"),
        }
    }

    pub fn class(&self, i: usize) -> &str {
        let j = match self.infos[i as usize] {
            ConstantInfo::Class { name_i } => name_i,
            _ => panic!("invalid class index {}", i),
        };
        self.utf8(j as usize)
    }

    pub fn u32(&self, i: usize) -> u32 {
        match self.infos[i] {
            ConstantInfo::Integer(j) => j as u32,
            _ => panic!("invalid u32 index found"),
        }
    }

    pub fn constant(&self, i: usize) -> Constant {
        match self.infos[i] {
            ConstantInfo::Integer(j) => Constant::Primitive(j as u64, false),
            ConstantInfo::Float(j) => Constant::Primitive(j.to_bits() as u64, false),
            ConstantInfo::Long(j) => Constant::Primitive(j, true),
            ConstantInfo::Double(j) => Constant::Primitive(j.to_bits(), true),
            ConstantInfo::Class { name_i } => Constant::ClassRef(name_i),
            ConstantInfo::String { utf8_i } => Constant::String(self.utf8(utf8_i as usize)),
            _ => panic!("invalid constant {} {:?}", i, self.infos[i]),
        }
    }

    pub fn f32(&self, i: usize) -> f32 {
        match self.infos[i] {
            ConstantInfo::Float(j) => j,
            _ => panic!("invalid integer index"),
        }
    }

    pub fn u64(&self, i: usize) -> u64 {
        match self.infos[i] {
            ConstantInfo::Long(j) => j,
            _ => panic!("invalid u64 index"),
        }
    }

    pub fn f64(&self, i: usize) -> f64 {
        match self.infos[i] {
            ConstantInfo::Double(j) => j,
            _ => panic!("invalid f64 index"),
        }
    }

    pub fn string(&self, i: usize) -> &str {
        let j = match self.infos[i as usize] {
            ConstantInfo::String { utf8_i } => utf8_i as usize,
            _ => panic!("invalid string index"),
        };
        self.utf8(j as usize)
    }

    cp_member!(field_ref, ConstantInfo::FieldRef);
    cp_member!(method_ref, ConstantInfo::MethodRef);
    cp_member!(iface_ref, ConstantInfo::IFaceMethodRef);

    pub fn len(&self) -> usize {
        self.infos.len()
    }
}

// 常量池
// 常量池的实际大小是 n - 1
// 常量池的有效索引是 1~n-1, 0 是无效索引
// CONSTANT_Long_info 和 CONSTANT_Double_info 各占两个位置, 如果常量池存在这两种常量, 实际的常量比 n - 1 还要少
#[derive(Debug, Clone)]
pub enum ConstantInfo {
    // since index ranges from 1~n-1, fill blank into zero entry
    Blank,
    Integer(u32),
    Float(f32),
    Long(u64),
    Double(f64),
    Utf8(String),
    String {
        // index refers to utf8
        utf8_i: u16,
    },
    Class {
        // index refers to utf8
        name_i: u16,
    },
    NameAndType {
        // index refers to utf8
        // name of field or method
        name_i: u16,
        // index refers to utf8
        // type descriptor
        // for basic type: byte,short,char,int,long,float,double -> B,S,C,I,J,F,D
        // reference type starts with L, ends wiht ';' , e.g. Ljava.lang.Object;
        // array starts with [, e.g. [I -> int[]
        //
        desc_i: u16,
    },
    // field reference
    FieldRef {
        // index refers to class
        class_i: u16,
        // index refers to name and type
        name_type_i: u16,
    },
    // method reference, not interface
    MethodRef {
        // index refers to class
        class_i: u16,
        // index refers to name and type
        name_type_i: u16,
    },
    // interface method reference
    IFaceMethodRef {
        // index refers to class
        class_i: u16,
        // index refers to name and type
        name_type_i: u16,
    },
    InvokeDynamic {
        boot_i: u16,
        name_type_i: u16,
    },
    MethodHandle {
        ref_kind: u8,
        ref_i: u16,
    },

    MethodType {
        desc_i: u16,
    },
}

impl ReadFrom for ConstantInfo {
    fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> ConstantInfo {
        let tag = p.u8();
        use ct_info_tag::*;
        match tag {
            INTEGER => ConstantInfo::Integer(p.u32()),
            FLOAT => ConstantInfo::Float(f32::from_bits(p.u32())),
            LONG => ConstantInfo::Long(p.u64()),
            DOUBLE => ConstantInfo::Double(f64::from_bits(p.u64())),
            UTF8 => {
                let str_len = p.u16() as usize;
                let bytes = p.bytes(str_len);
                let utf8 = mutf8::mutf8_to_utf8(bytes).unwrap();
                let s = String::from_utf8(utf8.into_owned()).unwrap();
                ConstantInfo::Utf8(s)
            }
            STRING => ConstantInfo::String { utf8_i: p.u16() },
            CLASS => ConstantInfo::Class { name_i: p.u16() },
            NAME_AND_TYPE => ConstantInfo::NameAndType {
                name_i: p.u16(),
                desc_i: p.u16(),
            },
            FIELD_REF => ConstantInfo::FieldRef {
                class_i: p.u16(),
                name_type_i: p.u16(),
            },
            METHOD_REF => ConstantInfo::MethodRef {
                class_i: p.u16(),
                name_type_i: p.u16(),
            },
            INTERFACE_METHOD_REF => ConstantInfo::IFaceMethodRef {
                class_i: p.u16(),
                name_type_i: p.u16(),
            },
            INVOKE_DYNAMIC => ConstantInfo::InvokeDynamic {
                boot_i: p.u16(),
                name_type_i: p.u16(),
            },

            METHOD_HANDLE => ConstantInfo::MethodHandle {
                ref_kind: p.u8(),
                ref_i: p.u16(),
            },

            METHOD_TYPE => ConstantInfo::MethodType { desc_i: p.u16() },
            _ => panic!("unknown tag {}", tag),
        }
    }
}

mod ct_info_tag {
    pub const CLASS: u8 = 7;
    pub const FIELD_REF: u8 = 9;
    pub const METHOD_REF: u8 = 10;
    pub const INTERFACE_METHOD_REF: u8 = 11;
    pub const STRING: u8 = 8;
    pub const INTEGER: u8 = 3;
    pub const FLOAT: u8 = 4;
    pub const LONG: u8 = 5;
    pub const DOUBLE: u8 = 6;
    pub const NAME_AND_TYPE: u8 = 12;
    pub const UTF8: u8 = 1;
    pub const METHOD_HANDLE: u8 = 15;
    pub const METHOD_TYPE: u8 = 16;
    pub const INVOKE_DYNAMIC: u8 = 18;
}

impl ClassFile {
    pub fn new(bin: Vec<u8>) -> ClassFile {
        let mut c = ClassFile::default();
        let mut p = ClassFileParser::new(bin);

        // 魔数, 主次版本号
        c.magic = p.u32();
        c.minor_version = p.u16();
        c.major_version = p.u16();

        // 常量池
        c.cp = ConstantPool::read_from(&mut p, &c.cp);
        // 类访问标志, 类索引, 超类索引, 接口索引
        c.access_flags = p.u16();
        c.this_class_i = p.u16();
        c.super_class_i = p.u16();

        c.interfaces_i = u16::read_vec_from(&mut p, &c.cp);

        // fields and methods
        c.fields = MemberInfo::read_vec_from(&mut p, &c.cp);
        c.methods = MemberInfo::read_vec_from(&mut p, &c.cp);
        c.attrs = AttrInfo::read_vec_from(&mut p, &c.cp);
        c
    }

    // name of this class
    pub fn this_class(&self) -> &str {
        self.cp.class(self.this_class_i as usize)
    }

    // name of super class, return "" if no super class
    pub fn super_class(&self) -> &str {
        if self.super_class_i == 0 {
            ""
        } else {
            self.cp.class(self.super_class_i as usize)
        }
    }

    pub fn interface_len(&self) -> usize {
        self.interfaces_i.len()
    }

    // interface list
    pub fn interface(&self, i: usize) -> &str {
        let j = self.interfaces_i[i];
        self.cp.class(j as usize)
    }
}

// 字节流工具类
pub struct ClassFileParser {
    bin: Vec<u8>,
    off: usize,
}

macro_rules! cp_u_n {
    ($a: ident, $sf: ident, $w: expr) => {
       pub fn $a(&mut $sf) -> $a {
            let s = &$sf.bin[$sf.off..$sf.off + $w];
            $sf.off += $w;
            let mut b = [0u8; $w];
            b.copy_from_slice(s);
            $a::from_be_bytes(b)
       }
    };
}

impl ClassFileParser {
    pub fn new(bin: Vec<u8>) -> ClassFileParser {
        let c = ClassFileParser { bin: bin, off: 0 };
        c
    }

    pub fn u8(&mut self) -> u8 {
        let r = self.bin[self.off];
        self.off += 1;
        r
    }

    cp_u_n!(u16, self, 2);
    cp_u_n!(u32, self, 4);
    cp_u_n!(u64, self, 8);

    pub fn bytes(&mut self, len: usize) -> &[u8] {
        let s = &self.bin[self.off..self.off + len];
        self.off += len;
        s
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::path::{Path, PathBuf};

    use crate::entry::{DirEntry, Entry};

    use super::{ClassFile, ClassFileParser};

    fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
        let mut f = File::open(&filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
        buffer
    }

    fn get_test_file() -> Vec<u8> {
        let this_file = file!();
        let mut buf = PathBuf::new();
        buf.push(this_file);
        buf.pop();
        buf.push("../test/Test.class");
        let p = buf.to_str().unwrap();
        println!("path = {}", p);
        get_file_as_byte_vec(p)
    }

    #[test]
    fn parser_test() {
        let bin = get_test_file();
        let mut p = ClassFileParser::new(bin);
        assert_eq!(p.u32(), 0xCAFEBABE);
    }

    #[test]
    fn classfile_test() {
        let e = DirEntry::new(".").unwrap();
        let c = ClassFile::new(e.read_class("test/Test").unwrap());
        println!("{:#?}", c);
        assert_eq!(c.magic, 0xCAFEBABE);

        println!("{}", c.this_class());
        println!("{}", c.super_class());

        for i in 0..c.interfaces_i.len() {
            println!("interface {}", c.interface(i))
        }
    }
}
