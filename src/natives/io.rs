use crate::natives::NativeMethod;
use crate::runtime::vm::{JFrame, JThread};
pub struct FileOSInitIds {}
pub struct FileISInitIds {}
pub struct FDInitIds {}

impl NativeMethod for FileOSInitIds {
    fn class_name(&self) -> &str {
        "java/io/FileOutputStream"
    }
    fn desc(&self) -> &str {
        "()V"
    }

    fn method_name(&self) -> &str {
        "initIDs"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {}
}

impl NativeMethod for FileISInitIds {
    fn class_name(&self) -> &str {
        "java/io/FileInputStream"
    }
    fn desc(&self) -> &str {
        "()V"
    }

    fn method_name(&self) -> &str {
        "initIDs"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {}
}

impl NativeMethod for FDInitIds {
    fn class_name(&self) -> &str {
        "java/io/FileDescriptor"
    }
    fn desc(&self) -> &str {
        "()V"
    }

    fn method_name(&self) -> &str {
        "initIDs"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        reg!(th.registry, FDSet);
    }
}

pub struct FDSet {}

impl NativeMethod for FDSet {
    fn class_name(&self) -> &str {
        "java/io/FileDescriptor"
    }
    fn desc(&self) -> &str {
        "(I)J"
    }

    fn method_name(&self) -> &str {
        "set"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        f.stack.push_u64(f.local_vars[0]);
    }
}
