use std::collections::BTreeMap;

use crate::heap::class::{Class, Object};
use crate::heap::loader::ClassLoader;
use crate::StringErr;
use rp::Rp;

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

pub mod flags {
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

pub const PRIMITIVE_N: usize = 8;

pub mod primitives {
    pub const Z: usize = 0;
    pub const C: usize = 1;
    pub const F: usize = 2;
    pub const D: usize = 3;
    pub const B: usize = 4;
    pub const S: usize = 5;
    pub const I: usize = 6;
    pub const J: usize = 7;
}

pub static PRIMITIVES: [&str; 8] = [
    "boolean", "char", "float", "double", "byte", "short", "int", "long",
];

pub static PRIMITIVE_DESC: [&str; 8] = ["Z", "C", "F", "D", "B", "S", "I", "J"];

pub struct Heap {
    pub loader: Rp<ClassLoader>,
    primitives: Vec<Rp<Class>>,
    primitive_array: Vec<Rp<Class>>,
    string_pool: BTreeMap<String, Rp<Object>>,
    pub java_lang_string: Rp<Class>,
}

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

macro_rules! arm {
    ($x: expr, $sz: expr, $t: ty) => {
        $x => Rp::new(Object {
                class,
                data: Rp::<$t>::new_vec($sz).ptr(),
        })
    };
}

macro_rules! arr {
    ($c: expr, $t: ty, $sz: expr) => {{
        let o = Object {
            class: $c,
            data: Rp::<$t>::new_a($sz).ptr(),
            size: $sz,
        };
        Rp::new(o)
    }};
}

impl Heap {
    pub fn new(cp: &str) -> Result<Rp<Self>, StringErr> {
        let mut h = Rp::new(Heap {
            loader: Rp::null(),
            primitives: Vec::new(),
            primitive_array: Vec::new(),
            java_lang_string: Rp::null(),
            string_pool: BTreeMap::new(),
        });

        let mut l = ClassLoader::new(cp, h)?;
        // int long char short boolean byte double float

        // cache is primitive types
        for p in PRIMITIVES.iter() {
            h.primitives.push(l.load(p));
        }

        for p in PRIMITIVE_DESC.iter() {
            h.primitive_array.push(l.load(&format!("[{}", p)))
        }

        Ok(h)
    }

    // create a string from pool
    pub fn new_jstr(&mut self, s: &str) -> Rp<Object> {
        let x = self.string_pool.get(s).map(|x| *x).unwrap_or(Rp::null());

        if !x.is_null() {
            return x;
        }

        let o = self.create_jstr(s);
        self.string_pool.insert(s.to_string(), o);
        o
    }

    fn create_jstr(&mut self, s: &str) -> Rp<Object> {
        let mut o = Class::new_obj(self.java_lang_string);
        self.string_pool.insert(s.to_string(), o);

        // set first field
        let v: Vec<u16> = s.encode_utf16().collect();
        let chars = self.new_primitive_array(primitives::C as i32, v.len());
        let mut arr: Rp<u16> = chars.data.into();
        for i in 0..v.len() {
            arr[i] = v[i];
        }

        o.set(0, chars.ptr() as u64);
        o
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

    pub fn array_class(&mut self, element_class: Rp<Class>) -> Rp<Class> {
        self.loader.load(&format!("[{}", element_class.desc))
    }

    pub fn new_multi_dim(&mut self, class: Rp<Class>, size: &[u64]) -> Rp<Object> {
        if class.dim == 1 {
            return self.new_array(class.element_class.name.as_str(), size[0] as usize);
        }

        let mut obj = arr!(class, u64, size[0] as usize);
        let next = self.loader.load(&class.desc[1..]);

        for i in 0..size[0] as usize {
            obj.set(i, self.new_multi_dim(next, &size[1..]).ptr() as u64)
        }
        obj
    }

    pub fn new_primitive_array(&self, id: i32, size: usize) -> Rp<Object> {
        let c = self.primitive_array[id as usize];
        return match id {
            // boolean
            0 => arr!(c, u8, size),
            // char
            1 => arr!(c, u16, size),
            // float
            2 => arr!(c, u32, size),
            // double
            3 => arr!(c, u64, size),
            // byte
            4 => arr!(c, u8, size),
            // short
            5 => arr!(c, u16, size),
            // int
            6 => arr!(c, u32, size),
            // long
            7 => arr!(c, u64, size),
            _ => panic!(),
        };
    }

    pub fn new_array(&mut self, element_class: &str, size: usize) -> Rp<Object> {
        let o = PRIMITIVES
            .iter()
            .position(|x| *x == element_class)
            .map(|x| x as i32)
            .unwrap_or(-1);
        let class = self.loader.load(element_class);
        let c = self.array_class(class);
        if o >= 0 {
            return self.new_primitive_array(o, size);
        }
        return arr!(c, u64, size);
    }
}

#[derive(Debug, Clone)]
pub struct SymRef {
    pub class: Rp<Class>,
    pub name: String,
    pub desc: String,
    pub member: Rp<ClassMember>,
}

#[cfg(test)]
mod test {
    use crate::heap::{desc::DescriptorParser, loader::ClassLoader, misc::Heap};

    #[test]
    fn loader_test() {
        let mut loader = Heap::new(".:test/rt.jar").unwrap().loader;
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
