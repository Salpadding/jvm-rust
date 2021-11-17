use crate::cp::ClassFile;
use crate::entry;
use crate::entry::Entry;
use crate::heap::class::Class;
use crate::heap::misc::DescriptorParser;
use crate::rp::{Rp, Unmanged};
use crate::StringErr;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ClassLoader {
    entry: Box<dyn Entry>,
    loaded: BTreeMap<String, Rp<Class>>,
    classes: Vec<Class>,
}

impl ClassLoader {
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
            cl.static_fields[i].id = i as i32;
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
            f.id = i as i32;
            cl.ins_fields.push(f.as_rp());
            i += 1;
        }

        let id = self.classes.len();
        self.classes.push(cl);

        // link members to class
        let mut p = self.classes[id].as_rp();
        p.id = id;

        let n = p;
        for f in p.fields.iter_mut() {
            f.class = n;
        }

        for m in p.methods.iter_mut() {
            m.class = n;
        }
        self.loaded.insert(name.to_string(), p);
        p
    }
}
