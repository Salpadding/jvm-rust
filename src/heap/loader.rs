use crate::heap::class::Class;
use crate::heap::desc::DescriptorParser;
use cp::ClassFile;
use cp::Entry;
use err::StringErr;
use rp::Rp;
use std::collections::BTreeMap;

use super::class::ClassMember;
use super::misc::{flags, AccessFlags, PRIMITIVES, PRIMITIVE_DESC, PRIMITIVE_N};
use crate::heap::misc::Heap;

pub struct ClassLoader {
    entry: Box<dyn Entry>,
    loaded: BTreeMap<String, Rp<Class>>,
    classes: Vec<Rp<Class>>,
    jclass: Rp<Class>,
    jstring: Rp<Class>,
    heap: Rp<Heap>,
}

impl ClassLoader {
    // insert a primitive class
    fn insert(&mut self, class: Class, alias: &str) -> Rp<Class> {
        let class_id = self.classes.len();
        self.classes.push(Rp::new(class));
        let p = self.classes[class_id];
        p.get_mut().id = class_id;
        self.loaded.insert(p.name.to_string(), p);

        if !alias.is_empty() {
            self.loaded.insert(alias.to_string(), p);
        }

        self.assign_j_class(p);
        p
    }

    // assign class object to class
    fn assign_j_class(&self, c: Rp<Class>) {
        if !c.j_class.is_null() {
            return;
        }

        // create a class object of size field + 1
        // we use the last field to store class pointer
        let mut o = Class::new_obj_size(self.jclass, self.jclass.ins_fields.len() + 1);

        o.set(self.jclass.ins_fields.len(), c.ptr());
        c.get_mut().j_class = o;
    }

    pub fn new(cp: &str, heap: Rp<Heap>) -> Result<Rp<Self>, StringErr> {
        let entry = cp::new_entry(cp)?;

        let mut cl = Rp::new(ClassLoader {
            entry,
            loaded: BTreeMap::new(),
            classes: Vec::new(),
            jclass: Rp::null(),
            jstring: Rp::null(),
            heap,
        });

        heap.get_mut().loader = cl;
        cl.init();

        Ok(cl)
    }

    fn init(&mut self) {
        if !self.jclass.is_null() {
            return;
        }

        self.jstring = self.load("java/lang/String");
        self.heap.jstring = self.jstring;
        self.jclass = self.load("java/lang/Class");

        for c in self.classes.iter() {
            self.assign_j_class(*c);
        }

        // load primitives, primitives has no super class
        for i in 0..PRIMITIVE_N {
            let mut c = Class::default();
            c.heap = self.heap;
            c.access_flags = AccessFlags(flags::ACC_PUBLIC);
            c.name = PRIMITIVES[i].to_string();
            c.desc = PRIMITIVE_DESC[i].to_string();
            c.initialized = true;
            self.insert(c, PRIMITIVE_DESC[i]);
        }
    }

    pub fn load(&mut self, name: &str) -> Rp<Class> {
        match self.loaded.get(name) {
            Some(cl) => return *cl,
            _ => {}
        };

        // array type
        if name.starts_with("[") {
            let mut parser = DescriptorParser::new(name.as_bytes());
            let (dim, _, el) = parser.parse_arr();

            let mut c = Class::default();
            c.heap = self.heap;
            c.initialized = true;
            c.access_flags = AccessFlags(flags::ACC_PUBLIC);
            c.super_class = self.load("java/lang/Object");
            c.name = name.to_string();
            c.desc = name.to_string();
            c.dim = dim;
            c.element_class = self.load(&el.class());

            return self.insert(c, "");
        }

        let bytes = self.entry.read_class(name).unwrap();
        self.define(name, bytes)
    }

    fn inject_native(&self, m: &mut ClassMember) {
        m.max_locals = if m.access_flags.is_static() {
            m.m_desc.arg_slots
        } else {
            m.m_desc.arg_slots + 1
        };

        use crate::heap::desc::JType;
        match &m.m_desc.ret {
            // return
            JType::V => m.code = [0xfe, 0xb1].to_vec(),
            // lreturn
            JType::DJ(_) => {
                m.code = [0xfe, 0xad].to_vec();
                m.max_stack = 2
            }
            // areturn
            JType::A(_) => {
                m.code = [0xfe, 0xb0].to_vec();
                m.max_stack = 1
            }
            // C Z B I F => ireturn
            _ => {
                m.code = [0xfe, 0xac].to_vec();
                m.max_stack = 1;
            }
        }
    }

    fn define(&mut self, name: &str, bytes: Vec<u8>) -> Rp<Class> {
        let file = ClassFile::new(bytes);
        let mut cl: Class = file.into();
        cl.heap = self.heap;
        for m in cl.methods.iter_mut() {
            let mut parser = DescriptorParser::new(m.desc.as_bytes());
            m.m_desc = parser.parse_method();

            if m.access_flags.is_native() {
                self.inject_native(m);
            }
        }

        // load super and interfaces
        if &cl.super_name != "" {
            cl.super_class = self.load(&cl.super_name);
        }

        cl.interfaces = cl.iface_names.iter().map(|x| self.load(x)).collect();

        cl.static_fields = cl
            .fields
            .iter_mut()
            .filter(|x| x.access_flags.is_static())
            .map(|x| x.into())
            .collect();

        // set field index
        for i in 0..cl.static_fields.len() {
            cl.static_fields[i].id = i;
        }

        cl.static_vars = vec![0u64; cl.static_fields.len()];
        cl.init_finals();

        // init instance fields
        let base = if cl.super_class.is_null() {
            0
        } else {
            cl.super_class.ins_fields.len()
        };

        for i in 0..base {
            cl.ins_fields.push(cl.super_class.ins_fields[i]);
        }

        let mut i = base;
        for f in cl.fields.iter_mut().filter(|x| !x.access_flags.is_static()) {
            f.id = i;
            cl.ins_fields.push(f.into());
            i += 1;
        }

        let class_id = self.classes.len();
        self.classes.push(Rp::new(cl));

        // link members to class
        let mut p = self.classes[class_id];
        p.get_mut().id = class_id;

        // create class object
        if !self.jclass.is_null() {
            self.assign_j_class(p);
        }

        let n = p;
        for f in p.fields.iter_mut() {
            f.class = n;
        }

        for m in p.methods.iter_mut() {
            m.class = n;
        }
        self.loaded.insert(name.to_string(), n);
        p
    }
}
