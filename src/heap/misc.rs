use crate::heap::class::{Class, Object};
use crate::heap::loader::ClassLoader;
use crate::rp::{Rp, Unmanged};
use crate::StringErr;

use super::class::ClassMember;

#[derive(Debug, Default)]
pub struct AccessFlags(pub u16);

macro_rules! is_xx {
    ($f: ident, $b: expr) => {
        pub fn $f(&self) -> bool {
            self.0 & $b != 0
        }
    };
}

impl AccessFlags {
    is_xx!(is_public, flags::ACC_PUBLIC);
    is_xx!(is_private, flags::ACC_PRIVATE);
    is_xx!(is_protected, flags::ACC_PROTECTED);
    is_xx!(is_static, flags::ACC_STATIC);
    is_xx!(is_final, flags::ACC_FINAL);
    is_xx!(is_super, flags::ACC_SUPER);
    is_xx!(is_sync, flags::ACC_SYNCHRONIZED);
    is_xx!(is_volatile, flags::ACC_VOLATILE);
    is_xx!(is_bridge, flags::ACC_BRIDGE);
    is_xx!(is_transient, flags::ACC_TRANSIENT);
    is_xx!(is_varargs, flags::ACC_VARARGS);
    is_xx!(is_native, flags::ACC_NATIVE);
    is_xx!(is_iface, flags::ACC_INTERFACE);
    is_xx!(is_abstract, flags::ACC_ABSTRACT);
    is_xx!(is_strict, flags::ACC_STRICT);
    is_xx!(is_synthetic, flags::ACC_SYNTHETIC);
    is_xx!(is_annotation, flags::ACC_ANNOTATION);
    is_xx!(is_enum, flags::ACC_ENUM);
}

mod flags {
    pub const ACC_PUBLIC: u16 = 0x0001; // class field method
    pub const ACC_PRIVATE: u16 = 0x0002; //       field method
    pub const ACC_PROTECTED: u16 = 0x0004; //       field method
    pub const ACC_STATIC: u16 = 0x0008; //       field method
    pub const ACC_FINAL: u16 = 0x0010; // class field method
    pub const ACC_SUPER: u16 = 0x0020; // class
    pub const ACC_SYNCHRONIZED: u16 = 0x0020; //             method
    pub const ACC_VOLATILE: u16 = 0x0040; //       field
    pub const ACC_BRIDGE: u16 = 0x0040; //             method
    pub const ACC_TRANSIENT: u16 = 0x0080; //       field
    pub const ACC_VARARGS: u16 = 0x0080; //             method
    pub const ACC_NATIVE: u16 = 0x0100; //             method
    pub const ACC_INTERFACE: u16 = 0x0200; // class
    pub const ACC_ABSTRACT: u16 = 0x0400; // class       method
    pub const ACC_STRICT: u16 = 0x0800; //             method
    pub const ACC_SYNTHETIC: u16 = 0x1000; // class field method
    pub const ACC_ANNOTATION: u16 = 0x2000; // class
    pub const ACC_ENUM: u16 = 0x4000; // class field
}

#[derive(Debug)]
pub struct Heap {
    pub loader: ClassLoader,
}

impl Unmanged for Heap {}

macro_rules! xx_ref {
    ($s: ident, $c: ident, $i: ident, $f1: ident, $f2: ident) => {{
        let r = &mut $c.sym_refs[$i];

        if !r.is_null() {
            return *r;
        }

        let (class_name, name, desc) = $c.cp.$f1($i);
        let class = $s.loader.load(class_name);
        let m = class.$f2(name, desc);

        let sym = SymRef {
            class: class,
            name: name.to_string(),
            desc: desc.to_string(),
            member: m,
        };

        *r = Rp::new(sym);
        *r
    }};
}

impl Heap {
    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let loader = ClassLoader::new(cp)?;
        Ok(Heap { loader })
    }

    pub fn class_ref(&mut self, cur: &mut Class, i: usize) -> Rp<SymRef> {
        let r = &mut cur.sym_refs[i];

        if !r.is_null() {
            return *r;
        }

        let name = cur.cp.class(i);
        let class = self.loader.load(name);
        let sym = SymRef {
            class,
            name: name.to_string(),
            desc: "".to_string(),
            member: Rp::null(),
        };

        *r = Rp::new(sym);
        *r
    }

    pub fn field_ref(&mut self, cur: &mut Class, i: usize) -> Rp<SymRef> {
        xx_ref!(self, cur, i, field_ref, lookup_field)
    }

    pub fn method_ref(&mut self, cur: &mut Class, i: usize) -> Rp<SymRef> {
        xx_ref!(self, cur, i, method_ref, lookup_method)
    }

    pub fn iface_ref(&mut self, cur: &mut Class, i: usize) -> Rp<SymRef> {
        xx_ref!(self, cur, i, iface_ref, lookup_iface_method)
    }

    pub fn new_obj(&self, class: Rp<Class>) -> Rp<Object> {
        let obj = Object {
            class: class,
            fields: vec![0u64; class.ins_fields.len()],
        };

        Rp::new(obj)
    }
}

impl Unmanged for SymRef {}
#[derive(Debug, Clone)]
pub struct SymRef {
    pub class: Rp<Class>,
    pub name: String,
    pub desc: String,
    pub member: Rp<ClassMember>,
}

pub struct DescriptorParser<'a> {
    bytes: &'a [u8],
    off: usize,
}

#[derive(Debug, Default)]
pub struct MethodDescriptor {
    pub params: Vec<String>,
    pub jtypes: Vec<JType>,
    pub ret: String,
    pub arg_cells: usize,
}

#[derive(Debug)]
pub enum JType {
    IF,
    DJ,
    A,
}

pub trait JTypeDescriptor {
    fn jtype(&self) -> JType;
}

impl JTypeDescriptor for str {
    fn jtype(&self) -> JType {
        match self.as_bytes()[0] {
            b'Z' | b'B' | b'C' | b'S' | b'I' | b'F' => JType::IF,
            b'D' | b'J' => JType::DJ,
            _ => JType::A,
        }
    }
}

impl DescriptorParser<'_> {
    pub fn new(bytes: &[u8]) -> DescriptorParser {
        DescriptorParser { bytes, off: 0 }
    }

    fn u8(&mut self) -> u8 {
        let u = self.bytes[self.off];
        self.off += 1;
        u
    }

    fn peek(&self) -> u8 {
        self.bytes[self.off]
    }

    pub fn parse_method(&mut self) -> MethodDescriptor {
        if self.peek() != b'(' {
            panic!("not a method descriptor");
        }

        self.u8();

        let params = self.parse_params();
        let jtypes: Vec<JType> = params.iter().map(|x| x.jtype()).collect();
        self.u8();

        let ret = self.parse_param();

        let mut r = MethodDescriptor {
            params,
            ret,
            jtypes,
            arg_cells: 0
        };

        for t in r.jtypes.iter() {
            match t {
                JType::A | JType::IF => { r.arg_cells += 1},
                _ => { r.arg_cells += 2},
            }
        }

        r
    }

    fn parse_params(&mut self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        while self.peek() != b')' {
            v.push(self.parse_param());
        }
        v
    }

    fn parse_param(&mut self) -> String {
        if self.peek() == b'[' {
            return self.parse_arr();
        }
        return self.parse_no_array();
    }

    fn parse_arr(&mut self) -> String {
        let mut s = String::new();

        while self.peek() == b'[' {
            s.push(self.u8() as char);
        }

        s.push_str(&self.parse_no_array());
        s
    }

    fn parse_no_array(&mut self) -> String {
        let cur = self.u8();
        match cur {
            b'B' | b'Z' | b'J' | b'I' | b'D' | b'F' | b'S' | b'V' => {
                return unsafe {
                    String::from_utf8_unchecked(self.bytes[self.off - 1..self.off].to_vec())
                };
            }
            b'L' => {
                let mut s = String::new();

                while self.peek() != b';' {
                    s.push(self.u8() as char);
                }

                self.u8();
                return s;
            }
            _ => panic!("parse no array failed"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::{AccessFlags, MethodDescriptor};
    use crate::heap::{loader::ClassLoader, misc::DescriptorParser};

    #[test]
    fn loader_test() {
        let mut loader = ClassLoader::new(".:test/rt.jar").unwrap();
        let class = loader.load("test/MyObject");
        let r = class.as_ref();
        println!("{:#?}", r);
    }

    #[test]
    fn method_test() {
        let method = "(Ljava.lang.Object;[[IIIF)V";
        let mut parser = DescriptorParser::new(method.as_bytes());
        let params = parser.parse_method();

        println!("{:?}", params);
    }
}
