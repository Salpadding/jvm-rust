mod cons;
mod load;
mod store;
mod stack;
mod math;
mod conv;
mod cmp;
mod ctl;

use crate::{op::OpCode, runtime::{JFrame, JThread, Jvm, BytesReader}};
use std::rc::Rc;
use core::cell::RefCell;

pub trait Constant {
    fn con(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}


pub trait Load {
    fn load(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>, wide: bool);
}

pub trait Store {
    fn store(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>, wide: bool);
}

pub trait Stack {
    fn stack(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}

pub trait Math {
    fn math(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>, wide: bool);
}

pub trait Conversion {
    fn conv(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}

pub trait Compare {
    fn cmp(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}

pub trait Control {
    fn ctl(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}

pub trait Ins {
    fn step(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>, wide: bool);
}


impl Ins for u8 {
    fn step(self, rd: &mut BytesReader,  th: &mut JThread, c: Rc<RefCell<JFrame>>, wide: bool) {
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
               let locals = &c.borrow().local_vars;
               th.stack.pop_frame();
               println!("return locals = {:?}", locals);
           }
           _ => panic!("invalid op {}", self)
        }
    }
}