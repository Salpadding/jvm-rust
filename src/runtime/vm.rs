use std::{cell::RefCell, cell::Ref};
use std::rc::Rc;

use crate::StringErr;
use crate::heap::{Class, ClassLoader, ClassMember, Heap, Object, SymRef};
use crate::runtime::misc::{BytesReader, Slots, OpStack};

const MAX_JSTACK_SIZE: usize = 1024;


// jvm runtime representation
#[derive(Debug)]
pub struct Jvm {
    heap: Rc<RefCell<Heap>>,
    thread: JThread,
}

impl Jvm {
    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let heap = Heap::new(cp)?;
        let heap = Rc::new(RefCell::new(heap));
        Ok(
            Jvm {
                heap: heap.clone(),
                thread: JThread::new(heap),
            }
        )
    }

    pub fn run_class(&mut self, c: &str) -> Result<(), StringErr> {
        // load class
        let c = {
            let mut h = self.heap.borrow_mut();
            h.loader.load(c)
        };

        // get main method
        let main = c.borrow().main_method();
        if main.is_none() {
            return err!("class {} has no main method", &c.borrow().name);
        }

        self.thread.stack.push_frame(JFrame::new(self.heap.clone(), c, main.unwrap()));
        self.thread.run();

        Ok(())
    }
}


#[derive(Debug)]
pub struct JThread {
    pub pc: i32,
    pub stack: JStack,
    pub next_pc: Option<i32>,
    pub heap: Rc<RefCell<Heap>>,
}

impl JThread {
    pub fn branch(&mut self, off: i32) {
        self.next_pc = Some(off);
    }
}

impl JThread {
    pub fn new(heap: Rc<RefCell<Heap>>) -> Self {
        Self {
            heap,
            pc: 0,
            stack: JStack::new(),
            next_pc: None,
        }
    }

    pub fn cur_frame(&self) -> Rc<RefCell<JFrame>> {
        self.stack.cur_frame()
    }


    pub fn run(&mut self) {
        use crate::ins::Ins;
        while !self.stack.is_empty() {
            let f = self.cur_frame();
            let method = {
                let b: Ref<JFrame> = RefCell::borrow(&*f);
                b.method.clone()
            };
            self.next_pc = None;
            let mut rd = BytesReader {
                bytes: &method.code,
                pc: self.pc,
            };

            let op: u8 = rd.u8();

            // wide
            if op == 0xc4 {
                let op: u8 = rd.u8();
                op.step(&mut rd, self, f, true);
            } else {
                op.step(&mut rd, self, f, false);
            }

            match self.next_pc {
                None => self.pc = rd.pc,
                Some(off) => self.pc += off,
            }
        }
    }
}

// TODO: limit stack size
#[derive(Debug, Default)]
pub struct JStack {
    pub max_size: usize,
    pub frames: Vec<Option<Rc<RefCell<JFrame>>>>,
    pub size: usize,
}

impl JStack {
    fn new() -> Self {
        Self {
            max_size: MAX_JSTACK_SIZE,
            frames: vec![None; MAX_JSTACK_SIZE],
            size: 0,
        }
    }

    fn push_frame(&mut self, frame: JFrame) {
        self.frames[self.size] = Some(Rc::new(RefCell::new(frame)));
        self.size += 1;
    }

    pub fn pop_frame(&mut self) -> Rc<RefCell<JFrame>> {
        let top = self.frames[self.size - 1].as_ref().unwrap().clone();
        self.size -= 1;
        top
    }

    fn cur_frame(&self) -> Rc<RefCell<JFrame>> {
        self.frames[self.size - 1].as_ref().unwrap().clone()
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }
}


#[derive(Debug)]
pub struct JFrame {
    pub local_vars: Vec<u64>,
    pub stack: OpStack,
    pub method: Rc<ClassMember>,
    pub class: Rc<RefCell<Class>>,
    pub heap: Rc<RefCell<Heap>>,
}

macro_rules! xx_ref {
    ($f: ident) => {
        pub fn $f(&self, i: usize) -> Rc<SymRef> {
            let mut cur = self.class.borrow_mut();
            let sym = {
                let mut heap = self.heap.borrow_mut();
                heap.$f(&mut cur, i)
            };
            sym
        }
    };
}

impl JFrame {
    pub fn new(heap: Rc<RefCell<Heap>>, class: Rc<RefCell<Class>>, method: Rc<ClassMember>) -> Self {
        Self {
            local_vars: vec![0u64; method.max_locals],
            stack: OpStack { slots: vec![0u64; method.max_stack], size: 0 },
            method,
            class,
            heap,
        }
    }

    xx_ref!(class_ref);
    xx_ref!(field_ref);

    pub fn new_obj(&self, class: Rc<RefCell<Class>>) -> Box<Object> {
        self.heap.borrow().new_obj(class)
    }
}
