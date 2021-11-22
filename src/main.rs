#![feature(unchecked_math)]
#[macro_use]
extern crate err;
mod heap;
mod ins;
mod natives;
mod op;
mod runtime;
mod utils;

fn main() {
    use crate::runtime::vm::Jvm;

    let cp = match std::env::var("CLASSPATH") {
        Ok(v) => v,
        Err(_) => ".".to_string(),
    };

    let args: Vec<String> = std::env::args().collect();
    let mut jvm = Jvm::new(&cp).unwrap();
    jvm.init();
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    #[test]
    fn test_ptr_size() {
        println!("pointer size = {}", size_of::<usize>())
    }
}
