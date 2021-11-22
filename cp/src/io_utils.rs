use std::io::Read;
use std::path::{self, Path};

pub(crate) fn read_file<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
    use std::fs;
    let f = fs::File::open(path);
    if f.is_err() {
        return None;
    }
    let mut f = f.unwrap();
    let mut buf: Vec<u8> = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    Some(buf)
}

pub(crate) fn norm_path(p: &str) -> String {
    let mut s = String::with_capacity(p.len());
    for c in p.chars().into_iter() {
        if c == '\\' || c == '/' {
            s.push(path::MAIN_SEPARATOR)
        } else {
            s.push(c)
        }
    }
    s
}
