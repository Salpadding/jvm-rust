mod cmp;
mod cons;
mod conv;
mod ctl;
mod load;
mod math;
mod refs;
mod stack;
mod store;

use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, vm::JFrame, vm::JThread};

use core::cell::RefCell;
use std::rc::Rc;

pub trait Constant {
    fn con(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

pub trait Load {
    fn load(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame, wide: bool);
}

pub trait Store {
    fn store(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame, wide: bool);
}

pub trait Stack {
    fn stack(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

pub trait Math {
    fn math(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame, wide: bool);
}

pub trait Conversion {
    fn conv(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

pub trait Compare {
    fn cmp(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

pub trait Control {
    fn ctl(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

pub trait Ins {
    fn step(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame, wide: bool);
}

pub trait Refs {
    fn refs(self, rd: &mut BytesReader, th: &mut JThread, frame: &mut JFrame);
}

impl Ins for u8 {
    fn step(self, rd: &mut BytesReader, th: &mut JThread, c: &mut JFrame, wide: bool) {
        let op: OpCode = self.into();

        match self {
            0x00..=0x14 => op.con(rd, th, c),
            0x15..=0x35 => op.load(rd, th, c, wide),
            0x36..=0x56 => op.store(rd, th, c, wide),
            0x57..=0x5f => op.stack(rd, th, c),
            0x60..=0x84 => op.math(rd, th, c, wide),
            0x85..=0x93 => op.conv(rd, th, c),
            0x94..=0xa6 | 0xc6 | 0xc7 => op.cmp(rd, th, c),
            0xa7..=0xb0 | 0xc8 => op.ctl(rd, th, c),
            0xb1 => {
                let locals = &c.local_vars;
                th.stack.pop_frame();
                println!("return locals = {:?}", locals);
            }
            0xb2..=0xc3 => {
                op.refs(rd, th, c);
            }
            _ => panic!("invalid op {}", self),
        }
    }
}
