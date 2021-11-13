mod cons;
mod load;

use crate::{op::OpCode, runtime::{JFrame, JThread, Jvm, BytesReader}};
use std::rc::Rc;
use core::cell::RefCell;

pub trait Constant {
    fn con(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}


pub trait Load {
    fn load(self, rd: &mut BytesReader,  th: &mut JThread, frame: Rc<RefCell<JFrame>>);
}

