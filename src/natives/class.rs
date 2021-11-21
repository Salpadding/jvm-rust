use crate::natives::NativeMethod;
use crate::runtime::vm::{JFrame, JThread};
pub struct ClassReg {}

impl NativeMethod for ClassReg {
    fn class_name(&self) -> &str {
        "java/lang/Class"
    }
    fn desc(&self) -> &str {
        "()V"
    }

    fn method_name(&self) -> &str {
        "registerNatives"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {}
}
