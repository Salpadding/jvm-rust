macro_rules! reg {
    ($r: expr, $($i: ident),+) => {{
        $(
            $r.register(std::boxed::Box::new($i {}));
        )+
    }};
}

mod class;
mod io;
mod object;
mod security;
mod sun;
mod system;

use crate::runtime::vm::{JFrame, JThread};
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
        use crate::natives::io::{FDInitIds, FileISInitIds, FileOSInitIds};
        use crate::natives::object::JLOReg;
        use crate::natives::security::*;
        use crate::natives::sun::{ReflectCallerClass, UnsafeReg};
        use crate::natives::system::{JLSReg, VM};
        reg!(
            r,
            JLOReg,
            JLSReg,
            VM,
            FileOSInitIds,
            FileISInitIds,
            FDInitIds,
            ReflectCallerClass,
            UnsafeReg,
            ACGetCtx,
            ACDopri,
            ACDopri2,
            ClassReg
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
