use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// unsafe raw pointer wrapper, which is also thread unsafe
// for escape compiler check
pub struct Rp<T> {
    p: PhantomData<T>,
    ptr: usize,
}

impl<T: Debug> Debug for Rp<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            f.write_str("NULL")
        } else {
            write!(f, "{:?}", self.as_ref())
        }
    }
}

impl<T> Clone for Rp<T> {
    fn clone(&self) -> Self {
        Self {
            p: PhantomData,
            ptr: self.ptr,
        }
    }
}

impl<T> Copy for Rp<T> {}

impl<T> Default for Rp<T> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

impl<T> Deref for Rp<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &'static T {
        self.get_mut()
    }
}

impl<T> DerefMut for Rp<T> {
    #[inline]
    fn deref_mut(&mut self) -> &'static mut T {
        self.get_mut()
    }
}

impl<T> AsRef<T> for Rp<T> {
    #[inline]
    fn as_ref(&self) -> &'static T {
        self.get_mut()
    }
}

// index operation for memory allocated by Rp::alloc
impl<T> std::ops::Index<usize> for Rp<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*(self.ptr as *mut T).add(index) }
    }
}

// index operation for memory allocated by Rp::alloc
impl<T> std::ops::IndexMut<usize> for Rp<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *(self.ptr as *mut T).add(index) }
    }
}

impl<T> From<&T> for Rp<T> {
    #[inline]
    fn from(x: &T) -> Self {
        {
            Rp {
                p: PhantomData,
                ptr: x as *const T as usize,
            }
        }
    }
}

impl<T> From<&mut T> for Rp<T> {
    #[inline]
    fn from(x: &mut T) -> Self {
        {
            Rp {
                p: PhantomData,
                ptr: x as *mut T as usize,
            }
        }
    }
}

impl<T> From<usize> for Rp<T> {
    #[inline]
    fn from(p: usize) -> Self {
        if p == 0 {
            return Rp::null();
        }
        Rp {
            p: PhantomData,
            ptr: p,
        }
    }
}

impl<T> From<*const T> for Rp<T> {
    #[inline]
    fn from(p: *const T) -> Self {
        if p.is_null() {
            return Rp::null();
        }
        Rp {
            p: PhantomData,
            ptr: p as usize,
        }
    }
}

impl<T> From<*mut T> for Rp<T> {
    #[inline]
    fn from(p: *mut T) -> Self {
        if p.is_null() {
            return Rp::null();
        }
        Rp {
            p: PhantomData,
            ptr: p as usize,
        }
    }
}

impl<T> Rp<T> {
    #[inline]
    pub fn is_null(&self) -> bool {
        self.ptr == 0
    }

    #[inline]
    pub fn null() -> Rp<T> {
        Rp {
            p: PhantomData,
            ptr: 0usize,
        }
    }

    // allocate on heap
    #[inline]
    pub fn new(x: T) -> Self {
        let b = Box::new(x);
        let l = Box::leak(b);
        Self {
            ptr: l as *mut T as usize,
            p: PhantomData,
        }
    }

    #[inline]
    pub fn get_mut(&self) -> &'static mut T {
        if self.is_null() {
            panic!("null pointer");
        }
        unsafe { &mut *(self.ptr as *mut T) }
    }

    #[inline]
    pub fn raw(&self) -> *mut T {
        self.ptr as *mut T
    }

    // drop is only available for Rp created by new
    #[inline]
    pub fn drop(&mut self) {
        if self.is_null() {
            return;
        }
        let b = unsafe { Box::from_raw(self.ptr as *mut T) };
        std::mem::drop(b);
        self.ptr = 0usize;
    }

    #[inline]
    pub fn ptr(&self) -> usize {
        self.ptr
    }
}

impl<T: Clone + Default> Rp<T> {
    #[inline]
    pub fn from_vec(b: Vec<T>) -> Self {
        let p = b.as_ptr() as usize;
        std::mem::forget(b);
        Self {
            ptr: p,
            p: PhantomData,
        }
    }
    // alloc a continous memory to store n struct T
    pub fn alloc(num: usize) -> Self {
        let b: Vec<T> = vec![T::default(); num];
        Self::from_vec(b)
    }

    // free a continous memory
    pub fn free(&mut self, num: usize) {
        let b = unsafe { Vec::from_raw_parts(self.ptr as *mut T, num, num) };
        self.ptr = 0;
        std::mem::drop(b);
    }
}

#[cfg(test)]
mod test {
    use super::Rp;
    use core::convert::AsRef;

    #[derive(Debug, Default, Clone)]
    pub struct Point {
        pub x: f64,
        pub y: f64,
    }

    fn main() {
        let size = 10usize;
        // new an array on heap
        let mut p: Rp<Point> = Rp::alloc(size);

        init_points(p, size);

        for i in 0..size {
            println!("{:?}", p[i]);
        }

        // free the memory, and p becomes null pointer
        p.free(size);
    }

    fn init_points(mut p: Rp<Point>, len: usize) {
        for i in 0..len {
            p[i] = Point { x: 1.0, y: 1.0 };
        }
    }
}
