use crate::attr::AttrInfo;
use crate::cp::{ClassFile, ConstantPool, MemberInfo};
use crate::heap::misc::{AccessFlags, SymRef};
use crate::rp::{Rp, Unmanaged};
use crate::runtime::vm::JThread;
use core::fmt::Debug;
use std::ops::Deref;

impl Unmanaged for Class {}

impl From<ClassFile> for Class {
    fn from(mut c: ClassFile) -> Self {
        let mut r = Class::default();
        r.access_flags = AccessFlags(c.access_flags);
        r.name = c.this_class().to_string();
        r.super_name = c.super_class().to_string();
        r.iface_names = c
            .interfaces_i
            .iter()
            .map(|&x| c.cp.class(x as usize).to_string())
            .collect();
        r.fields = c.fields.iter().map(|x| x.into()).collect();
        r.methods = c.methods.iter().map(|x| x.into()).collect();
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

impl Unmanaged for Object {}

// represents both object and primitive array
pub struct Object {
    pub class: Rp<Class>,
    pub data: usize,
}

impl Object {
    pub fn instance_of(&self, c: &Class) -> bool {
        c.is_assignable(&self.class)
    }

    pub fn set<T: Unmanaged>(&mut self, i: usize, val: T) {
        let mut rp: Rp<T> = Rp::from_ptr(self.data);
        rp[i] = val;
    }

    pub fn get<T: Unmanaged + Copy>(&self, i: usize) -> T {
        let rp: Rp<T> = Rp::from_ptr(self.data);
        rp[i]
    }
}

#[derive(Default)]
pub struct Class {
    pub access_flags: AccessFlags,
    pub name: String,
    pub super_name: String,
    pub iface_names: Vec<String>,
    pub cp: ConstantPool,
    pub fields: Vec<ClassMember>,
    pub methods: Vec<ClassMember>,

    pub super_class: Rp<Class>,
    pub interfaces: Vec<Rp<Class>>,

    pub static_fields: Vec<Rp<ClassMember>>,
    pub static_vars: Vec<u64>,

    pub ins_fields: Vec<Rp<ClassMember>>,

    // runtime loaded symbols
    pub sym_refs: Vec<Rp<SymRef>>,
    pub id: usize,
    pub initialized: bool,
    pub primitive: bool,
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
            .field("id", &self.id)
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
                return m.as_rp();
            }
        }
        return Rp::null();
    }

    pub fn lookup_method_in_class(&self, name: &str, desc: &str) -> Rp<ClassMember> {
        let mut cur: &Class = &self;

        // lookup in class
        loop {
            for m in cur.methods.iter() {
                if m.name == name && m.desc == desc {
                    return m.as_rp();
                }
            }
            if cur.super_class.is_null() {
                break;
            }
            cur = &cur.super_class;
        }

        Rp::null()
    }

    // run static block
    pub fn clinit(&mut self, th: &mut JThread) -> bool {
        if self.initialized {
            return false;
        }

        self.initialized = true;

        let init = self.clinit_method();

        if !init.is_null() {
            th.revert_pc();
            let fr = th.new_frame(init);
            th.stack.push_frame(fr);
        }

        // init super class
        if !self.super_class.is_null() {
            self.super_class.clinit(th);
        }
        true
    }

    fn clinit_method(&self) -> Rp<ClassMember> {
        self.methods
            .iter()
            .find(|&x| x.name == "<clinit>" && x.desc == "()V")
            .map(|x| x.as_rp())
            .unwrap_or(Rp::null())
    }

    pub fn lookup_method(&self, name: &str, desc: &str) -> Rp<ClassMember> {
        let m = self.lookup_method_in_class(name, desc);
        if !m.is_null() {
            return m;
        }
        // lookup in super interfaces
        Self::lookup_method_in_ifaces(&self.interfaces, name, desc)
    }

    // lookup in interfaces and parent interfaces
    pub fn lookup_iface_method(&self, name: &str, desc: &str) -> Rp<ClassMember> {
        for m in self.methods.iter() {
            if m.name == name && m.desc == desc {
                return m.as_rp();
            }
        }

        // lookup in super interfaces
        Self::lookup_method_in_ifaces(&self.interfaces, name, desc)
    }

    fn lookup_method_in_ifaces<T: Deref<Target = Class>>(
        ifaces: &[T],
        name: &str,
        desc: &str,
    ) -> Rp<ClassMember> {
        for i in ifaces.iter() {
            for m in i.methods.iter() {
                if m.name == name && m.desc == desc {
                    return m.as_rp();
                }
            }

            let m = Self::lookup_method_in_ifaces(&i.interfaces, name, desc);

            if !m.is_null() {
                return m;
            }
        }

        Rp::null()
    }

    pub fn lookup_field(&self, name: &str, desc: &str) -> Rp<ClassMember> {
        // lookup in this class fields
        for f in self.fields.iter() {
            if f.name == name && f.desc == desc {
                return f.as_rp();
            }
        }

        // lookup in implements
        for iface in self.interfaces.iter() {
            let f = iface.lookup_field(name, desc);

            if !f.is_null() {
                return f;
            }
        }

        if !self.super_class.is_null() {
            return self.super_class.lookup_field(name, desc);
        }

        Rp::null()
    }

    pub fn set_static(&mut self, i: usize, v: u64) {
        self.static_vars[i] = v;
    }

    pub fn set_instance(&self, obj: &mut Object, i: usize, v: u64) {
        obj.set(i, v);
    }

    pub fn get_static(&self, i: usize) -> u64 {
        self.static_vars[i]
    }

    pub fn get_instance(&self, obj: &Object, i: usize) -> u64 {
        obj.get(i)
    }

    pub fn is_assignable(&self, from: &Class) -> bool {
        if self.id == from.id {
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
            if sup.id == other.id {
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
                if i.id == iface.id || i.is_sub_iface(iface) {
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
            if i.id == iface.id || i.is_sub_iface(iface) {
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
                _ => {}
            }
        }
    }
}

impl Unmanaged for ClassMember {}
#[derive(Default)]
pub struct ClassMember {
    pub access_flags: AccessFlags,
    pub name: String,
    pub desc: String,
    pub max_stack: usize,
    pub max_locals: usize,
    pub code: Vec<u8>,
    pub cons_i: usize,
    pub id: usize,
    pub class: Rp<Class>,
    pub arg_cells: usize,
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
