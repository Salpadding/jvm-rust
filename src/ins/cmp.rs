use crate::ins::Compare;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JFrame, vm::JThread};

macro_rules! cmp {
    ($mf: ident, $p: ident, $el: expr) => {{
        let (v2, v1) = { ($mf.stack.$p(), $mf.stack.$p()) };

        if v1 > v2 {
            $mf.stack.push_i32(1);
        } else if v1 == v2 {
            $mf.stack.push_i32(0);
        } else if v1 < v2 {
            $mf.stack.push_i32(-1);
        } else {
            $mf.stack.push_i32($el);
        }
    }};
}

macro_rules! br_1 {
    ($th: ident, $rd: ident, $mf: ident, $p: ident, $x: ident, $e: expr) => {{
        let off = $rd.i16() as i32;
        let $x = { $mf.stack.$p() };

        if $e {
            $th.branch(off);
        }
    }};
}

macro_rules! br_1i {
    ($th: ident, $rd: ident, $mf: ident, $x: ident, $e: expr) => {
        br_1!($th, $rd, $mf, pop_i32, $x, $e)
    };
}

macro_rules! br_1a {
    ($th: ident, $rd: ident, $mf: ident, $x: ident, $e: expr) => {
        br_1!($th, $rd, $mf, pop_cell, $x, $e)
    };
}

macro_rules! br_2 {
    ($th: ident, $rd: ident, $mf: ident, $p: ident, $x: ident, $y: ident, $e: expr) => {{
        let off = $rd.i16() as i32;
        let ($y, $x) = { ($mf.stack.$p(), $mf.stack.$p()) };

        if $e {
            $th.branch(off);
        }
    }};
}

macro_rules! br_2i {
    ($th: ident, $rd: ident, $mf: ident, $x: ident, $y: ident, $e: expr) => {
        br_2!($th, $rd, $mf, pop_i32, $x, $y, $e)
    };
}

macro_rules! br_2a {
    ($th: ident, $rd: ident, $mf: ident, $x: ident, $y: ident, $e: expr) => {
        br_2!($th, $rd, $mf, pop_cell, $x, $y, $e)
    };
}

impl Compare for OpCode {
    fn cmp(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            lcmp => cmp!(mf, pop_i64, 0),
            fcmpl => cmp!(mf, pop_f32, -1),
            fcmpg => cmp!(mf, pop_f32, 1),
            dcmpl => cmp!(mf, pop_f64, -1),
            dcmpg => cmp!(mf, pop_f64, 1),
            ifeq => br_1i!(th, rd, mf, x, x == 0),
            ifne => br_1i!(th, rd, mf, x, x != 0),
            iflt => br_1i!(th, rd, mf, x, x < 0),
            ifge => br_1i!(th, rd, mf, x, x >= 0),
            ifgt => br_1i!(th, rd, mf, x, x > 0),
            ifle => br_1i!(th, rd, mf, x, x <= 0),
            if_icmpeq => br_2i!(th, rd, mf, x, y, x == y),
            if_icmpne => br_2i!(th, rd, mf, x, y, x != y),
            if_icmplt => br_2i!(th, rd, mf, x, y, x < y),
            if_icmpge => br_2i!(th, rd, mf, x, y, x >= y),
            if_icmpgt => br_2i!(th, rd, mf, x, y, x > y),
            if_icmple => br_2i!(th, rd, mf, x, y, x <= y),
            if_acmpeq => br_2a!(th, rd, mf, x, y, x == y),
            if_acmpne => br_2a!(th, rd, mf, x, y, x != y),

            ifnull => br_1a!(th, rd, mf, x, x == 0),
            ifnonnull => br_1a!(th, rd, mf, x, x != 0),
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn nan_test() {
        let l = f32::NAN;

        println!("{}", l == l);
    }
}
