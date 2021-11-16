use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub type Np<T> = Rp<T>;

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

impl<T> Rp<T> {
    pub fn from_ref(x: &T) -> Rp<T> {
        {
            Rp {
                p: PhantomData,
                ptr: x as *const T as usize,
            }
        }
    }

    #[inline]
    pub fn from_ptr(p: usize) -> Rp<T> {
        if p == 0 {
            return Rp::null();
        }
        Rp {
            p: PhantomData,
            ptr: p,
        }
    }

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
        let b = unsafe { Box::from_raw(self.ptr as *mut T) };
        Box::leak(b)
    }

    #[inline]
    pub fn raw(&self) -> *mut T {
        self.ptr as *mut T
    }

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
