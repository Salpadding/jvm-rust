use std::{ops::Deref};

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

mod utils;
mod attr;
mod cp;
mod entry;

fn main() {
    println!("Hello, world!");
}
