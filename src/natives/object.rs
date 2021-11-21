use crate::natives::NativeMethod;
use crate::runtime::vm::{JFrame, JThread};

macro_rules! jlo {
    () => {
        fn class_name(&self) -> &str {
            "java/lang/Object"
        }
    };
}

pub struct JLOReg {}

impl NativeMethod for JLOReg {
    jlo!();

    fn desc(&self) -> &str {
        "()V"
    }

    fn method_name(&self) -> &str {
        "registerNatives"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        reg!(f.registry, JLOgetClass, JLOHashCode);
    }
}

pub struct JLOgetClass {}

impl NativeMethod for JLOgetClass {
    jlo!();

    fn desc(&self) -> &str {
        "()Ljava/lang/Class;"
    }

    fn method_name(&self) -> &str {
        "getClass"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        let ths = f.this();
        f.stack.push_obj(ths.class.j_class);
    }
}

pub struct JLOHashCode {}

impl NativeMethod for JLOHashCode {
    jlo!();

    fn desc(&self) -> &str {
        "()I"
    }

    fn method_name(&self) -> &str {
        "hashCode"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        let ths = f.this();
        f.stack.push_u32(ths.ptr() as u32)
    }
}
