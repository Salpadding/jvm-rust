use std::ops::Add;

use crate::heap::{class::Class, class::ClassMember, class::Object, misc::Heap, misc::SymRef};
use crate::natives::NativeRegistry;
use crate::runtime::frame::JFrame;
use crate::runtime::misc::BytesReader;
use err::StringErr;
use rp::Rp;
const MAX_JSTACK_SIZE: usize = 1024;

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

        self.thread.stack.push_frame(self.thread.new_frame(main));
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
    pub frame_id: Rp<u32>,
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
            stack: JStack::new(),
            next_pc: None,
            registry,
            frame_id: Rp::new(0),
        }
    }
    #[inline]
    pub fn stack(&self) -> &JStack {
        &self.stack
    }

    #[inline]
    pub fn push_frame(&mut self, frame: JFrame) {
        self.stack.push_frame(frame)
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

    pub fn new_frame(&self, m: Rp<ClassMember>) -> JFrame {
        let id = *self.frame_id;
        // println!("create frame {}.{} id = {}", m.class.name, m.name, id);
        (*self.frame_id.get_mut()) = id + 1;
        self.stack.new_frame(id as u16, self.heap, m)
    }

    #[inline]
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
        let mut nf = self.new_frame(m);
        nf.drop = drop;
        nf.local_vars().copy_from_slice(args);
        self.stack.push_frame(nf);
    }
}

// TODO: limit stack size
pub struct JStack {
    frames: [Option<JFrame>; MAX_JSTACK_SIZE],
    // default stack size = 64k = 64 * 1024
    stack_data: Vec<u64>,
    size: usize,
}

impl JStack {
    fn new() -> Self {
        const init: Option<JFrame> = None;
        Self {
            frames: [init; MAX_JSTACK_SIZE],
            size: 0,
            stack_data: vec![0; 64 * 1024 / 8],
        }
    }

    fn new_frame(&self, id: u16, heap: Rp<Heap>, m: Rp<ClassMember>) -> JFrame {
        let mut f = JFrame::new(id, heap, m);

        // TODO: use realloc to allocate memory
        // assign stack base
        // new stack base = prev stack base + prev max stack
        let new_base: Rp<u64> = if self.is_empty() {
            self.stack_data.as_ptr().into()
        } else {
            let cur = self.cur_frame();
            unsafe { cur.stack_base.raw().add(cur.max_stack()).into() }
        };

        f.local_base = new_base;
        f.stack_base = unsafe { f.local_base.raw().add(f.max_locals()).into() };
        f
    }

    #[inline]
    fn push_frame(&mut self, mut frame: JFrame) {
        self.frames[self.size] = Some(frame);
        self.size += 1;
    }

    #[inline]
    fn pop_frame(&mut self) {
        self.size -= 1;
    }

    #[inline]
    fn back_frame(&self, i: usize) -> Rp<JFrame> {
        self.frames[self.size - i].as_ref().unwrap().into()
    }

    #[inline]
    fn prev_frame(&self) -> Rp<JFrame> {
        self.frames[self.size - 2].as_ref().unwrap().into()
    }

    #[inline]
    fn cur_frame(&self) -> Rp<JFrame> {
        self.frames[self.size - 1].as_ref().unwrap().into()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
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
