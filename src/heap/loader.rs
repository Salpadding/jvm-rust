use crate::cp::ClassFile;
use crate::entry;
use crate::entry::Entry;
use crate::heap::class::Class;
use crate::rp::Rp;
use crate::StringErr;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ClassLoader {
    entry: Box<dyn Entry>,
    loaded: BTreeMap<String, Rp<Class>>,
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
            cl.super_class.count_ins_fields()
        };

        for i in 0..base {
            cl.ins_fields.push(cl.super_class.get_ins_field(i));
        }

        let mut base = base;
        for f in cl.fields.iter().filter(|x| !x.access_flags.is_static()) {
            f.get_mut().id = base as i32;
            cl.ins_fields.push(*f);
            base += 1;
        }

        // link members to class
        let mut p = Rp::new(cl);
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
