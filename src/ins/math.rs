use crate::ins::Math;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JThread, vm::JFrame};
use std::rc::Rc;
use std::cell::RefCell;
use crate::runtime::misc::Slots;

macro_rules! b_op {
    ($mf: ident, $p: ident, $psh: ident, $f: ident) => {
       {
           let (v2, v1) = {
               (
                $mf.stack.$p(),
                $mf.stack.$p()
               )
           };

           $mf.stack.$psh(v1.$f(v2));
       } 
    };
}

macro_rules! b_i32 {
    ($mf: ident, $f: ident) => {
       unsafe { b_op!($mf, pop_i32, push_i32, $f) }
    };
}

macro_rules! b_u32 {
    ($mf: ident, $f: ident) => {
       { b_op!($mf, pop_u32, push_u32, $f) }
    };
}

macro_rules! b_i64 {
    ($mf: ident, $f: ident) => {
       unsafe { b_op!($mf, pop_i64, push_i64, $f) }
    };
}

macro_rules! b_u64 {
    ($mf: ident, $f: ident) => {
       { b_op!($mf, pop_u64, push_u64, $f) }
    };
}

macro_rules! b_f32 {
    ($mf: ident, $f: ident) => {
       { b_op!($mf, pop_f32, push_f32, $f) }
    };
}

macro_rules! b_f64 {
    ($mf: ident, $f: ident) => {
       { b_op!($mf, pop_f64, push_f64, $f) }
    };
}

macro_rules! u_op {
    ($mf: ident, $pp: ident, $psh: ident, $f: ident) => {
       {
           let v = $mf.stack.$pp();
           $mf.stack.$psh(v.$f());
       } 
    };
}

macro_rules! sh {
    ($mf: ident, $p: ident, $psh: ident, $f: ident, $m: expr) => {
        {
           let (v2, v1) = {
               (
                $mf.stack.pop_u32(),
                $mf.stack.$p()
               )
           };

           $mf.stack.$psh(v1.$f(v2 & $m));
       }           
    };
}

impl Math for OpCode {
    fn math(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>, w: bool) {
        use crate::op::OpCode::*;
        use core::ops::{Add, Mul, Sub, Div, Rem, Neg, Shl, Shr, BitAnd, BitOr, BitXor};
        let mut mf = frame.borrow_mut();

        match self {
            iadd => b_i32!(mf, unchecked_add),
            ladd => b_i64!(mf, unchecked_add),
            fadd => b_f32!(mf, add),
            dadd => b_f64!(mf, add),
            isub => b_i32!(mf, unchecked_sub),
            lsub => b_i64!(mf, unchecked_sub),
            fsub => b_f32!(mf, sub),
            dsub => b_f64!(mf, sub),
            imul => b_i32!(mf, unchecked_mul),
            lmul => b_i64!(mf, unchecked_mul),
            fmul => b_f32!(mf, mul),
            dmul => b_f64!(mf, mul),
            idiv => b_i32!(mf, div),
            ldiv => b_i64!(mf, div),
            fdiv => b_f32!(mf, div),
            ddiv => b_f64!(mf, div),
            irem => b_i32!(mf, rem),
            lrem => b_i64!(mf, rem),
            frem => b_f32!(mf, rem),
            drem => b_f64!(mf, rem),
            ineg => u_op!(mf, pop_i32, push_i32, neg),
            lneg => u_op!(mf, pop_i64, push_i64, neg),
            fneg => u_op!(mf, pop_f32, push_f32, neg),
            dneg => u_op!(mf, pop_f64, push_f64, neg),
            ishl => sh!(mf, pop_i32, push_i32, shl, 0x1fu32),
            lshl => sh!(mf, pop_i64, push_i64, shl, 0x3fu32),
            ishr => sh!(mf, pop_i32, push_i32, shr, 0x1fu32),
            lshr => sh!(mf, pop_i64, push_i64, shr, 0x3fu32),
            iushr => sh!(mf, pop_u32, push_u32, shl, 0x1fu32),
            lushr => sh!(mf, pop_u64, push_u64, shl, 0x3fu32),
            iand => b_u32!(mf, bitand),
            land => b_u64!(mf, bitand),
            ior => b_u32!(mf, bitor),
            lor => b_u64!(mf, bitor),
            ixor => b_u32!(mf, bitxor),
            lxor => b_u64!(mf, bitxor),
            iinc => {
                let (i, c) = {
                    (
                        if w { rd.u16() as usize } else { rd.u8() as usize },
                        if w { rd.u16() as i16 as i32 } else { rd.u8() as i8 as i32 },
                    )
                };
                let v = {
                    mf.local_vars.get_i32(i)
                };
                mf.local_vars.set_i32(i, v + c);
            },
            _ => panic!("invalid op {:?}", self)
        };
    }
}