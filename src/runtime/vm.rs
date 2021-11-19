use crate::heap::{class::Class, class::ClassMember, class::Object, misc::Heap, misc::SymRef};
use crate::rp::Rp;
use crate::runtime::misc::{BytesReader, OpStack};
use crate::StringErr;
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
        Ok(Jvm {
            heap: p,
            thread: JThread::new(p),
        })
    }

    pub fn run_class(&mut self, c: &str) -> Result<(), StringErr> {
        // load class
        let c = self.heap.loader.load(c);

        // get main method
        let main = c.main_method();

        if main.is_null() {
            return err!("class {} has no main method", &c.name);
        }

        self.thread.stack.push_frame(JFrame::new(self.heap, main));
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
        self.next_pc = Some(self.pc + off);
    }

    pub fn revert_pc(&mut self) {
        self.next_pc = Some(self.pc);
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

    pub fn new_frame(&self, m: Rp<ClassMember>) -> JFrame {
        JFrame::new(self.heap, m)
    }

    pub fn cur_frame(&self) -> Rp<JFrame> {
        self.stack.cur_frame()
    }

    fn print_stack(&self, sep: char) {
        for i in 0..32 {
            print!("{}", sep);
        }
        print!("{}", '\n');
        println!("thread pc = {}, next_pc = {:?}", self.pc, self.next_pc);

        for i in 0..self.stack.size {
            println!(
                "frame at {} method = {} next pc = {}",
                i,
                self.stack.frames[i].as_ref().unwrap().method.name,
                self.stack.frames[i].as_ref().unwrap().next_pc,
            );
        }
    }

    pub fn run(&mut self) {
        use crate::ins::Ins;
        while !self.stack.is_empty() {
            // self.print_stack('=');
            let f = self.cur_frame();
            if f.method.access_flags.is_native() {
                self.stack.pop_frame();
                continue;
            }
            self.pc = f.next_pc;

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

            f.get_mut().next_pc = rd.pc;

            match self.next_pc {
                Some(pc) => f.get_mut().next_pc = pc,
                _ => {}
            };
            // self.print_stack('*');
        }
    }
}

// TODO: limit stack size
#[derive(Debug)]
pub struct JStack {
    pub frames: [Option<JFrame>; MAX_JSTACK_SIZE],
    pub size: usize,
}

impl JStack {
    fn new() -> Self {
        const init: Option<JFrame> = None;
        Self {
            frames: [init; MAX_JSTACK_SIZE],
            size: 0,
        }
    }

    pub fn push_frame(&mut self, frame: JFrame) {
        self.frames[self.size] = Some(frame);
        self.size += 1;
    }

    pub fn pop_frame(&mut self) {
        self.size -= 1;
    }

    pub fn prev_frame(&self) -> Rp<JFrame> {
        self.frames[self.size - 2].as_ref().unwrap().into()
    }

    fn cur_frame(&self) -> Rp<JFrame> {
        self.frames[self.size - 1].as_ref().unwrap().into()
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }
}

#[derive(Debug, Default, Clone)]
pub struct JFrame {
    pub local_vars: Vec<u64>,
    pub stack: OpStack,
    pub method: Rp<ClassMember>,
    pub class: Rp<Class>,
    pub heap: Rp<Heap>,
    pub next_pc: i32,
}

macro_rules! xx_ref {
    ($f: ident) => {
        pub fn $f(&mut self, i: usize) -> Rp<SymRef> {
            let mut cur = self.class.get_mut();
            let sym = { self.heap.$f(&mut cur, i) };
            sym
        }
    };
}

impl JFrame {
    fn new(heap: Rp<Heap>, method: Rp<ClassMember>) -> Self {
        Self {
            local_vars: vec![0u64; method.max_locals],
            stack: OpStack {
                slots: vec![0u64; method.max_stack],
                size: 0,
            },
            method,
            class: method.class,
            heap,
            next_pc: 0,
        }
    }

    xx_ref!(class_ref);
    xx_ref!(field_ref);
    xx_ref!(method_ref);
    xx_ref!(iface_ref);

    pub fn new_obj(&self, class: Rp<Class>) -> Rp<Object> {
        self.heap.new_obj(class)
    }

    pub fn pass_args(&mut self, other: &mut JFrame, arg_cells: usize) {
        let stack_data = &self.stack.slots[self.stack.size - arg_cells..self.stack.size];
        other.local_vars[..arg_cells].copy_from_slice(stack_data);
        self.stack.size -= arg_cells;
    }
}

#[cfg(test)]
mod test {
    use crate::runtime::vm::Jvm;

    #[test]
    fn test_jvm() {
        let mut jvm = Jvm::new(".:test/rt.jar").unwrap();
        jvm.run_class("test/Gauss").unwrap();
    }

    #[test]
    fn test_jvm_obj() {
        let mut jvm = Jvm::new(".:test/rt.jar").unwrap();
        jvm.run_class("test/MyObject").unwrap();
    }

    #[test]
    fn test_jvm_invoke() {
        let mut jvm = Jvm::new(".:test/rt.jar").unwrap();
        jvm.run_class("test/InvokeTest").unwrap();
    }

    #[test]
    fn test_fibo() {
        let mut jvm = Jvm::new(".:test/rt.jar").unwrap();
        jvm.run_class("test/FibonacciTest").unwrap();
    }

    #[test]
    fn test_array() {
        let mut jvm = Jvm::new(".:test/rt.jar").unwrap();
        jvm.run_class("test/MultiDimensionalArray").unwrap();
    }

    #[test]
    fn test_hello_world() {
        let mut jvm = Jvm::new(".:test/rt.jar").unwrap();
        // TODO: jni
        // jvm.run_class("test/HelloWorld").unwrap();
    }
}
