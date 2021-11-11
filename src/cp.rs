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
}

#[derive(Default, Debug)]
pub struct ConstantPool {
    n: usize,
    infos: Vec<ConstantInfo>,
}

impl ConstantPool {
    fn read_from(parser: &mut ClassFileParser) -> ConstantPool {
        // n of constant pool
        let n = parser.u16() as usize;
        let mut infos: Vec<ConstantInfo> = Vec::with_capacity(n);
        infos.push(ConstantInfo::Blank);

        // parse constant infos
        for _ in 1..n {

        }

        ConstantPool {
            n,
            infos,
        }
    }

    fn read_info(parser: &mut ClassFileParser) -> ConstantInfo {
        ConstantInfo::Blank
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
    const class: u8 = 7;
    const field_ref: u8 = 9;
    const method_ref:u8 = 10;
    const interface_method_ref:u8 = 11;
    const string:u8 = 8;
    const integer:u8 = 3;
    const float:u8 = 4;
    const long:u8 = 5;
    const double:u8 = 6;
    const name_and_type:u8 = 12;
    const utf8:u8 = 1;
    const method_handle:u8 = 15;
    const method_type:u8 = 16;
    const invoke_dynamic:u8 = 1;
}

impl ClassFile {
    pub fn new(bin: Vec<u8>) -> ClassFile {
        let mut c = ClassFile::default();
        let mut parser = ClassFileParser::new(bin);

        // 魔数, 主次版本号
        c.magic = parser.u32();
        c.minor_version = parser.u16();
        c.major_version = parser.u16();

        // 常量池
        c.cp = ConstantPool::read_from(&mut parser);

        c
    }
}

// 字节流工具类
struct ClassFileParser {
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
    }
}