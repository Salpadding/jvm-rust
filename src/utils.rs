use std::path::Path;
use std::io::Read;

pub fn read_file<P: AsRef<Path>>(path: P) -> Option<Vec<u8>> {
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