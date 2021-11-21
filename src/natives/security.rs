use crate::natives::NativeMethod;
use crate::runtime::vm::{JFrame, JThread};
pub struct ACDopri {}
pub struct ACDopri2 {}

pub struct ACGetCtx {}

macro_rules! ac {
    () => {
        fn exec(&self, th: &mut JThread, f: &mut JFrame) {
            let this = f.this();
            f.stack.push_obj(this);

            th.invoke_obj(
                this.get_mut(),
                "run",
                "()Ljava/lang/Object;",
                &[this.ptr() as u64],
            );
        }
    };
}

impl NativeMethod for ACDopri {
    fn class_name(&self) -> &str {
        "java/security/AccessController"
    }
    fn desc(&self) -> &str {
        "(Ljava/security/PrivilegedExceptionAction;)Ljava/lang/Object;"
    }

    fn method_name(&self) -> &str {
        "doPrivileged"
    }

    ac!();
}

impl NativeMethod for ACDopri2 {
    fn class_name(&self) -> &str {
        "java/security/AccessController"
    }
    fn desc(&self) -> &str {
        "(Ljava/security/PrivilegedAction;)Ljava/lang/Object;"
    }

    fn method_name(&self) -> &str {
        "doPrivileged"
    }

    ac!();
}

impl NativeMethod for ACGetCtx {
    fn class_name(&self) -> &str {
        "java/security/AccessController"
    }
    fn desc(&self) -> &str {
        "()Ljava/security/AccessControlContext;"
    }

    fn method_name(&self) -> &str {
        "getStackAccessControlContext"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        f.stack.push_null()
    }
}
