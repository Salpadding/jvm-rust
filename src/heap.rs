use crate::attr::AttrInfo;
use crate::cp::{ClassFile, ConstantPool, MemberInfo};
use crate::entry::Entry;
use crate::{entry, StringErr};
use std::collections::BTreeMap;
use crate::rp::Rp;

impl From<ClassFile> for Class {
    fn from(mut c: ClassFile) -> Self {
        let mut r = Class::default();
        r.access_flags = AccessFlags(c.access_flags);
        r.name = c.this_class().to_string();
        r.super_name = c.super_class().to_string();
        r.iface_names = c
            .interfaces_i
            .iter()
            .map(|x| c.cp.class(*x as usize).to_string())
            .collect();
        r.fields = c.fields.iter().map(|x| Rp::new(x.into())).collect();
        r.methods = c.methods.iter().map(|x| Rp::new(x.into())).collect();
        r.sym_refs = vec![Rp::null(); c.cp.infos.len()];

        core::mem::swap(&mut c.cp, &mut r.cp);
        r
    }
}

impl From<&MemberInfo> for ClassMember {
    fn from(m: &MemberInfo) -> Self {
        let mut r = ClassMember::default();
        r.name = m.name.to_string();
        r.access_flags = AccessFlags(m.access_flags);
        r.desc = m.desc.to_string();

        for attr in m.attrs.iter() {
            match attr {
                &AttrInfo::Code(ref c) => {
                    r.code = c.code.clone();
                    r.max_stack = c.max_stack as usize;
                    r.max_locals = c.max_locals as usize;
                }
                &AttrInfo::ConstantValue(i) => {
                    r.cons_i = i as usize;
                }
                _ => {}
            }
        }
        r
    }
}

pub struct Object {
    pub class: Rp<Class>,
    pub fields: Vec<u64>,
}

impl Object {
    pub fn instance_of(&self, c: &Class) -> bool {
        c.is_assignable(&self.class)
    }
}

#[derive(Debug, Default)]
pub struct Class {
    pub access_flags: AccessFlags,
    pub name: String,
    pub super_name: String,
    pub iface_names: Vec<String>,
    pub cp: ConstantPool,
    pub fields: Vec<Rp<ClassMember>>,
    pub methods: Vec<Rp<ClassMember>>,

    pub super_class: Rp<Class>, 
    pub interfaces: Vec<Rp<Class>>,

    pub static_fields: Vec<Rp<ClassMember>>,
    pub static_vars: Vec<u64>,

    pub ins_fields: Vec<Rp<ClassMember>>,

    // runtime loaded symbols
    pub sym_refs: Vec<Rp<SymRef>>,
}

impl Class {
    pub fn main_method(&self) -> Rp<ClassMember> {
        for m in self.methods.iter() {
            if &m.name == "main"
                && &m.desc == "([Ljava/lang/String;)V"
                && m.access_flags.is_static()
            {
                return *m;
            }
        }
        return Rp::null();
    }

    fn field_index(&self, f: &str) -> usize {
        let p = self
            .static_fields
            .iter()
            .map(|x| &x.name)
            .position(|x| x == f);
        if p.is_some() {
            return p.unwrap();
        }
        self.ins_fields
            .iter()
            .map(|x| &x.name)
            .position(|x| x == f)
            .unwrap()
    }

    fn count_ins_fields(&self) -> usize {
        let base = if self.super_class.is_null()  { 0 } else { self.super_class.count_ins_fields() };
        base + self
            .fields
            .iter()
            .filter(|x| !x.access_flags.is_static())
            .count()
    }

    fn get_ins_field(&self, i: usize) -> Rp<ClassMember> {
        self.ins_fields[i]
    }

    pub fn set_static(&mut self, i: usize, v: u64) {
        self.static_vars[i] = v;
        println!(
            "set field {} of class {}",
            self.static_fields[i].name, self.name
        );
        println!(
            "static vars of class {} = {:?}",
            self.name, self.static_vars
        );
    }

    pub fn set_instance(&self, obj: &mut Object, i: usize, v: u64) {
        obj.fields[i] = v;
        println!(
            "set field {} of class {}",
            self.ins_fields[i].name, self.name
        );
        println!("instance vars of class {} = {:?}", self.name, obj.fields);
    }

    pub fn get_static(&self, i: usize) -> u64 {
        self.static_vars[i]
    }

    pub fn get_instance(&self, obj: &Object, i: usize) -> u64 {
        obj.fields[i]
    }

    pub fn is_assignable(&self, from: &Class) -> bool {
        if self.name == from.name {
            return true;
        }

        if self.access_flags.is_iface() {
            from.is_impl(self)
        } else {
            from.is_sub_class(self)
        }
    }

    pub fn is_sub_class(&self, other: &Class) -> bool {
        let mut sup = self.super_class.clone();

        while !sup.is_null() {
            if sup.name == other.name {
                return true;
            }
            sup = sup.super_class;
        }
        false
    }

    pub fn is_impl(&self, iface: &Class) -> bool {
        for i in self.interfaces.iter() {
            if i.name == iface.name || i.is_sub_iface(iface) {
                return true;
            }
        }

        if self.super_class.is_null() {
            return false;
        }

        return self.super_class.is_impl(iface);
    }

    pub fn is_sub_iface(&self, iface: &Class) -> bool {
        for i in self.interfaces.iter() {
            if i.name == iface.name || i.is_sub_iface(iface) {
                return true;
            }
        }
        false
    }

    fn init_finals(&mut self) {
        for i in 0..self.static_fields.len() {
            let f = &self.static_fields[i];

            if !f.access_flags.is_final() {
                continue;
            }

            match &*f.desc {
                "Z" | "B" | "C" | "S" | "I" => {
                    self.static_vars[i] = self.cp.u32(f.cons_i) as u64;
                }
                "J" => {
                    self.static_vars[i] = self.cp.u64(f.cons_i);
                }
                "F" => self.static_vars[i] = self.cp.f32(f.cons_i).to_bits() as u64,
                "D" => self.static_vars[i] = self.cp.f64(f.cons_i).to_bits(),
                _ => panic!("invalid final type {}", &f.desc),
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ClassMember {
    pub access_flags: AccessFlags,
    pub name: String,
    pub desc: String,
    pub max_stack: usize,
    pub max_locals: usize,
    pub code: Vec<u8>,
    pub cons_i: usize,
}

#[derive(Debug, Default)]
pub struct AccessFlags(u16);

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
pub struct ClassLoader {
    entry: Box<dyn Entry>,
    loaded: BTreeMap<String, Rp<Class>>
}

impl ClassLoader {
    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let entry = entry::new_entry(cp)?;

        Ok(ClassLoader {
            entry,
            loaded: BTreeMap::new(),
        })
    }

    pub fn load(&mut self, name: &str) -> Rp<Class> {
        match self.loaded.get(name) {
            Some(cl) => return *cl,
            _ => {}
        };

        let bytes = self.entry.read_class(name).unwrap();
        self.define(name, bytes)
    }

    fn define(&mut self, name: &str, bytes: Vec<u8>) -> Rp<Class> {
        let file = ClassFile::new(bytes);
        let mut cl: Class = file.into();

        // load super and interfaces
        if &cl.super_name != "" {
            cl.super_class = self.load(&cl.super_name);
        }

        cl.interfaces = cl.iface_names.iter().map(|x| self.load(x)).collect();

        cl.static_fields = cl
            .fields
            .iter()
            .filter(|x| x.access_flags.is_static())
            .map(|x| *x)
            .collect();
        cl.static_vars = vec![0u64; cl.static_fields.len()];
        cl.init_finals();

        // init instance fields
        let base = if cl.super_class.is_null() { 0 } else { cl.super_class.count_ins_fields() };

        for i in 0..base {
            cl.ins_fields
                .push(cl.super_class.get_ins_field(i));
        }

        for f in cl.fields.iter().filter(|x| !x.access_flags.is_static()) {
            cl.ins_fields.push(f.clone());
        }

        let p = Rp::new(cl);
        self.loaded.insert(name.to_string(), p);
        p
    }
}

#[derive(Debug)]
pub struct Heap {
    pub loader: ClassLoader,
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
            field_i: 0,
        };

        *r = Rp::new(sym);
        *r
    }

    pub fn field_ref(&mut self, cur: &mut Class, i: usize) -> Rp<SymRef> {
        let r = &mut cur.sym_refs[i];

        if !r.is_null() {
            return *r;
        }

        let (class_name, name, desc) = cur.cp.field_ref(i);
        let class = self.loader.load(class_name);
        let mut sym = SymRef {
            class: class.clone(),
            name: name.to_string(),
            desc: desc.to_string(),
            field_i: 0,
        };

        sym.field_i = class.field_index(name);

        *r = Rp::new(sym);
        *r
    }

    pub fn method_ref(&mut self, cur: &mut Class, i: usize) -> Rp<SymRef> {
        todo!()
    }

    pub fn iface_ref(&mut self, cur: &mut Class, i: usize) -> Rp<SymRef> {
        todo!()
    }

    pub fn new_obj(&self, class: Rp<Class>) -> Rp<Object> {
        let obj = Object {
            class: class,
            fields: vec![0u64; class.ins_fields.len()],
        };

        Rp::new(obj)
    }
}

#[derive(Debug, Clone)]
pub struct SymRef {
    pub class: Rp<Class>,
    pub name: String,
    pub desc: String,
    pub field_i: usize,
}

#[cfg(test)]
mod test {
    use super::{AccessFlags, ClassLoader};

    #[test]
    fn loader_test() {
        let mut loader = ClassLoader::new(".:test/rt.jar").unwrap();
        let class = loader.load("test/Test");
    }
}
