use crate::ins::Stack;
use crate::op::OpCode;
use crate::runtime::{misc::BytesReader, misc::OpStack, vm::JFrame, vm::JThread};

trait DupStack {
    fn dup(&mut self);
    fn dup2(&mut self);
    fn dup_x1(&mut self);
    fn dup_x2(&mut self);
    fn dup2_x1(&mut self);
    fn dup2_x2(&mut self);
    fn swap(&mut self);
}

impl DupStack for OpStack {
    fn dup(&mut self) {
        println!("stack size = {}", self.size);
        let top = { self.slots[self.size - 1] };
        self.push_cell(top);
    }

    fn dup2(&mut self) {
        let (v2, v1) = { (self.slots[self.size - 2], self.slots[self.size - 1]) };
        self.slots[self.size] = v2;
        self.slots[self.size + 1] = v1;
        self.size += 2;
    }

    fn dup_x1(&mut self) {
        let v1 = self.slots[self.size - 1];
        let v2 = self.slots[self.size - 2];
        self.slots[self.size - 1] = v2;
        self.slots[self.size - 2] = v1;
        self.push_cell(v1);
    }

    fn dup_x2(&mut self) {
        let v1 = self.slots[self.size - 1];
        let v2 = self.slots[self.size - 2];
        let v3 = self.slots[self.size - 3];
        self.slots[self.size - 1] = v2;
        self.slots[self.size - 2] = v3;
        self.slots[self.size - 3] = v1;
        self.push_cell(v1);
    }

    fn dup2_x1(&mut self) {
        let v1 = self.slots[self.size - 1];
        let v2 = self.slots[self.size - 2];
        let v3 = self.slots[self.size - 3];
        self.slots[self.size - 1] = v3;
        self.slots[self.size - 2] = v1;
        self.slots[self.size - 3] = v2;
        self.slots[self.size] = v2;
        self.slots[self.size + 1] = v1;
        self.size += 2;
    }

    fn dup2_x2(&mut self) {
        let v1 = self.slots[self.size - 1];
        let v2 = self.slots[self.size - 2];
        let v3 = self.slots[self.size - 3];
        let v4 = self.slots[self.size - 4];
        self.slots[self.size - 1] = v3;
        self.slots[self.size - 2] = v4;
        self.slots[self.size - 3] = v1;
        self.slots[self.size - 4] = v2;
        self.slots[self.size] = v2;
        self.slots[self.size + 1] = v1;
        self.size += 2;
    }

    fn swap(&mut self) {
        let v1 = self.slots[self.size - 1];
        let v2 = self.slots[self.size - 2];
        self.slots[self.size - 1] = v2;
        self.slots[self.size - 2] = v1;
    }
}

impl Stack for OpCode {
    fn stack(self, rd: &mut BytesReader, th: &mut JThread, mf: &mut JFrame) {
        use crate::op::OpCode::*;

        match self {
            pop => mf.stack.size -= 1,
            pop2 => mf.stack.size -= 2,
            dup => mf.stack.dup(),
            dup_x1 => mf.stack.dup_x1(),
            dup_x2 => mf.stack.dup_x2(),
            dup2 => mf.stack.dup2(),
            dup2_x1 => mf.stack.dup2_x1(),
            dup2_x2 => mf.stack.dup2_x2(),
            swap => mf.stack.swap(),
            _ => {
                panic!("invalid op {:?}", self);
            }
        };
    }
}
