use crate::cp::{ClassFile, ConstantPool, MemberInfo};
use crate::attr::AttrInfo;
use crate::entry::Entry;
use std::collections::BTreeMap;
use std::sync::Arc;
use crate::{StringErr, entry};

impl From<ClassFile> for Class {
    fn from(c: ClassFile) -> Self {
       let mut c = c;
       let mut r = Class::default(); 
       r.access_flags = AccessFlags(c.access_flags);
       r.name = c.this_class().to_string();
       r.super_name = c.super_class().to_string();
       r.iface_names = c.interfaces_i.iter().map(|x| c.cp.class(*x as usize).to_string()).collect();
       r.fields = c.fields.iter().map(|x| Arc::new(x.into())).collect();
       r.methods = c.methods.iter().map(|x| Arc::new(x.into())).collect();

       core::mem::swap(&mut c.cp, &mut r.cp);
       r
    }
}

impl From<&MemberInfo> for ClassMember {
    fn from(m: &MemberInfo) -> Self {
        let mut r = ClassMember::default();
            r.name = m.name.to_string();
            r.access_flags = AccessFlags(m.access_flags);
            r.desc= m.desc.to_string();

        for attr in m.attrs.iter() {
            match attr {
               &AttrInfo::Code(ref c)  => {
                 r.code = c.code.clone();
                 r.max_stack = c.max_stack as usize;
                 r.max_locals = c.max_locals as usize;
               },
               _ => {}
            }
        }
            r

    }
}

#[derive(Debug, Default)]
pub struct Class {
    pub access_flags: AccessFlags,
    pub name: String,
    pub super_name: String,
    pub iface_names: Vec<String>,
    pub cp: ConstantPool,
    pub fields: Vec<Arc<ClassMember>>,
    pub methods: Vec<Arc<ClassMember>>,

    pub super_class: Option<Arc<Class>>,
    pub interfaces: Vec<Arc<Class>>,
}

impl Class {
    pub fn main_method(&self) -> Option<Arc<ClassMember>> {
        for m in self.methods.iter() {
            if &m.name == "main" && &m.desc == "([Ljava/lang/String;)V" {
                return Some(m.clone());
            }
        }
        None
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
}


#[derive(Debug, Default)]
pub struct AccessFlags(u16);


#[derive(Debug)]
pub struct ClassLoader {
    entry: Box<dyn Entry>,
    loaded: BTreeMap<String, Arc<Class>>,
}


impl ClassLoader {
    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let entry = entry::new_entry(cp)?;

        Ok(
            ClassLoader {
                entry,
                loaded: BTreeMap::new(),
            }
        )
    }

    pub fn load(&mut self, name: &str) -> Arc<Class> {
        match self.loaded.get(name) {
           Some(cl)  => return cl.clone(),
           _ => {},
        };

        let bytes = self.entry.read_class(name).unwrap();
        self.define(name, bytes)
    }

    fn define(&mut self, name: &str, bytes: Vec<u8>) -> Arc<Class> {
        let file = ClassFile::new(bytes);
        let mut cl: Class = file.into();

        // load super and interfaces
        if &cl.super_name != "" {
            cl.super_class = Some(self.load(&cl.super_name));
        }

        let mut ifaces: Vec<Arc<Class>> = Vec::with_capacity(cl.iface_names.len());
        for n in cl.iface_names.iter() {
            ifaces.push(self.load(n));
        }

        cl.interfaces = ifaces;

        let arc = Arc::new(cl);
        let cloned = arc.clone();
        self.loaded.insert(name.to_string(), arc);
        cloned

    }
}

#[cfg(test)]
mod test {
    use super::ClassLoader;

    #[test]
    fn loader_test() {
        let mut loader = ClassLoader::new(".:test/rt.jar").unwrap();
        let class = loader.load("test/Test");
    }
}