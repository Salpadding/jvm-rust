use crate::natives::NativeMethod;
use crate::runtime::vm::JFrame;

pub struct JLOReg {}

impl NativeMethod for JLOReg {
    jlo!();

    fn desc(&self) -> &str {
        "()V"
    }

    fn method_name(&self) -> &str {
        "registerNatives"
    }

    fn exec(&self, f: &mut JFrame) {
        println!("{}", "Object.registerNatives");
        reg!(f.registry, JLOgetClass);
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

    fn exec(&self, f: &mut JFrame) {
        let ths = f.this();
        f.stack.push_obj(ths.class.j_class)
    }
}
