use std::fmt::format;
use std::fs;
use std::fs::File;
use std::io::Read;
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


pub struct ZipEntry {
    // path for zip
    zip: String,
}

impl ZipEntry {
    fn new(path: &str) -> Result<ZipEntry, StringErr>{
        let m = fs::metadata(path)?;
        if !m.is_file() {
            return err!("create zip entry failed: {} is not a regular file", path);
        }

        let buf = fs::canonicalize(path).unwrap();
        let p = buf.into_os_string().into_string().unwrap();

        Ok(ZipEntry { zip: p })
    }
}


impl Entry for ZipEntry {
    fn read_class(&self, name: &str) -> Option<Vec<u8>> {
        let full_name = format!("{}.class", name);
       let file = File::open(&self.zip);
       if file.is_err() {
           return None;
       }

       let file = file.unwrap();
       let archive = zip::ZipArchive::new(file);

       if archive.is_err() {
           return None;
       }
       let mut archive = archive.unwrap();

       for i in 0..archive.len() {
           let mut file = archive.by_index(i).unwrap();
           if file.name() !=  &full_name {
               continue;
           }

           // file found
           // read all from file
           let mut r: Vec<u8> = Vec::new();
           file.read_to_end(&mut r).unwrap();

           return Some(r);
       }

       return None;
    }
}
