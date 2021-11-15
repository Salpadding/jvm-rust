use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// unsafe raw pointer wrapper, which is also thread unsafe
// for escape compiler check
#[derive(Clone, Copy, Debug)]
pub struct Rp<T: Sized> {
    p: PhantomData<T>,
    raw: usize,
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
    #[inline]
    pub fn is_null(&self) -> bool {
        (self.raw as usize) == 0
    }

    #[inline]
    pub fn null<U>() -> Rp<U> {
        Rp {
            p: PhantomData,
            raw: 0usize,
        }
    }

    #[inline]
    pub fn new(x: T) -> Self {
        let b = Box::new(x);
        let l = Box::leak(b);
        Self {
            raw: l as *mut T as usize,
            p: PhantomData,
        }
    }

    #[inline]
    pub fn get_mut(&self) -> &'static mut T {
        let b = unsafe { Box::from_raw(self.raw as *mut T) };
        Box::leak(b)
    }

    #[inline]
    pub fn raw(&self) -> *mut T {
        self.raw as *mut T
    }

    #[inline]
    pub fn drop(&mut self) {
        let b = unsafe { Box::from_raw(self.raw as *mut T) };
        self.raw = 0usize;
    }
}
