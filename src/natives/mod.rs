macro_rules! na {
    ($n: ident, $c: expr, $m: expr, $d: expr, $th: ident, $f: ident, $b: block) => {
        pub struct $n {}

        impl crate::natives::NativeMethod for $n {
            fn class_name(&self) -> &str {
                $c
            }

            fn method_name(&self) -> &str {
                $m
            }

            fn desc(&self) -> &str {
                $d
            }

            fn exec(
                &self,
                $th: &mut crate::runtime::vm::JThread,
                $f: &mut crate::runtime::frame::JFrame,
            ) {
                $b
            }
        }
    };
}

macro_rules! reg {
    ($r: expr, $($i: ident),*) => {{
        $(
            $r.register(std::boxed::Box::new($i {}));
        )*
    }};
}

mod class;
mod debug;
mod object;
mod sun;
mod system;
mod thread;

use crate::runtime::frame::JFrame;
use crate::runtime::vm::JThread;
use std::collections::BTreeMap;

use crate::heap::desc::{DescriptorParser, MethodDescriptor};

pub trait NativeMethod {
    fn class_name(&self) -> &str;
    fn method_name(&self) -> &str;
    fn desc(&self) -> &str;
    fn exec(&self, th: &mut JThread, f: &mut JFrame);
}

pub struct NativeMethodW {
    pub inner: Box<dyn NativeMethod>,
    pub desc: MethodDescriptor,
}

pub struct NativeRegistry {
    data: BTreeMap<String, NativeMethodW>,
}

impl NativeRegistry {
    pub fn new() -> Self {
        let mut r = NativeRegistry {
            data: BTreeMap::new(),
        };
        use crate::natives::class::ClassReg;
        use crate::natives::debug::DebugReg;
        use crate::natives::object::JLOReg;
        use crate::natives::sun::UnsafeReg;
        use crate::natives::system::JLSReg;
        use crate::natives::thread::ThreadReg;
        reg!(
            r, DebugReg, JLOReg, JLSReg, ClassReg, ThreadReg,
            // ReflectCallerClass,
            UnsafeReg // ACGetCtx,
                      // ACDopri,
                      // ACDopri2
        );
        r
    }

    #[inline]
    fn hash(&self, class: &str, method: &str, desc: &str) -> String {
        format!("{}_{}_{}", class, method, desc)
    }
    pub fn register(&mut self, native: Box<dyn NativeMethod>) {
        let h = self.hash(&native.class_name(), &native.method_name(), &native.desc());
        let mut parser = DescriptorParser::new(native.desc().as_bytes());
        let desc = parser.parse_method();

        let o = NativeMethodW {
            inner: native,
            desc: desc,
        };

        self.data.insert(h, o);
    }
    pub fn find(&self, class: &str, method: &str, desc: &str) -> &NativeMethodW {
        let h = self.hash(class, method, desc);
        let data = self.data.get(&h);

        if data.is_none() {
            panic!("native method {} not found", h);
        }

        data.unwrap()
    }
}
