use core::panic;
use crate::attr::AttrInfo;

#[derive(Default, Debug)]
// java 类文件
pub struct ClassFile {
    // 魔数, CAFEBABE
    magic: u32,
    // 次版本号, 通常是 0
    minor_version: u16,
    // 主版本号, 对于 jdk8 编译出的 class 文件通常是 52
    major_version: u16,

    // 常量池
    cp: ConstantPool,
    // 类访问标志
    access_flags: u16,
    // 类索引, refers to cp ClassInfo
    this_class_i: u16,
    // 超类索引 refers to cp ClassInfo
    super_class_i: u16,
    // 接口索引 refers to cp ClassInfo
    interfaces_i: Vec<u16>,

    // fields
    fields: Vec<MemberInfo>,

    // methods
    methods: Vec<MemberInfo>,
}

#[derive(Default, Debug)]
pub struct MemberInfo {
    access_flags: u16,
    // refers to constant pool utf8
    name_i: u16,
    // refers to constant pool utf8
    desc_i: u16,
    attrs: Vec<AttrInfo>,
}

impl MemberInfo {
    pub fn read_from(p: &mut ClassFileParser, cp: &ConstantPool) -> MemberInfo {
        let mut m = MemberInfo { access_flags: p.u16(), name_i: p.u16(), desc_i: p.u16(), attrs: Vec::new() };
        // parse attributes
        let attr_len = p.u16();
        let mut attrs: Vec<AttrInfo> = Vec::with_capacity(attr_len as usize);
        for _ in 0..attr_len {
            attrs.push(AttrInfo::read_from(p, cp))
        }
        m.attrs = attrs;
        m
    }
}

#[derive(Default, Debug)]
pub struct ConstantPool {
    n: usize,
    infos: Vec<ConstantInfo>,
}

impl ConstantPool {
    pub fn read_from(parser: &mut ClassFileParser) -> ConstantPool {
        // n of constant pool
        let n = parser.u16() as usize;
        let mut infos: Vec<ConstantInfo> = Vec::with_capacity(n);
        infos.push(ConstantInfo::Blank);

        // parse constant infos
        for _ in 1..n {
            infos.push(Self::read_info(parser))
        }

        ConstantPool {
            n,
            infos,
        }
    }
    
    pub fn utf8(&self, i: usize) -> &str {
        match self.infos[i as usize] {
            ConstantInfo::Utf8(ref a) => &a,
            _ => panic!("invalid utf8 index")
        }
    }

    pub fn class(&self, i: usize) -> &str {
        let j = match self.infos[i as usize] {
            ConstantInfo::Class { name_i} => name_i,
            _ => panic!("invalid class index")
        };
        self.utf8(j as usize)
    }

    fn read_info(p: &mut ClassFileParser) -> ConstantInfo {
        let tag = p.u8();
        use ct_info_tag::*;
        match tag {
            INTEGER => ConstantInfo::Integer(p.u32() as i32),
            FLOAT => ConstantInfo::Float(f32::from_bits(p.u32())),
            LONG => ConstantInfo::Long(p.u64() as i64),
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
            _ => panic!("unknown tag {}", tag)
        }
    }
}

// 常量池
// 常量池的实际大小是 n - 1
// 常量池的有效索引是 1~n-1, 0 是无效索引
// CONSTANT_Long_info 和 CONSTANT_Double_info 各占两个位置, 如果常量池存在这两种常量, 实际的常量比 n - 1 还要少
#[derive(Debug)]
pub enum ConstantInfo {
    // since index ranges from 1~n-1, fill blank into zero entry
    Blank,
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Utf8(String),
    String {
        // index refers to utf8
        utf8_i: u16
    },
    Class { 
        // index refers to utf8
        name_i: u16 
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
}

mod ct_info_tag {
    pub const CLASS: u8 = 7;
    pub const FIELD_REF: u8 = 9;
    pub const METHOD_REF:u8 = 10;
    pub const INTERFACE_METHOD_REF:u8 = 11;
    pub const STRING:u8 = 8;
    pub const INTEGER:u8 = 3;
    pub const FLOAT:u8 = 4;
    pub const LONG:u8 = 5;
    pub const DOUBLE:u8 = 6;
    pub const NAME_AND_TYPE:u8 = 12;
    pub const UTF8:u8 = 1;
    pub const METHOD_HANDLE:u8 = 15;
    pub const METHOD_TYPE:u8 = 16;
    pub const INVOKE_DYNAMIC:u8 = 18;
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
        c.cp = ConstantPool::read_from(&mut p);
        // 类访问标志, 类索引, 超类索引, 接口索引 
        c.access_flags = p.u16();
        c.this_class_i = p.u16();
        c.super_class_i = p.u16();
        let iface_cnt = p.u16();
        c.interfaces_i = Vec::with_capacity(iface_cnt as usize);
        for _ in 0..c.interfaces_i.capacity() {
            c.interfaces_i.push(p.u16());
        }

        // fields and methods
        c.fields = Self::read_members(&mut p, &c.cp);
        c.methods = Self::read_members(&mut p, &c.cp);
        c
    }

    fn read_members(p: &mut ClassFileParser, cp: &ConstantPool) -> Vec<MemberInfo> {
        let n = p.u16() as usize;
        let mut v: Vec<MemberInfo> = Vec::with_capacity(n);

        for _ in 0..n {
            v.push(MemberInfo::read_from(p, cp));
        }
        v
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
        let mut c = ClassFileParser {
            bin: bin,
            off: 0,
        };
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

    pub fn u16_vec(&mut self) -> Vec<u16> {
        let len = self.u16();
        let mut r: Vec<u16> = Vec::with_capacity(len as usize);

        for _ in 0..len {
            r.push(self.u16());
        }
        r
    }
}


#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};
    use std::fs::File;
    use std::io::Read;
    use std::fs;

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
        let bin = get_test_file();
        let c = ClassFile::new(bin);
        println!("{:#?}", c);
        assert_eq!(c.magic, 0xCAFEBABE);

        println!("{}", c.this_class());
        println!("{}", c.super_class());

        for i in 0..c.interfaces_i.len() {
            println!("interface {}", c.interface(i))
        }
    }
}