use crate::heap::class::{Class, ClassMember, Object};
use crate::heap::misc::{Heap, SymRef};
use rp::Rp;

#[derive(Default, Clone)]
pub struct JFrame {
    pub local_base: Rp<u64>,
    pub method: Rp<ClassMember>,
    pub heap: Rp<Heap>,
    pub next_pc: u32,
    pub id: u16,
    pub drop: bool,
    pub stack_base: Rp<u64>,
    pub stack_size: u16,
}

macro_rules! xx_ref {
    ($f: ident) => {
        pub fn $f(&mut self, i: usize) -> Rp<SymRef> {
            let mut cur = self.class().get_mut();
            let sym = { self.heap.$f(&mut cur, i) };
            sym
        }
    };
}

impl JFrame {
    #[inline]
    pub fn pop_slots(&mut self, i: usize) -> &'static [u64] {
        let r = &self.slots()[self.stack_size as usize - i..self.stack_size as usize];
        self.stack_size -= i as u16;
        r
    }

    #[inline]
    pub fn local_vars(&self) -> &'static mut [u64] {
        self.local_base.as_slice(self.max_locals())
    }

    #[inline]
    pub fn slots(&self) -> &'static mut [u64] {
        self.stack_base.as_slice(self.max_stack())
    }

    #[inline]
    pub fn stack_size(&self) -> u16 {
        self.stack_size
    }

    #[inline]
    pub fn max_locals(&self) -> usize {
        self.method.max_locals
    }

    #[inline]
    pub fn max_stack(&self) -> usize {
        self.method.max_stack
    }

    #[inline]
    pub fn class(&self) -> Rp<Class> {
        self.method.class
    }

    #[inline]
    pub fn this(&self) -> Rp<Object> {
        (self.local_vars()[0] as usize).into()
    }

    pub fn new(id: u16, heap: Rp<Heap>, method: Rp<ClassMember>) -> Self {
        Self {
            local_base: Rp::null(),
            method,
            heap,
            next_pc: 0,
            id,
            drop: false,
            stack_base: Rp::null(),
            stack_size: 0,
        }
    }

    xx_ref!(class_ref);
    xx_ref!(field_ref);
    xx_ref!(method_ref);
    xx_ref!(iface_ref);

    pub fn pass_args(&mut self, other: &mut JFrame, arg_cells: usize) {
        let stack_data =
            &self.slots()[self.stack_size as usize - arg_cells..self.stack_size as usize];
        other.local_vars()[..arg_cells].copy_from_slice(stack_data);
        self.stack_size -= arg_cells as u16;
    }
}

macro_rules! pop_x {
    ($f: ident, $t: ty) => {
        #[inline]
        pub fn $f(&mut self) -> $t {
            self.pop_u32() as $t
        }
    };
}

macro_rules! pop_xx {
    ($f: ident, $t: ty) => {
        #[inline]
        pub fn $f(&mut self) -> $t {
            self.pop_u64() as $t
        }
    };
}

macro_rules! push_x {
    ($f: ident, $t: ty) => {
        #[inline]
        pub fn $f(&mut self, v: $t) {
            self.push_u32(v as u32);
        }
    };
}

macro_rules! push_xx {
    ($f: ident, $t: ty) => {
        #[inline]
        pub fn $f(&mut self, v: $t) {
            self.push_u64(v as u64)
        }
    };
}

impl JFrame {
    #[inline]
    pub fn push_u32(&mut self, v: u32) {
        self.stack_base[self.stack_size as usize] = v as u64;
        self.stack_size += 1;
    }

    push_x!(push_u8, u8);
    push_x!(push_u16, u16);
    push_x!(push_i32, i32);

    #[inline]
    pub fn push_u64(&mut self, v: u64) {
        self.stack_base[self.stack_size as usize] = v & 0xffffffff;
        self.stack_base[self.stack_size as usize + 1] = v >> 32;
        self.stack_size += 2;
    }

    push_xx!(push_i64, i64);

    #[inline]
    pub fn push_f32(&mut self, v: f32) {
        self.push_u32(v.to_bits());
    }

    #[inline]
    pub fn push_f64(&mut self, v: f64) {
        self.push_u64(v.to_bits());
    }

    #[inline]
    pub fn pop_u32(&mut self) -> u32 {
        let r = self.slots().get_u32(self.stack_size as usize - 1);
        self.stack_size -= 1;
        r
    }

    pop_x!(pop_u8, u8);
    pop_x!(pop_u16, u16);
    pop_x!(pop_i32, i32);

    #[inline]
    pub fn pop_u64(&mut self) -> u64 {
        let low = self.stack_base[self.stack_size as usize - 2];
        let high = self.stack_base[self.stack_size as usize - 1];
        self.stack_size -= 2;
        high << 32 | low
    }

    pop_xx!(pop_i64, i64);

    #[inline]
    pub fn pop_f32(&mut self) -> f32 {
        f32::from_bits(self.pop_u32())
    }

    #[inline]
    pub fn pop_f64(&mut self) -> f64 {
        f64::from_bits(self.pop_u64())
    }

    #[inline]
    pub fn push_null(&mut self) {
        self.slots()[self.stack_size as usize] = 0;
        self.stack_size += 1;
    }

    #[inline]
    pub fn push_slot(&mut self, v: u64) {
        self.slots()[self.stack_size as usize] = v;
        self.stack_size += 1;
    }

    #[inline]
    pub fn pop_slot(&mut self) -> u64 {
        let r = self.slots()[self.stack_size as usize - 1];
        self.stack_size -= 1;
        r
    }

    #[inline]
    pub fn pop_obj(&mut self) -> Rp<Object> {
        (self.pop_slot() as usize).into()
    }

    #[inline]
    pub fn push_obj(&mut self, obj: Rp<Object>) {
        self.push_slot(obj.ptr() as u64)
    }

    #[inline]
    pub fn back_obj(&self, i: usize) -> Rp<Object> {
        (self.slots()[self.stack_size as usize - i] as usize).into()
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
    fn set_i32(&mut self, i: usize, v: i32);
    fn get_i32(&self, i: usize) -> i32;

    #[inline]
    fn get_f32(&self, i: usize) -> f32 {
        f32::from_bits(self.get_u32(i))
    }

    #[inline]
    fn set_u64(&mut self, i: usize, v: u64) {
        self.set_u32(i, v as u32);
        self.set_u32(i + 1, (v >> 32) as u32);
    }

    #[inline]
    fn get_u64(&self, i: usize) -> u64 {
        let low = self.get_u32(i);
        let high = self.get_u32(i + 1);
        ((high as u64) << 32) | (low as u64)
    }

    #[inline]
    fn get_i64(&self, i: usize) -> i64 {
        self.get_u64(i) as i64
    }

    #[inline]
    fn set_f32(&mut self, i: usize, v: f32) {
        self.set_u32(i, v.to_bits());
    }

    #[inline]
    fn set_f64(&mut self, i: usize, v: f64) {
        self.set_u64(i, v.to_bits());
    }

    #[inline]
    fn get_f64(&self, i: usize) -> f64 {
        f64::from_bits(self.get_u64(i))
    }

    fn get_slot(&self, i: usize) -> u64;
    fn set_slot(&mut self, i: usize, v: u64);
}

impl Slots for [u64] {
    #[inline]
    fn set_u32(&mut self, i: usize, v: u32) {
        self[i] = v as u64;
    }

    #[inline]
    fn get_u32(&self, i: usize) -> u32 {
        self[i] as u32
    }

    #[inline]
    fn set_i32(&mut self, i: usize, v: i32) {
        self[i] = v as u32 as u64;
    }

    #[inline]
    fn get_i32(&self, i: usize) -> i32 {
        self[i] as u32 as i32
    }

    #[inline]
    fn get_slot(&self, i: usize) -> u64 {
        self[i]
    }

    #[inline]
    fn set_slot(&mut self, i: usize, v: u64) {
        self[i] = v;
    }
}

pub(crate) trait DupStack {
    fn dup(&mut self);
    fn dup2(&mut self);
    fn dup_x1(&mut self);
    fn dup_x2(&mut self);
    fn dup2_x1(&mut self);
    fn dup2_x2(&mut self);
    fn swap(&mut self);
}

impl DupStack for JFrame {
    fn dup(&mut self) {
        let top = { self.slots()[self.stack_size as usize - 1] };
        self.push_slot(top);
    }

    fn dup2(&mut self) {
        let s = self.slots();
        let (v2, v1) = {
            (
                self.slots()[self.stack_size as usize - 2],
                self.slots()[self.stack_size as usize - 1],
            )
        };
        self.slots()[self.stack_size as usize] = v2;
        self.slots()[self.stack_size as usize + 1] = v1;
        self.stack_size += 2;
    }

    fn dup_x1(&mut self) {
        let v1 = self.slots()[self.stack_size as usize - 1];
        let v2 = self.slots()[self.stack_size as usize - 2];
        self.slots()[self.stack_size as usize - 1] = v2;
        self.slots()[self.stack_size as usize - 2] = v1;
        self.push_slot(v1);
    }

    fn dup_x2(&mut self) {
        let v1 = self.slots()[self.stack_size as usize - 1];
        let v2 = self.slots()[self.stack_size as usize - 2];
        let v3 = self.slots()[self.stack_size as usize - 3];
        self.slots()[self.stack_size as usize - 1] = v2;
        self.slots()[self.stack_size as usize - 2] = v3;
        self.slots()[self.stack_size as usize - 3] = v1;
        self.push_slot(v1);
    }

    fn dup2_x1(&mut self) {
        let v1 = self.slots()[self.stack_size as usize - 1];
        let v2 = self.slots()[self.stack_size as usize - 2];
        let v3 = self.slots()[self.stack_size as usize - 3];
        self.slots()[self.stack_size as usize - 1] = v3;
        self.slots()[self.stack_size as usize - 2] = v1;
        self.slots()[self.stack_size as usize - 3] = v2;
        self.slots()[self.stack_size as usize] = v2;
        self.slots()[self.stack_size as usize + 1] = v1;
        self.stack_size += 2;
    }

    fn dup2_x2(&mut self) {
        let v1 = self.slots()[self.stack_size as usize - 1];
        let v2 = self.slots()[self.stack_size as usize - 2];
        let v3 = self.slots()[self.stack_size as usize - 3];
        let v4 = self.slots()[self.stack_size as usize - 4];
        self.slots()[self.stack_size as usize - 1] = v3;
        self.slots()[self.stack_size as usize - 2] = v4;
        self.slots()[self.stack_size as usize - 3] = v1;
        self.slots()[self.stack_size as usize - 4] = v2;
        self.slots()[self.stack_size as usize] = v2;
        self.slots()[self.stack_size as usize + 1] = v1;
        self.stack_size += 2;
    }

    fn swap(&mut self) {
        let v1 = self.slots()[self.stack_size as usize - 1];
        let v2 = self.slots()[self.stack_size as usize - 2];
        self.slots()[self.stack_size as usize - 1] = v2;
        self.slots()[self.stack_size as usize - 2] = v1;
    }
}
