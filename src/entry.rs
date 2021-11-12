use std::fs;
use std::path::Path;
use crate::utils;

use crate::StringErr;

pub trait Entry {
    // java/lang/Object -> open java/lang/Object.class
    fn read_class(&self, name: &str) -> Option<Vec<u8>>;
}

pub struct DirEntry {
    abs_dir: String
}

impl DirEntry {
    pub fn new(dir: &str) -> Result<Self, StringErr> {
        let buf = fs::canonicalize(dir).unwrap();
        let p = buf.into_os_string().into_string().unwrap();
        let attr = fs::metadata(&p)?;

        if !attr.is_dir() {
            return err!("DirEntry::new {} is not a directory", dir);
        }

        let r = Self {
            abs_dir: p
        };
        Ok(r)
    }
}

impl Entry for DirEntry {
    fn read_class(&self, name: &str) -> Option<Vec<u8>> {
        // join path
        let file = format!("{}.class", name);
        let p = Path::new(&self.abs_dir).join(&file);
        let m = fs::metadata(&p);

        if m.is_err() {
            return None;
        }

        let m = m.unwrap();
        if !m.is_file() {
            return None;
        }

        utils::read_file(p)
    }
}
