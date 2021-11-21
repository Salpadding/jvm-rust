use std::mem::size_of;

use crate::natives::NativeMethod;
use crate::runtime::vm::{JFrame, JThread};
pub struct ReflectCallerClass {}

impl NativeMethod for ReflectCallerClass {
    fn class_name(&self) -> &str {
        "sun/reflect/Reflection"
    }
    fn desc(&self) -> &str {
        "()Ljava/lang/Class;"
    }

    fn method_name(&self) -> &str {
        "getCallerClass"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        let caller_frame = th.back_frame(3);
        let caller_class = caller_frame.class.j_class;
        println!("caller class = {}", caller_frame.class.name);
        f.stack.push_obj(caller_class)
    }
}

pub struct UnsafeReg {}

impl NativeMethod for UnsafeReg {
    fn class_name(&self) -> &str {
        "sun/misc/Unsafe"
    }
    fn desc(&self) -> &str {
        "()V"
    }

    fn method_name(&self) -> &str {
        "registerNatives"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        reg!(
            th.registry,
            UnsafeArrBaseOff,
            UnsafeArrIdxScale,
            UnsafeAddrSize
        )
    }
}

pub struct UnsafeArrBaseOff {}

impl NativeMethod for UnsafeArrBaseOff {
    fn class_name(&self) -> &str {
        "sun/misc/Unsafe"
    }
    fn desc(&self) -> &str {
        "(Ljava/lang/Class;)I"
    }

    fn method_name(&self) -> &str {
        "arrayBaseOffset"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        f.stack.push_u32(0)
    }
}

pub struct UnsafeArrIdxScale {}

impl NativeMethod for UnsafeArrIdxScale {
    fn class_name(&self) -> &str {
        "sun/misc/Unsafe"
    }
    fn desc(&self) -> &str {
        "(Ljava/lang/Class;)I"
    }

    fn method_name(&self) -> &str {
        "arrayIndexScale"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        f.stack.push_u32(0)
    }
}

pub struct UnsafeAddrSize {}

impl NativeMethod for UnsafeAddrSize {
    fn class_name(&self) -> &str {
        "sun/misc/Unsafe"
    }
    fn desc(&self) -> &str {
        "()I"
    }

    fn method_name(&self) -> &str {
        "addressSize"
    }

    fn exec(&self, th: &mut JThread, f: &mut JFrame) {
        f.stack.push_u32(size_of::<usize>() as u32)
    }
}
