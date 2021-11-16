use crate::StringErr;
use crate::heap::{class::Class, class::ClassMember, misc::Heap, class::Object, misc::SymRef};
use crate::runtime::misc::{BytesReader, OpStack};
use crate::rp::Rp;
const MAX_JSTACK_SIZE: usize = 1024;


// jvm runtime representation
#[derive(Debug)]
pub struct Jvm {
    heap: Rp<Heap>,
    thread: JThread,
}

impl Jvm {
    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let heap = Heap::new(cp)?;
        let p = Rp::new(heap);
        Ok(
            Jvm {
                heap: p,
                thread: JThread::new(p),
            }
        )
    }

    pub fn run_class(&mut self, c: &str) -> Result<(), StringErr> {
        // load class
        let c = 
            self.heap.loader.load(c);

        // get main method
        let main = c.main_method();

        if main.is_null() {
            return err!("class {} has no main method", &c.name);
        }

        self.thread.stack.push_frame(JFrame::new(self.heap, c, main));
        self.thread.run();

        Ok(())
    }
}


#[derive(Debug)]
pub struct JThread {
    pub pc: i32,
    pub stack: JStack,
    pub next_pc: Option<i32>,
    pub heap: Rp<Heap>,
}

impl JThread {
    pub fn branch(&mut self, off: i32) {
        self.next_pc = Some(off);
    }
}

impl JThread {
    pub fn new(heap: Rp<Heap>) -> Self {
        Self {
            heap,
            pc: 0,
            stack: JStack::new(),
            next_pc: None,
        }
    }

    pub fn cur_frame(&self) -> Rp<JFrame> {
        self.stack.cur_frame()
    }


    pub fn run(&mut self) {
        use crate::ins::Ins;
        while !self.stack.is_empty() {
            let f = self.cur_frame();
            let method = f.method;
            self.next_pc = None;
            let mut rd = BytesReader {
                bytes: &method.code,
                pc: self.pc,
            };

            let op: u8 = rd.u8();

            // wide
            if op == 0xc4 {
                let op: u8 = rd.u8();
                op.step(&mut rd, self, f.get_mut(), true);
            } else {
                op.step(&mut rd, self, f.get_mut(), false);
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
    pub frames: Vec<Rp<JFrame>>,
    pub size: usize,
}

impl JStack {
    fn new() -> Self {
        Self {
            max_size: MAX_JSTACK_SIZE,
            frames: vec![Rp::null(); MAX_JSTACK_SIZE],
            size: 0,
        }
    }

    fn push_frame(&mut self, frame: JFrame) {
        self.frames[self.size] = Rp::new(frame);
        self.size += 1;
    }

    pub fn pop_frame(&mut self) -> Rp<JFrame> {
        let top = self.frames[self.size - 1];
        self.size -= 1;
        top
    }

    fn cur_frame(&self) -> Rp<JFrame> {
        self.frames[self.size - 1]
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }
}


#[derive(Debug)]
pub struct JFrame {
    pub local_vars: Vec<u64>,
    pub stack: OpStack,
    pub method: Rp<ClassMember>,
    pub class: Rp<Class>,
    pub heap: Rp<Heap>,
}

macro_rules! xx_ref {
    ($f: ident) => {
        pub fn $f(&mut self, i: usize) -> Rp<SymRef> {
            let mut cur = self.class.get_mut();
            let sym = {
                self.heap.$f(&mut cur, i)
            };
            sym
        }
    };
}

impl JFrame {
    pub fn new(heap: Rp<Heap>, class: Rp<Class>, method: Rp<ClassMember>) -> Self {
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

    pub fn new_obj(&self, class: Rp<Class>) -> Rp<Object> {
        self.heap.new_obj(class)
    }
}
