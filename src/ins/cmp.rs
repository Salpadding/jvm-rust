use crate::ins::Compare;
use crate::op::OpCode;
use crate::runtime::{BytesReader, JThread, JFrame};
use std::rc::Rc;
use std::cell::RefCell;

macro_rules! cmp {
    ($mf: ident, $p: ident, $el: expr) => {
        {
            let (v2, v1) = {
                (
                    $mf.stack.$p(),
                    $mf.stack.$p()
                )
            };

            if v1 > v2 {
                $mf.stack.push_i32(1);
            } else if v1 == v2 {
                $mf.stack.push_i32(0);
            } else if v1 < v2 {
                $mf.stack.push_i32(-1);
            } else {
                $mf.stack.push_i32($el);
            }
        }
    };
}

macro_rules! br_1 {
    ($rd: ident, $mf: ident, $x: ident, $e: expr) => {
        {
            let off = $rd.u16() as usize; 
            let $x = {
                $mf.stack.pop_i32()
            };

            if $e {
                $rd.pc += off;
            }
        }
    };
}

macro_rules! br_2 {
    ($rd: ident, $mf: ident, $p: ident, $x: ident, $y: ident, $e: expr) => {
        {
            let off = $rd.u16() as usize; 
            let ($y, $x) = {
                ($mf.stack.$p(), $mf.stack.$p())
            };

            if $e {
                $rd.pc += off;
            }
        }
    };
}

macro_rules! br_2i {
    ($rd: ident, $mf: ident, $x: ident, $y: ident, $e: expr) => {
        br_2!($rd, $mf, pop_i32, $x, $y, $e)    
    };
}

macro_rules! br_2a {
    ($rd: ident, $mf: ident, $x: ident, $y: ident, $e: expr) => {
        br_2!($rd, $mf, pop_cell, $x, $y, $e)    
    };
}

impl Compare for OpCode {
    fn cmp(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>) {
        use crate::op::OpCode::*;
        use crate::runtime::Slots;
        let mut mf = frame.borrow_mut();

        match self {
            lcmp => cmp!(mf, pop_i64, 0),
            fcmpl => cmp!(mf, pop_f32, -1),
            fcmpg => cmp!(mf, pop_f32, 1),
            dcmpl => cmp!(mf, pop_f64, -1),
            dcmpg => cmp!(mf, pop_f64, 1),
            ifeq => br_1!(rd, mf, x, x == 0),
            ifne => br_1!(rd, mf, x, x != 0),
            iflt => br_1!(rd, mf, x, x < 0),
            ifge => br_1!(rd, mf, x, x >= 0),
            ifgt => br_1!(rd, mf, x, x > 0),
            ifle => br_1!(rd, mf, x, x <= 0),
            if_icmpeq => br_2i!(rd, mf, x, y, x == y),
            if_icmpne => br_2i!(rd, mf, x, y, x != y),
            if_icmplt => br_2i!(rd, mf, x, y, x < y),
            if_icmpge => br_2i!(rd, mf, x, y, x >= y),
            if_icmpgt => br_2i!(rd, mf, x, y, x > y),
            if_icmple => br_2i!(rd, mf, x, y, x <= y),
            if_acmpeq => br_2a!(rd, mf, x, y, x == y),
            if_acmpne => br_2a!(rd, mf, x, y, x != y),
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