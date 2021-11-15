use crate::attr::AttrInfo;
use crate::cp::{ClassFile, ConstantPool, MemberInfo};
use crate::entry::Entry;
use crate::{entry, StringErr};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

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
        r.fields = c.fields.iter().map(|x| Rc::new(x.into())).collect();
        r.methods = c.methods.iter().map(|x| Rc::new(x.into())).collect();
        r.sym_refs = vec![None; c.cp.infos.len()];

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
    pub class: Rc<RefCell<Class>>,
    pub fields: Vec<u64>,
}

impl Object {
    pub fn forget(o: Box<Object>) -> u64 {
        let p = std::boxed::Box::leak(o);
        p as *mut Object as usize as u64
    }

    pub fn from_ptr(p: u64) -> Box<Object> {
        let p = p as usize as *mut Object;
        unsafe { Box::from_raw(p) }
    }
}

#[derive(Debug, Default)]
pub struct Class {
    pub access_flags: AccessFlags,
    pub name: String,
    pub super_name: String,
    pub iface_names: Vec<String>,
    pub cp: ConstantPool,
    pub fields: Vec<Rc<ClassMember>>,
    pub methods: Vec<Rc<ClassMember>>,

    pub super_class: Option<Rc<RefCell<Class>>>,
    pub interfaces: Vec<Rc<RefCell<Class>>>,

    pub static_fields: Vec<Rc<ClassMember>>,
    pub static_vars: Vec<u64>,

    pub ins_fields: Vec<Rc<ClassMember>>,

    // runtime loaded symbols
    pub sym_refs: Vec<Option<Rc<SymRef>>>,
}

impl Class {
    pub fn main_method(&self) -> Option<Rc<ClassMember>> {
        for m in self.methods.iter() {
            if &m.name == "main"
                && &m.desc == "([Ljava/lang/String;)V"
                && m.access_flags.is_static()
            {
                return Some(m.clone());
            }
        }
        None
    }

    fn count_class_vars(&self) -> usize {
        self.fields
            .iter()
            .filter(|x| x.access_flags.is_static())
            .count()
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
        let base = match self.super_class {
            None => 0,
            Some(ref c) => c.borrow().count_ins_fields(),
        };
        base + self
            .fields
            .iter()
            .filter(|x| !x.access_flags.is_static())
            .count()
    }

    fn get_ins_field(&self, i: usize) -> Rc<ClassMember> {
        self.ins_fields[i].clone()
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
    loaded: BTreeMap<String, Rc<RefCell<Class>>>,
}

impl ClassLoader {
    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let entry = entry::new_entry(cp)?;

        Ok(ClassLoader {
            entry,
            loaded: BTreeMap::new(),
        })
    }

    pub fn load(&mut self, name: &str) -> Rc<RefCell<Class>> {
        match self.loaded.get(name) {
            Some(cl) => return cl.clone(),
            _ => {}
        };

        let bytes = self.entry.read_class(name).unwrap();
        self.define(name, bytes)
    }

    fn define(&mut self, name: &str, bytes: Vec<u8>) -> Rc<RefCell<Class>> {
        let file = ClassFile::new(bytes);
        let mut cl: Class = file.into();

        // load super and interfaces
        if &cl.super_name != "" {
            cl.super_class = Some(self.load(&cl.super_name));
        }

        let mut ifaces: Vec<Rc<RefCell<Class>>> = Vec::with_capacity(cl.iface_names.len());
        for n in cl.iface_names.iter() {
            ifaces.push(self.load(n));
        }
        cl.interfaces = ifaces;
        cl.static_fields = cl
            .fields
            .iter()
            .filter(|x| x.access_flags.is_static())
            .map(|x| x.clone())
            .collect();
        cl.static_vars = vec![0u64; cl.static_fields.len()];
        cl.init_finals();

        // init instance fields
        let base = match cl.super_class {
            None => 0,
            Some(ref c) => c.borrow().count_ins_fields(),
        };

        for i in 0..base {
            cl.ins_fields
                .push(cl.super_class.as_ref().unwrap().borrow().get_ins_field(i));
        }

        for f in cl.fields.iter().filter(|x| !x.access_flags.is_static()) {
            cl.ins_fields.push(f.clone());
        }

        let arc = Rc::new(RefCell::new(cl));
        let cloned = arc.clone();
        self.loaded.insert(name.to_string(), arc);
        cloned
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

    pub fn class_ref(&mut self, cur: &mut Class, i: usize) -> Rc<SymRef> {
        match cur.sym_refs[i] {
            Some(_) => {
                return cur.sym_refs[i].as_ref().unwrap().clone();
            }
            _ => {}
        };

        let name = cur.cp.class(i);
        let class = self.loader.load(name);
        let sym = SymRef {
            class,
            name: name.to_string(),
            desc: "".to_string(),
            field_i: 0,
        };

        cur.sym_refs[i] = Some(Rc::new(sym));
        cur.sym_refs[i].as_ref().unwrap().clone()
    }

    pub fn field_ref(&mut self, cur: &mut Class, i: usize) -> Rc<SymRef> {
        match cur.sym_refs[i] {
            Some(_) => {
                return cur.sym_refs[i].as_ref().unwrap().clone();
            }
            _ => {}
        };

        let (class_name, name, desc) = cur.cp.field_ref(i);
        let class = self.loader.load(class_name);
        let mut sym = SymRef {
            class: class.clone(),
            name: name.to_string(),
            desc: desc.to_string(),
            field_i: 0,
        };

        if &cur.name == class_name {
            sym.field_i = cur.field_index(name);
        } else {
            sym.field_i = class.borrow().field_index(name);
        }

        cur.sym_refs[i] = Some(Rc::new(sym));
        cur.sym_refs[i].as_ref().unwrap().clone()
    }

    pub fn method_ref(&mut self, cur: &mut Class, i: usize) -> Rc<SymRef> {
        todo!()
    }

    pub fn iface_ref(&mut self, cur: &mut Class, i: usize) -> Rc<SymRef> {
        todo!()
    }

    pub fn new_obj(&self, class: Rc<RefCell<Class>>) -> Box<Object> {
        let obj = Object {
            class: class.clone(),
            fields: vec![0u64; class.borrow().ins_fields.len()],
        };

        let obj = Box::new(obj);
        obj
    }
}

#[derive(Debug, Clone)]
pub struct SymRef {
    pub class: Rc<RefCell<Class>>,
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
