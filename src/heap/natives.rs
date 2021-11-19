use crate::{
    rp::Rp,
    runtime::vm::{JFrame, JThread},
};
use std::collections::BTreeMap;

use super::desc::{DescriptorParser, JType, MethodDescriptor};

pub trait NativeMethod {
    fn class_name(&self) -> &str;
    fn method_name(&self) -> &str;
    fn desc(&self) -> &str;
    fn exec(&self, f: &mut JFrame);
}

pub struct NativeMethodW {
    pub inner: Box<dyn NativeMethod>,
    pub desc: MethodDescriptor,
}

#[derive(Default)]
pub struct NativeRegistry {
    data: BTreeMap<String, NativeMethodW>,
}

impl NativeRegistry {
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
        self.data
            .get(&self.hash(class, method, desc))
            .as_ref()
            .unwrap()
    }
}
