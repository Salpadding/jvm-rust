use std::{cell::RefCell, cell::Ref};
use std::rc::Rc;
use std::sync::Arc;

use crate::StringErr;
use crate::heap::{Class, ClassLoader, ClassMember, Heap};

const MAX_JSTACK_SIZE: usize = 1024;

pub struct BytesReader<'a> {
    pub bytes: &'a [u8],
    pub pc: i32,
}

macro_rules! br_un {
    ($a: ident, $w: expr) => {
       pub fn $a(&mut self) -> $a {
            let p = self.pc as usize;
            let s = &self.bytes[p..p + $w];
            self.pc += $w;
            let mut b = [0u8; $w];
            b.copy_from_slice(s);
            $a::from_be_bytes(b)
       } 
    };
}

impl <'a> BytesReader<'a> {
    pub fn u8(&mut self) -> u8 {
        let u = self.bytes[self.pc as usize];
        self.pc += 1;
        u
    }

    br_un!(u16, 2);
    br_un!(u32, 4);

    pub fn i16(&mut self) -> i16 {
        self.u16() as i16
    }

    pub fn i32(&mut self) -> i32 {
        self.u32() as i32
    }

    pub fn skip_padding(&mut self) {
        loop {
            if self.pc % 4 != 0 {
                self.pc += 1;
            } else {
                break;
            }
        }
    }

    pub fn read_i32s(&mut self, n: usize) -> Vec<i32> {
        let mut r: Vec<i32> = Vec::with_capacity(n);
        for _ in 0..n {
            r.push(self.i32());
        }
        r
    }
}

// jvm runtime representation
#[derive(Debug)]
pub struct Jvm {
    heap: Arc<RefCell<Heap>>,
    thread: JThread,
}

impl Jvm {
    pub fn new(cp: &str) -> Result<Self, StringErr> {
        let heap = Heap::new(cp)?;
        Ok(
            Jvm {
                heap: Arc::new(RefCell::new(heap)),
                thread: JThread::new(),
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
        let main = c.main_method();
        if main.is_none() {
            return err!("class {} has no main method", c.name);
        }

        self.thread.stack.push_frame(JFrame::from_method(c, main.unwrap()));
        self.thread.run();

        Ok(())
    }
}


#[derive(Debug)]
pub struct JThread {
    pub pc: i32,
    pub stack: JStack,
    pub next_pc: Option<i32>,
}

impl JThread {
    pub fn branch(&mut self, off: i32) {
        self.next_pc = Some(off);
    }
}

impl JThread {
    pub fn new() -> Self {
        Self {
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


#[derive(Debug, Default)]
pub struct JFrame {
    pub local_vars: Vec<u64>,
    pub stack: OpStack,
    pub method: Arc<ClassMember>,
    pub class: Arc<Class>,
}

impl JFrame {
    pub fn from_method(c: Arc<Class>, m: Arc<ClassMember>) -> Self {
        Self {
            local_vars: vec![0u64; m.max_locals],
            stack: OpStack { slots: vec![0u64; m.max_stack], size: 0 },
            method: m.clone(),
            class: c,
        }
    }
}


#[derive(Debug, Default)]
pub struct OpStack {
    pub slots: Vec<u64>,
    pub size: usize,
}

impl OpStack {
   pub fn push_u32(&mut self, v: u32) {
        self.slots.set_u32(self.size, v);
        self.size += 1;
   } 

   pub fn push_i32(&mut self, v: i32) {
       self.push_u32(v as u32);
   } 

   pub fn push_u64(&mut self, v: u64) {
       self.slots.set_u64(self.size, v);
       self.size += 2;
   } 

   pub fn push_i64(&mut self, v: i64) {
       self.push_u64(v as u64)
   } 

   pub fn push_f32(&mut self, v: f32) {
       self.push_u32(v.to_bits());
   }


   pub fn push_f64(&mut self, v: f64) {
       self.push_u64(v.to_bits());
   }

   pub fn pop_u32(&mut self) -> u32 {
        let r = self.slots.get_u32(self.size - 1);
        self.size -= 1;
        r
   } 

   pub fn pop_i32(&mut self) -> i32 {
       self.pop_u32() as i32
   } 

   pub fn pop_u64(&mut self) -> u64 {
        let r = self.slots.get_u64(self.size - 2);
        self.size -= 2;
        r
   } 

   pub fn pop_i64(&mut self) -> i64 {
       self.pop_u64() as i64
   } 

   pub fn pop_f32(&mut self) -> f32 {
       f32::from_bits(self.pop_u32())
   }

   pub fn pop_f64(&mut self) -> f64 {
       f64::from_bits(self.pop_u64())
   }

   pub fn push_nil(&mut self) {
       self.slots[self.size] = 0;
       self.size += 1;
   }

   pub fn push_cell(&mut self, v: u64) {
       self.slots[self.size] = v;
       self.size += 1;
   }

   pub fn pop_cell(&mut self) -> u64 {
       let r = self.slots[self.size - 1];
       self.size -= 1;
       r
   }
}

// Each frame (§2.6) contains an array of variables known as its local variables. The length of the local variable array of a frame is determined at compile-time and supplied in the binary representation of a class or interface along with the code for the method associated with the frame (§4.7.3).
// A single local variable can hold a value of type boolean, byte, char, short, int, float, reference, or returnAddress. A pair of local variables can hold a value of type long or double.
// Local variables are addressed by indexing. The index of the first local variable is zero. An integer is considered to be an index into the local variable array if and only if that integer is between zero and one less than the size of the local variable array.
// A value of type long or type double occupies two consecutive local variables. Such a value may only be addressed using the lesser index. For example, a value of type double stored in the local variable array at index n actually occupies the local variables with indices n and n+1; however, the local variable at index n+1 cannot be loaded from. It can be stored into. However, doing so invalidates the contents of local variable n.
// The Java Virtual Machine does not require n to be even. In intuitive terms, values of types long and double need not be 64-bit aligned in the local variables array. Implementors are free to decide the appropriate way to represent such values using the two local variables reserved for the value.
// The Java Virtual Machine uses local variables to pass parameters on method invocation. On class method invocation, any parameters are passed in consecutive local variables starting from local variable 0. On instance m
pub trait Slots {
    fn set_u32(&mut self, i: usize, v: u32);
    fn get_u32(&self, i: usize) -> u32;
    
    fn set_i32(&mut self, i: usize, v: i32) ;

    fn get_i32(&self, i: usize) -> i32;

    fn get_f32(&self, i: usize) -> f32 {
        f32::from_bits(self.get_u32(i))
    }

    fn set_u64(&mut self, i: usize, v: u64) {
        self.set_u32(i, v as u32);
        self.set_u32(i + 1, (v >> 32) as u32);
    }

    fn get_u64(&self, i: usize) -> u64 {
        let low = self.get_u32(i);
        let high = self.get_u32(i + 1);
        ((high as u64) << 32) | (low as u64)
    }

    fn set_f32(&mut self, i: usize, v: f32) {
        self.set_u32(i, v.to_bits());
    }

    fn set_f64(&mut self, i: usize, v: f64) {
        self.set_u64(i, v.to_bits());
    }
    
    fn get_f64(&self, i: usize) -> f64{
        f64::from_bits(self.get_u64(i))
    }

    fn get_cell(&self, i: usize) -> u64;
    fn set_cell(&mut self, i: usize, v: u64);
}

impl Slots for Vec<u64> {
    fn set_u32(&mut self, i: usize, v: u32) {
        self[i] = v as u64;
    }

    fn get_u32(&self, i: usize) -> u32 {
        self[i] as u32
    }

    fn set_i32(&mut self, i: usize, v: i32) {
        self[i] = v as u32 as u64;
    }

    fn get_i32(&self, i: usize) -> i32 {
        self[i] as u32 as i32
    }


    fn get_cell(&self, i: usize) -> u64 {
        self[i]
    }

    fn set_cell(&mut self, i: usize, v: u64) {
        self[i] = v;
    }
}

#[cfg(test)]
mod test {
    use crate::runtime::OpStack;

    use super::{Jvm, Slots};
    #[test]
    fn test_local_vars() {
        let mut v: Vec<u64> = vec![0u64; 32];
        v.set_u32(0, 100);
        v.set_u32(1, -100i32 as u32);
        v.set_u64(2, 2997924580u64);
        v.set_u64(4, -2997924580i64 as u64);
        v.set_f32(6, 3.1415926f32);
        v.set_f64(7, 2.71828182845);

        assert_eq!(v.get_u32(0), 100u32);
        assert_eq!(v.get_u32(1), -100i32 as u32);
        assert_eq!(v.get_u64(2), 2997924580u64);
        assert_eq!(v.get_u64(4), -2997924580i64 as u64);
        assert_eq!(v.get_f32(6), 3.1415926f32);
        assert_eq!(v.get_f64(7), 2.71828182845f64);
    }

    #[test]
    fn test_op_stack() {
        let v: Vec<u64> = vec![0u64; 32];
        let mut s = OpStack {
            slots: v,
            size: 0,
        };

        s.push_u32(100);
        s.push_u32(-100i32 as u32);
        s.push_u64(2997924580u64);
        s.push_u64(-2997924580i64 as u64);
        s.push_f32(3.1415926f32);
        s.push_f64(2.71828182845f64);

        assert_eq!(s.pop_f64(), 2.71828182845f64);
        assert_eq!(s.pop_f32(), 3.1415926f32);
        assert_eq!(s.pop_u64(), -2997924580i64 as u64);
        assert_eq!(s.pop_u64(), 2997924580u64);
        assert_eq!(s.pop_u32(), -100i32 as u32);
        assert_eq!(s.pop_u32(), 100);
    }

    #[test]
    fn test_jvm() {
        let mut jvm = Jvm::new(".:test/rt.jar").unwrap();
        jvm.run_class("test/Gauss").unwrap();
    }
}
