#![feature(unchecked_math)]
use std::{ffi::OsString, ops::Deref, string::FromUtf8Error};

#[derive(Debug)]
pub struct StringErr(String);

impl Deref for StringErr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! err {
    ($($arg:tt)*) => {{
        let res = format!($($arg)*);
       Result::Err(StringErr(res))
    }}
}

macro_rules! im_err {
    ($e: ty) => {
        impl From<$e> for StringErr {
            fn from(e: $e) -> Self {
                Self(format!("{:?}", e))
            }
        }
    };
}

im_err!(std::io::Error);
im_err!(FromUtf8Error);
im_err!(OsString);

mod attr;
mod cp;
mod entry;
mod heap;
mod ins;
mod op;
mod rp;
mod runtime;
mod utils;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    #[test]
    fn test_ptr_size() {
        println!("pointer size = {}", size_of::<usize>())
    }
}
