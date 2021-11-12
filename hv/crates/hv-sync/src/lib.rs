#![feature(const_fn_trait_bound)]
#![feature(generic_associated_types)]
#![feature(result_into_ok_or_err)]
#![feature(slice_ptr_get, slice_ptr_len)]
#![no_std]

extern crate alloc;

pub extern crate hv_atom as atom;
pub extern crate hv_cell as cell;
pub extern crate hv_elastic as elastic;

pub mod borrow;
pub mod capability;

/// A wrapper type which only allows `&mut` access to the inner value, therefore making it
/// unconditionally `Sync`.
pub struct NoSharedAccess<T>(T);

unsafe impl<T: Send> Send for NoSharedAccess<T> {}
unsafe impl<T> Sync for NoSharedAccess<T> {}

impl<T> NoSharedAccess<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn get(&mut self) -> &T {
        &self.0
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}
