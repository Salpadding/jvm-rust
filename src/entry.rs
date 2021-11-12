use std::fmt::Write;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::utils;

use crate::StringErr;

pub trait Entry: std::fmt::Debug {
    // java/lang/Object -> open java/lang/Object.class
    fn read_class(&self, name: &str) -> Option<Vec<u8>>;
}

#[derive(Debug)]
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
        use crate::utils::norm_path;
        // join path
        let file = format!("{}.class", norm_path(name));
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


#[derive(Debug)]
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
           println!("name = {}", file.name());
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

pub struct CompositeEntry {
    children: Vec<Box<dyn Entry>>,
}

impl Entry for CompositeEntry {
    fn read_class(&self, name: &str) -> Option<Vec<u8>> {
        for e in self.children.iter() {
            match e.read_class(name) {
                Some(v) => { return Some(v); }
                None => continue, 
            }
        }
        None
    }
}

impl std::fmt::Debug for CompositeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ele in self.children.iter() {
            ele.fmt(f)?;
            f.write_char('\n')?;
        }
        Ok(())
    }
}

impl CompositeEntry {
    // spilt path by ':'
    fn from_paths(paths: &str) -> Result<Self, StringErr> {
        let sp: Vec<String> = paths.split(':').map(|x| x.to_string()).collect();
        let mut children: Vec<Box<dyn Entry>> = Vec::with_capacity(sp.len());

        for p in sp.iter() {
            let e = new_entry(p)?;
            children.push(e);
        }

        Ok(CompositeEntry { children })
    }

    // from wildcard
    fn from_wildcard(path: &str) -> Result<Self, StringErr> {
        // trim suffix *
        let trim = &path.as_bytes()[..path.len() - 1];
        let trim = String::from_utf8(trim.to_vec())?;
        let mut children: Vec<Box<dyn Entry>> = Vec::new();

        // traverse directory, skip sub directories
        // add .zip, .jar, .ZIP, .JAR entries

        let dir = fs::read_dir(&trim)?;

        for d in dir {
            let d = d?;
            let md = fs::metadata(d.path())?;

            if !md.is_file() {
                continue;
            }

            let n = Path::new(&trim).join(d.file_name()).into_os_string().into_string()?;
            if  n.ends_with(".jar") || n.ends_with(".JAR") {
                let e = ZipEntry::new(&n)?;
                children.push(Box::new(e));
            }
        }

        Ok(CompositeEntry { children })
    }
}

pub fn new_entry(path: &str) -> Result<Box<dyn Entry>, StringErr> {
    // if contains :
    if path.contains(':') {
        let c = CompositeEntry::from_paths(path)?;
        return Ok(Box::new(c));
    }

    // if ends with *
    if path.ends_with('*') {
        let e = CompositeEntry::from_wildcard(path)?;
        return Ok(Box::new(e));
    }

    // if ends with .zip .jar .ZIP .JAR
    if path.ends_with(".zip") || path.ends_with(".jar") || path.ends_with(".ZIP") || path.ends_with(".JAR") {
        let d = ZipEntry::new(path)?;
        return Ok(Box::new(d));
    }

    let d = DirEntry::new(path)?;
    Ok(Box::new(d))
}

#[cfg(test)]
mod test{
    use super::Entry;

    #[test]
    fn test_dir_entry() {
        let e = super::DirEntry::new(".").unwrap();
        e.read_class("test/Test").unwrap();
    }
}