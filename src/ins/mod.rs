mod cons;
mod load;
mod store;
mod stack;
mod math;
mod conv;

use crate::{op::OpCode, runtime::{JFrame, JThread, Jvm, BytesReader}};
use std::rc::Rc;
use core::cell::RefCell;

pub trait Constant {
    fn con(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}


pub trait Load {
    fn load(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}

pub trait Store {
    fn store(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}

pub trait Stack {
    fn stack(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}

pub trait Math {
    fn math(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}

pub trait Conversion {
    fn conv(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}