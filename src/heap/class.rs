use crate::heap::misc::{AccessFlags, SymRef};
use crate::runtime::vm::JThread;
use cp::AttrInfo;
use cp::{ClassFile, ConstantPool, MemberInfo};
use rp::Rp;
use std::fmt::Debug;
use std::ops::Deref;

use super::desc::MethodDescriptor;
use crate::heap::misc::Heap;

impl From<ClassFile> for Class {
    fn from(mut c: ClassFile) -> Self {
        let mut r = Class::default();
        r.access_flags = AccessFlags(c.access_flags);
        r.name = c.this_class().to_string();
        r.desc = format!("L{};", r.name);
        r.super_name = c.super_class().to_string();
        r.iface_names = c
            .interfaces_i
            .iter()
            .map(|&x| c.cp.class(x as usize).to_string())
            .collect();
        r.fields = c.fields.iter_mut().map(|x| x.into()).collect();
        r.methods = c.methods.iter_mut().map(|x| x.into()).collect();
        r.sym_refs = vec![Rp::null(); c.cp.len()];

        core::mem::swap(&mut c.cp, &mut r.cp);
        r
    }
}

impl From<&mut MemberInfo> for ClassMember {
    fn from(m: &mut MemberInfo) -> Self {
        let mut r = ClassMember::default();
        r.name = m.name.to_string();
        r.access_flags = AccessFlags(m.access_flags);
        r.desc = m.desc.to_string();

        for attr in m.attrs.iter_mut() {
            match attr {
                &mut AttrInfo::Code(ref mut c) => {
                    std::mem::swap(&mut r.code, &mut c.code);
                    r.max_stack = c.max_stack as usize;
                    r.max_locals = c.max_locals as usize;
                }
                &mut AttrInfo::ConstantValue(i) => {
                    r.cons_i = i as usize;
                }
                _ => {}
            }
        }
        r
    }
}

// represents both object and primitive array
pub struct Object {
    pub class: Rp<Class>,
    // object size/array length
    pub size: usize,
    // heap data
    pub data: usize,
}

impl Object {
    pub fn fields(&self) -> &mut [u64] {
        let p: Rp<u64> = self.data.into();
        p.as_slice(self.size)
    }

    pub fn extra_class(&self) -> Rp<Class> {
        let p: u64 = self.fields()[self.size - 1];
        (p as usize).into()
    }

    pub fn extra_member(&self) -> Rp<ClassMember> {
        let p: u64 = self.fields()[self.size - 1];
        (p as usize).into()
    }

    pub fn instance_of(&self, c: &Class) -> bool {
        c.is_assignable(&self.class)
    }

    pub fn set_field_ref(&mut self, field: &str, r: Rp<Object>) {
        self.set_field(field, r.ptr() as u64);
    }

    pub fn set_field(&mut self, field: &str, v: u64) {
        for i in (0..self.class.ins_fields.len()).rev() {
            if self.class.ins_fields[i].name == field {
                self.set(i, v);
                return;
            }
        }
    }

    pub fn get_field(&self, field: &str) -> u64 {
        for i in (0..self.class.ins_fields.len()).rev() {
            if self.class.ins_fields[i].name == field {
                return self.fields()[i];
            }
        }
        0
    }

    pub fn jarray<T: 'static>(&self) -> &mut [T] {
        let p: Rp<T> = self.data.into();
        p.as_slice(self.size)
    }

    pub fn jstring(&self) -> String {
        let arr: Rp<Object> = (self.fields()[0] as usize).into();
        let data: &[u16] = arr.jarray();
        String::from_utf16(data).unwrap()
    }

    pub fn set<T>(&mut self, i: usize, val: T) {
        let mut rp: Rp<T> = self.data.into();
        rp[i] = val;
    }

    pub fn get<T: Copy>(&self, i: usize) -> T {
        let rp: Rp<T> = self.data.into();
        rp[i]
    }
}

#[derive(Default)]
pub struct Class {
    pub heap: Rp<Heap>,
    pub access_flags: AccessFlags,
    pub name: String,
    pub desc: String,
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

    // for array type, this is class of element
    pub element_class: Rp<Class>,
    // dimension of array
    pub dim: u8,

    // class object
    pub j_class: Rp<Object>,
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
                return m.into();
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
                    return m.into();
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
            let fr = th.new_frame(init);
            th.push_frame(fr);
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
            .map(|x| x.into())
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
                return m.into();
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
                    return m.into();
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
                return f.into();
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

            if !f.access_flags.is_final() || f.cons_i == 0 {
                continue;
            }

            let (c, b) = match self.cp.constant(f.cons_i) {
                cp::Constant::Primitive(i, _) => (i, true),
                cp::Constant::String(s) => (self.heap.new_jstr(s).ptr() as u64, true),
                _ => (0, false),
            };

            if b {
                self.static_vars[i] = c;
                return;
            }

            if f.cons_i != 0 {
                println!("need con {:?}", self.cp.constant(f.cons_i))
            }
        }
    }

    pub fn new_obj_size(class: Rp<Class>, size: usize) -> Rp<Object> {
        let v: Rp<u64> = Rp::new_a(size);
        let obj = Object {
            class: class,
            size: size,
            data: v.ptr(),
        };

        Rp::new(obj)
    }

    pub fn new_obj(class: Rp<Class>) -> Rp<Object> {
        Self::new_obj_size(class, class.ins_fields.len())
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
    pub id: usize,
    pub class: Rp<Class>,
    pub m_desc: MethodDescriptor,
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
