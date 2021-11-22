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

#[derive(Default, Debug)]
pub struct ConstantPool {
    infos: Vec<ConstantInfo>,
}

#[derive(Debug)]
pub enum Constant<'a> {
    // value, wide?
    Primitive(u64, bool),
    // class reference
    ClassRef(u16),
    String(&'a str),
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

impl ConstantPool {
    pub fn new(infos: Vec<ConstantInfo>) -> Self {
        Self { infos }
    }
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

    pub fn infos(&self) -> &[ConstantInfo] {
        &self.infos
    }

    cp_member!(field_ref, ConstantInfo::FieldRef);
    cp_member!(method_ref, ConstantInfo::MethodRef);
    cp_member!(iface_ref, ConstantInfo::IFaceMethodRef);

    pub fn len(&self) -> usize {
        self.infos.len()
    }
}
