use crate::heap::class::{Class, Object};
use crate::heap::loader::ClassLoader;
use crate::rp::Rp;
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

pub const PRIMITIVES_N_OFFSET: usize = 0;
pub const PRIMITIVES_N: usize = 8usize;
#[derive(Debug)]
pub struct Heap {
    pub loader: ClassLoader,
    // primitive types
    pub z: Rp<Class>,
    pub c: Rp<Class>,
    pub f: Rp<Class>,
    pub d: Rp<Class>,
    pub b: Rp<Class>,
    pub s: Rp<Class>,
    pub i: Rp<Class>,
    pub j: Rp<Class>,
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

macro_rules! asf {
    ($h: ident, $f: ident, $n: expr, $d: expr) => {{
        let mut c = Class::default();
        c.name = $n.to_string();
        c.desc = $d.to_string();
        c.primitive = true;
        let p = $h.loader.insert(c, $d);
        $h.$f = p;
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
            data: Rp::<$t>::alloc($sz).ptr(),
            size: $sz,
        };
        Rp::new(o)
    }};
}

impl Heap {
    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let l = ClassLoader::new(cp)?;
        // int long char short boolean byte double float

        let mut h = Heap {
            loader: l,
            i: Rp::null(),
            c: Rp::null(),
            z: Rp::null(),
            j: Rp::null(),
            b: Rp::null(),
            d: Rp::null(),
            s: Rp::null(),
            f: Rp::null(),
        };
        // id 0-7 is primitive types
        asf!(h, z, "boolean", "Z");
        asf!(h, c, "char", "C");
        asf!(h, f, "float", "F");
        asf!(h, d, "double", "D");
        asf!(h, b, "byte", "B");
        asf!(h, s, "short", "S");
        asf!(h, i, "int", "I");
        asf!(h, j, "long", "J");

        // id 8~ is primitive array types
        for i in 0..PRIMITIVES_N {
            let n = format!("[{}", h.loader.get(i).desc);
            h.loader.load(&n);
        }
        Ok(h)
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
        let v: Rp<u64> = Rp::alloc(class.ins_fields.len());
        let obj = Object {
            class: class,
            size: class.ins_fields.len(),
            data: v.ptr(),
        };

        Rp::new(obj)
    }

    pub fn array_class(&mut self, element_class: Rp<Class>) -> Rp<Class> {
        self.loader.load(&format!("[{}", element_class.desc))
    }

    pub fn new_multi_dim(&mut self, class: Rp<Class>, size: &[u64]) -> Rp<Object> {
        if class.dim == 1 {
            return self.new_array(class.element_class.id, size[0] as usize);
        }

        let mut obj = arr!(class, u64, size[0] as usize);
        let next = self.loader.load(&class.desc[1..]);

        for i in 0..size[0] as usize {
            obj.set(i, self.new_multi_dim(next, &size[1..]).ptr() as u64)
        }
        obj
    }

    pub fn new_array(&mut self, element_id: usize, size: usize) -> Rp<Object> {
        if element_id < PRIMITIVES_N + PRIMITIVES_N_OFFSET && element_id >= PRIMITIVES_N_OFFSET {
            let c = self
                .loader
                .get(element_id + PRIMITIVES_N + PRIMITIVES_N_OFFSET);

            return match element_id {
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
                _ => panic!("new array failed invalid id {}", element_id),
            };
        }

        let element_class = self.loader.get(element_id);
        let a_class = self.array_class(element_class);

        return arr!(a_class, u64, size);
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
    use crate::heap::{desc::DescriptorParser, loader::ClassLoader};

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
