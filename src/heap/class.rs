use crate::attr::AttrInfo;
use crate::cp::{ClassFile, ConstantPool, MemberInfo};
use crate::heap::misc::{AccessFlags, SymRef};
use crate::rp::{Np, Rp};
use core::fmt::Debug;

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
        r.id = -1;
        r.name = m.name.to_string();
        r.access_flags = AccessFlags(m.access_flags);
        r.desc = m.desc.to_string();

        for attr in m.attrs.iter() {
            match attr {
                &AttrInfo::Code(ref c) => {
                    r.code = c.code.to_vec();
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

#[derive(Default)]
pub struct Class {
    pub access_flags: AccessFlags,
    pub name: String,
    pub super_name: String,
    pub iface_names: Vec<String>,
    pub cp: ConstantPool,
    pub fields: Vec<Rp<ClassMember>>,
    pub methods: Vec<Rp<ClassMember>>,

    pub super_class: Np<Class>,
    pub interfaces: Vec<Rp<Class>>,

    pub static_fields: Vec<Rp<ClassMember>>,
    pub static_vars: Vec<u64>,

    pub ins_fields: Vec<Rp<ClassMember>>,

    // runtime loaded symbols
    pub sym_refs: Vec<Np<SymRef>>,
}

impl Debug for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Class")
            .field("access_flags", &self.access_flags)
            .field("name", &self.name)
            .field("super_name", &self.super_name)
            .field("iface_names", &self.iface_names)
            .field("fields", &self.fields)
            .field("methods", &self.methods)
            .field("static_fields", &self.static_fields)
            .field("static_vars", &self.static_vars)
            .field("ins_fields", &self.ins_fields)
            .field("sym_refs", &self.sym_refs)
            .finish()
    }
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

    pub fn field_index(&self, name: &str, desc: &str) -> usize {
        let p = self
            .static_fields
            .iter()
            .rev()
            .position(|x| x.name == name && x.desc == desc);
        if p.is_some() {
            return p.unwrap();
        }
        self.ins_fields
            .iter()
            .rev()
            .position(|x| x.name == name && x.desc == desc)
            .unwrap()
    }

    pub fn count_ins_fields(&self) -> usize {
        let base = if self.super_class.is_null() {
            0
        } else {
            self.super_class.count_ins_fields()
        };
        base + self
            .fields
            .iter()
            .filter(|x| !x.access_flags.is_static())
            .count()
    }

    pub fn get_ins_field(&self, i: usize) -> Rp<ClassMember> {
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
        let mut sup = self.super_class;

        while !sup.is_null() {
            if sup.name == other.name {
                return true;
            }
            sup = sup.super_class;
        }
        false
    }

    pub fn is_impl(&self, iface: &Class) -> bool {
        let mut cur: &Class = self;

        loop {
            for i in self.interfaces.iter() {
                if i.name == iface.name || i.is_sub_iface(iface) {
                    return true;
                }
            }

            if cur.super_class.is_null() {
                return false;
            }

            cur = &cur.super_class;
        }
    }

    pub fn is_sub_iface(&self, iface: &Class) -> bool {
        for i in self.interfaces.iter() {
            if i.name == iface.name || i.is_sub_iface(iface) {
                return true;
            }
        }
        false
    }

    pub fn init_finals(&mut self) {
        for i in 0..self.static_fields.len() {
            let f = self.static_fields[i];

            if !f.access_flags.is_final() {
                continue;
            }

            match f.desc.as_str() {
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

#[derive(Default)]
pub struct ClassMember {
    pub access_flags: AccessFlags,
    pub name: String,
    pub desc: String,
    pub max_stack: usize,
    pub max_locals: usize,
    pub code: Vec<u8>,
    pub cons_i: usize,
    pub id: i32,
    pub class: Rp<Class>,
}

impl Debug for ClassMember {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassMember")
            .field("access_flags", &self.access_flags)
            .field("name", &self.name)
            .field("desc", &self.desc)
            .field("max_stack", &self.max_stack)
            .field("max_locals", &self.max_locals)
            .field("cons_i", &self.cons_i)
            .field("id", &self.id)
            .finish()
    }
}
