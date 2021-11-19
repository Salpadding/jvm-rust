use crate::cp::ClassFile;
use crate::entry;
use crate::entry::Entry;
use crate::heap::class::Class;
use crate::heap::desc::DescriptorParser;
use crate::rp::{Rp};
use crate::StringErr;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ClassLoader {
    entry: Box<dyn Entry>,
    pub loaded: BTreeMap<String, Rp<Class>>,
    classes: Vec<Rp<Class>>,
}

impl ClassLoader {
    pub fn insert(&mut self, class: Class, alias: &str) -> Rp<Class> {
        let class_id = self.classes.len();
        self.classes.push(Rp::new(class));
        let p = self.classes[class_id];
        p.get_mut().id = class_id;
        self.loaded.insert(p.name.to_string(), p);

        if !alias.is_empty() {
            self.loaded.insert(alias.to_string(), p);
        }
        p
    }

    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let entry = entry::new_entry(cp)?;

        Ok(ClassLoader {
            entry,
            loaded: BTreeMap::new(),
            classes: Vec::new(),
        })
    }

    pub fn get(&self, i: usize) -> Rp<Class> {
        self.classes[i].as_rp()
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
            c.initialized = true;
            c.super_class = self.load("java/lang/Object");
            c.name = name.to_string();
            c.desc = name.to_string();
            c.dim = dim;
            c.element_class = self.load(&el);

            return self.insert(c, "");
        }

        let bytes = self.entry.read_class(name).unwrap();
        self.define(name, bytes)
    }

    fn define(&mut self, name: &str, bytes: Vec<u8>) -> Rp<Class> {
        let file = ClassFile::new(bytes);
        let mut cl: Class = file.into();
        for m in cl.methods.iter_mut() {
            let mut parser = DescriptorParser::new(m.desc.as_bytes());
            m.arg_cells = parser.parse_method().arg_cells;
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
            .map(|x| x.as_rp())
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
            cl.ins_fields.push(f.as_rp());
            i += 1;
        }

        let class_id = self.classes.len();
        self.classes.push(Rp::new(cl));

        // link members to class
        let mut p = self.classes[class_id];
        p.get_mut().id = class_id;

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
