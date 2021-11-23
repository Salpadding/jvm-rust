use crate::heap::{class::Class, class::ClassMember, class::Object, misc::Heap, misc::SymRef};
use crate::natives::NativeRegistry;
use crate::runtime::frame::{JFrame, JStack};
use crate::runtime::misc::BytesReader;
use err::StringErr;
use rp::Rp;
use std::ops::Add;

// jvm runtime representation
pub struct Jvm {
    heap: Rp<Heap>,
    thread: JThread,
    registry: Rp<NativeRegistry>,
}

impl Jvm {
    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let heap = Heap::new(cp)?;
        let r = Rp::new(NativeRegistry::new());
        Ok(Jvm {
            registry: r,
            heap: heap,
            thread: JThread::new(heap, r),
        })
    }

    pub fn init(&mut self) {
        // 1. init java/lang/System
        // 2. init sun/misc/VM
        let mut sys = self.heap.loader.load("java/lang/System");
        sys.clinit(&mut self.thread);
        self.thread.run();

        let mut sys = self.heap.loader.load("sun/misc/VM");
        sys.clinit(&mut self.thread);
        self.thread.run();
    }

    pub fn run_class(&mut self, c: &str) -> Result<(), StringErr> {
        // load class
        let c = self.heap.loader.load(c);

        // get main method
        let main = c.main_method();

        if main.is_null() {
            return err!("class {} has no main method", &c.name);
        }

        self.thread.push_frame(main);
        self.thread.run();

        Ok(())
    }
}

pub struct JThread {
    pc: u32,
    stack: JStack,
    next_pc: Option<u32>,
    pub heap: Rp<Heap>,
    pub registry: Rp<NativeRegistry>,
}

impl JThread {
    pub fn branch(&mut self, off: i32) {
        self.next_pc = Some((self.pc as i32 + off) as u32);
    }

    pub fn revert_pc(&mut self) {
        self.next_pc = Some(self.pc);
    }
}

impl JThread {
    pub fn new(heap: Rp<Heap>, registry: Rp<NativeRegistry>) -> Self {
        Self {
            heap,
            pc: 0,
            stack: JStack::new(heap),
            next_pc: None,
            registry,
        }
    }
    #[inline]
    pub fn stack(&self) -> &JStack {
        &self.stack
    }

    #[inline]
    pub fn push_frame(&mut self, m: Rp<ClassMember>) -> Rp<JFrame> {
        self.stack.push_frame(m)
    }

    #[inline]
    pub fn back_frame(&self, i: usize) -> Rp<JFrame> {
        self.stack.back_frame(i)
    }

    #[inline]
    pub fn pop_frame(&mut self) {
        self.stack.pop_frame()
    }

    #[inline]
    pub fn prev_frame(&self) -> Rp<JFrame> {
        self.stack.prev_frame()
    }

    #[inline]
    pub fn cur_frame(&self) -> Rp<JFrame> {
        self.stack.cur_frame()
    }

    fn print_stack(&self, sep: char) {
        // for i in 0..32 {
        //     print!("{}", sep);
        // }
        // print!("{}", '\n');
        // println!("thread pc = {}, next_pc = {:?}", self.pc, self.next_pc);

        // for i in 0..self.stack.size {
        //     println!(
        //         "frame at {} method = {} next pc = {}",
        //         i,
        //         self.stack.frames[i].as_ref().unwrap().method.name,
        //         self.stack.frames[i].as_ref().unwrap().next_pc,
        //     );
        // }
    }

    pub fn run(&mut self) {
        use crate::ins::Ins;
        while !self.stack.is_empty() {
            // self.print_stack('=');
            let f = self.cur_frame();
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

    // create a new thread to invoke object
    pub fn invoke_obj(
        &mut self,
        obj: &mut Object,
        name: &str,
        desc: &str,
        args: &[u64],
        drop: bool,
    ) {
        let m = obj.class.lookup_method(name, desc);
        let mut nf = self.push_frame(m);
        nf.no_ret = drop;
        nf.local_vars().copy_from_slice(args);
    }
}

// TODO: limit stack size
#[cfg(test)]
mod test {
    use crate::runtime::vm::Jvm;

    #[test]
    fn test_jvm() {
        let mut jvm = Jvm::new(".:test/rt.jar").unwrap();
        jvm.run_class("test/Gauss").unwrap();
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
        jvm.run_class("test/Debug").unwrap();
    }
}
