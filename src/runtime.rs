use std::borrow::Borrow;
use std::{cell::RefCell, cell::Ref};
use std::rc::Rc;

use crate::heap::{ClassLoader, ClassMember};
use crate::op::OpCode;

pub struct BytesReader<'a> {
    pub bytes: &'a [u8],
    pub pc: usize,
}

macro_rules! br_un {
    ($a: ident, $w: expr) => {
       pub fn $a(&mut self) -> $a {
            let s = &self.bytes[self.pc..self.pc + $w];
            self.pc += $w;
            let mut b = [0u8; $w];
            b.copy_from_slice(s);
            $a::from_be_bytes(b)
       } 
    };
}

impl <'a> BytesReader<'a> {
    pub fn u8(&mut self) -> u8 {
        let u = self.bytes[self.pc];
        self.pc += 1;
        u
    }

    br_un!(u16, 2);
    br_un!(u32, 4);
}

// jvm runtime representation
#[derive(Debug)]
pub struct Jvm {
    loader: ClassLoader,
}


#[derive(Debug)]
pub struct JThread {
    pub pc: u32,
    pub stack: JStack,
    pub vm: Rc<RefCell<Jvm>>,
}



impl JThread {
    pub fn cur_frame(&self) -> Rc<RefCell<JFrame>> {
        self.stack.cur_frame()
    }

    pub fn run(&mut self) {
        use crate::ins::Constant;
        loop {
            let f = self.cur_frame();
            let method = {
                let b: Ref<JFrame> = RefCell::borrow(&*f);
                b.method.clone()
            };
            // let pc = f.next_pc();
            let mut rd = BytesReader {
                bytes: &method.code,
                pc: 0,
            };

            let op: OpCode = rd.u8().into();
            op.con(&mut rd, self, f);
        }
    }
}

// TODO: limit stack size
#[derive(Debug, Default)]
pub struct JStack {
    pub max_size: usize,
    pub frames: Vec<Rc<RefCell<JFrame>>>,
    pub size: usize,
}

impl JStack {
    fn push_frame(&mut self, frame: JFrame) {
        self.frames[self.size] = Rc::new(RefCell::new(frame));
        self.size += 1;
    }

    fn pop_frame(&mut self) -> Rc<RefCell<JFrame>> {
        let top = self.frames[self.size - 1].clone();
        self.size -= 1;
        top
    }

    fn cur_frame(&self) -> Rc<RefCell<JFrame>> {
        self.frames[self.size - 1].clone()
    }
}


#[derive(Debug, Default)]
pub struct JFrame {
    pub local_vars: Vec<u64>,
    pub stack: OpStack,
    pub method: Rc<ClassMember>,
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

    use super::Slots;
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
}