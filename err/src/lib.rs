use core::ops::Deref;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub struct StringErr(pub String);

impl Deref for StringErr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {{
        let res = format!($($arg)*);
       Result::Err(err::StringErr(res))
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
im_err!(std::ffi::OsString);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
