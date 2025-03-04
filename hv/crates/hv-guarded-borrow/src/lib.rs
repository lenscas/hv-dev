//! Heavy Guarded-Borrow - traits for generalizing over guarded borrow operations
//!
//! Using these traits allows you to write code which generalizes over the type of "guard" that a
//! type like `Mutex`, `RwLock`, `RefCell`, `AtomicRefCell`, etc. may return.

#![feature(generic_associated_types)]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, sync::Arc};
use core::ops::{Deref, DerefMut};
use core::{
    cell::{BorrowError, BorrowMutError, Ref, RefCell, RefMut},
    convert::Infallible,
};

#[cfg(feature = "hv-ecs")]
mod hv_ecs;

#[cfg(feature = "std")]
mod std;

/// Abstracts over non-blocking guarded immutable borrows (for example, `RefCell::try_borrow`.)
pub trait NonBlockingGuardedBorrow<T: ?Sized> {
    /// The guard type (for example, `std::cell::Ref<'a, T>`.)
    type Guard<'a>: Deref<Target = T>
    where
        T: 'a,
        Self: 'a;
    /// The type returned in the case the value cannot be borrowed.
    type BorrowError<'a>
    where
        T: 'a,
        Self: 'a;

    /// Attempt to perform the borrow.
    fn try_nonblocking_guarded_borrow(&self) -> Result<Self::Guard<'_>, Self::BorrowError<'_>>;
}

/// Abstracts over non-blocking guarded mutable borrows from behind immutable references
/// (for example, `RefCell::try_borrow_mut`.)
pub trait NonBlockingGuardedBorrowMut<T: ?Sized> {
    /// The guard type (for example, `std::cell::RefMut<'a, T>`.)
    type GuardMut<'a>: Deref<Target = T> + DerefMut
    where
        T: 'a,
        Self: 'a;
    /// The type returned in the case the value cannot be borrowed.
    type BorrowMutError<'a>
    where
        T: 'a,
        Self: 'a;

    /// Attempt to perform the borrow.
    fn try_nonblocking_guarded_borrow_mut(
        &self,
    ) -> Result<Self::GuardMut<'_>, Self::BorrowMutError<'_>>;
}

/// Abstracts over non-blocking guarded mutable borrows from behind mutable references
/// (for example, `RefCell::get_mut`, or calling `.write()` on an `&mut Arc<RwLock<T>>`.)
pub trait NonBlockingGuardedMutBorrowMut<T: ?Sized> {
    /// The guard type (for example, `std::sync::RwLockWriteGuard<'a, T>`.)
    type MutGuardMut<'a>: Deref<Target = T> + DerefMut
    where
        T: 'a,
        Self: 'a;
    /// The type returned in the case the value cannot be borrowed.
    type MutBorrowMutError<'a>
    where
        T: 'a,
        Self: 'a;

    /// Attempt to perform the borrow.
    fn try_nonblocking_guarded_mut_borrow_mut(
        &mut self,
    ) -> Result<Self::MutGuardMut<'_>, Self::MutBorrowMutError<'_>>;
}

impl<'a, T: ?Sized> NonBlockingGuardedBorrow<T> for &'a T {
    type Guard<'b>
    where
        T: 'b,
        Self: 'b,
    = &'b T;
    type BorrowError<'b>
    where
        T: 'b,
        Self: 'b,
    = Infallible;

    fn try_nonblocking_guarded_borrow(&self) -> Result<Self::Guard<'_>, Self::BorrowError<'_>> {
        Ok(self)
    }
}

impl<'a, T: ?Sized> NonBlockingGuardedBorrowMut<T> for &'a T {
    type GuardMut<'b>
    where
        T: 'b,
        Self: 'b,
    = &'b mut T;
    type BorrowMutError<'b>
    where
        T: 'b,
        Self: 'b,
    = &'static str;

    fn try_nonblocking_guarded_borrow_mut(
        &self,
    ) -> Result<Self::GuardMut<'_>, Self::BorrowMutError<'_>> {
        Err("cannot mutably borrow from behind a shared reference")
    }
}

impl<'a, T: ?Sized> NonBlockingGuardedMutBorrowMut<T> for &'a T {
    type MutGuardMut<'b>
    where
        T: 'b,
        Self: 'b,
    = &'b mut T;
    type MutBorrowMutError<'b>
    where
        T: 'b,
        Self: 'b,
    = &'static str;

    fn try_nonblocking_guarded_mut_borrow_mut(
        &mut self,
    ) -> Result<Self::MutGuardMut<'_>, Self::MutBorrowMutError<'_>> {
        Err("cannot mutably borrow from behind a shared reference")
    }
}

impl<'a, T: ?Sized> NonBlockingGuardedBorrow<T> for &'a mut T {
    type Guard<'b>
    where
        T: 'b,
        Self: 'b,
    = &'b T;
    type BorrowError<'b>
    where
        T: 'b,
        Self: 'b,
    = Infallible;

    fn try_nonblocking_guarded_borrow(&self) -> Result<Self::Guard<'_>, Self::BorrowError<'_>> {
        Ok(self)
    }
}

impl<'a, T: ?Sized> NonBlockingGuardedBorrowMut<T> for &'a mut T {
    type GuardMut<'b>
    where
        T: 'b,
        Self: 'b,
    = &'b mut T;
    type BorrowMutError<'b>
    where
        T: 'b,
        Self: 'b,
    = &'static str;

    fn try_nonblocking_guarded_borrow_mut(
        &self,
    ) -> Result<Self::GuardMut<'_>, Self::BorrowMutError<'_>> {
        Err("cannot mutably borrow from behind a shared reference")
    }
}

impl<'a, T: ?Sized> NonBlockingGuardedMutBorrowMut<T> for &'a mut T {
    type MutGuardMut<'b>
    where
        T: 'b,
        Self: 'b,
    = &'b mut T;
    type MutBorrowMutError<'b>
    where
        T: 'b,
        Self: 'b,
    = Infallible;

    fn try_nonblocking_guarded_mut_borrow_mut(
        &mut self,
    ) -> Result<Self::MutGuardMut<'_>, Self::MutBorrowMutError<'_>> {
        Ok(self)
    }
}

impl<T: ?Sized> NonBlockingGuardedBorrow<T> for RefCell<T> {
    type Guard<'a>
    where
        T: 'a,
    = Ref<'a, T>;
    type BorrowError<'a>
    where
        T: 'a,
    = BorrowError;

    fn try_nonblocking_guarded_borrow(&self) -> Result<Self::Guard<'_>, Self::BorrowError<'_>> {
        self.try_borrow()
    }
}

impl<T: ?Sized> NonBlockingGuardedBorrowMut<T> for RefCell<T> {
    type GuardMut<'a>
    where
        T: 'a,
    = RefMut<'a, T>;
    type BorrowMutError<'a>
    where
        T: 'a,
    = BorrowMutError;

    fn try_nonblocking_guarded_borrow_mut(
        &self,
    ) -> Result<Self::GuardMut<'_>, Self::BorrowMutError<'_>> {
        self.try_borrow_mut()
    }
}

impl<T: ?Sized> NonBlockingGuardedMutBorrowMut<T> for RefCell<T> {
    type MutGuardMut<'a>
    where
        T: 'a,
    = &'a mut T;
    type MutBorrowMutError<'a>
    where
        T: 'a,
    = Infallible;

    fn try_nonblocking_guarded_mut_borrow_mut(
        &mut self,
    ) -> Result<Self::MutGuardMut<'_>, Self::MutBorrowMutError<'_>> {
        Ok(self.get_mut())
    }
}

impl<T: ?Sized, U: ?Sized> NonBlockingGuardedBorrow<U> for Rc<T>
where
    T: NonBlockingGuardedBorrow<U>,
{
    type Guard<'a>
    where
        U: 'a,
        Self: 'a,
    = T::Guard<'a>;
    type BorrowError<'a>
    where
        U: 'a,
        Self: 'a,
    = T::BorrowError<'a>;

    fn try_nonblocking_guarded_borrow(&self) -> Result<Self::Guard<'_>, Self::BorrowError<'_>> {
        T::try_nonblocking_guarded_borrow(self)
    }
}

impl<T: ?Sized, U: ?Sized> NonBlockingGuardedBorrowMut<U> for Rc<T>
where
    T: NonBlockingGuardedBorrowMut<U>,
{
    type GuardMut<'a>
    where
        U: 'a,
        Self: 'a,
    = T::GuardMut<'a>;
    type BorrowMutError<'a>
    where
        U: 'a,
        Self: 'a,
    = T::BorrowMutError<'a>;

    fn try_nonblocking_guarded_borrow_mut(
        &self,
    ) -> Result<Self::GuardMut<'_>, Self::BorrowMutError<'_>> {
        T::try_nonblocking_guarded_borrow_mut(self)
    }
}

impl<T: ?Sized, U: ?Sized> NonBlockingGuardedMutBorrowMut<U> for Rc<T>
where
    T: NonBlockingGuardedBorrowMut<U>,
{
    type MutGuardMut<'a>
    where
        U: 'a,
        Self: 'a,
    = T::GuardMut<'a>;
    type MutBorrowMutError<'a>
    where
        U: 'a,
        Self: 'a,
    = T::BorrowMutError<'a>;

    fn try_nonblocking_guarded_mut_borrow_mut(
        &mut self,
    ) -> Result<Self::MutGuardMut<'_>, Self::MutBorrowMutError<'_>> {
        T::try_nonblocking_guarded_borrow_mut(self)
    }
}

impl<T: ?Sized, U: ?Sized> NonBlockingGuardedBorrow<U> for Arc<T>
where
    T: NonBlockingGuardedBorrow<U>,
{
    type Guard<'a>
    where
        U: 'a,
        Self: 'a,
    = T::Guard<'a>;
    type BorrowError<'a>
    where
        U: 'a,
        Self: 'a,
    = T::BorrowError<'a>;

    fn try_nonblocking_guarded_borrow(&self) -> Result<Self::Guard<'_>, Self::BorrowError<'_>> {
        T::try_nonblocking_guarded_borrow(self)
    }
}

impl<T: ?Sized, U: ?Sized> NonBlockingGuardedBorrowMut<U> for Arc<T>
where
    T: NonBlockingGuardedBorrowMut<U>,
{
    type GuardMut<'a>
    where
        U: 'a,
        Self: 'a,
    = T::GuardMut<'a>;
    type BorrowMutError<'a>
    where
        U: 'a,
        Self: 'a,
    = T::BorrowMutError<'a>;

    fn try_nonblocking_guarded_borrow_mut(
        &self,
    ) -> Result<Self::GuardMut<'_>, Self::BorrowMutError<'_>> {
        T::try_nonblocking_guarded_borrow_mut(self)
    }
}

impl<T: ?Sized, U: ?Sized> NonBlockingGuardedMutBorrowMut<U> for Arc<T>
where
    T: NonBlockingGuardedBorrowMut<U>,
{
    type MutGuardMut<'a>
    where
        U: 'a,
        Self: 'a,
    = T::GuardMut<'a>;
    type MutBorrowMutError<'a>
    where
        U: 'a,
        Self: 'a,
    = T::BorrowMutError<'a>;

    fn try_nonblocking_guarded_mut_borrow_mut(
        &mut self,
    ) -> Result<Self::MutGuardMut<'_>, Self::MutBorrowMutError<'_>> {
        T::try_nonblocking_guarded_borrow_mut(self)
    }
}

impl<T: ?Sized, U: ?Sized> NonBlockingGuardedBorrow<U> for Box<T>
where
    T: NonBlockingGuardedBorrow<U>,
{
    type Guard<'a>
    where
        U: 'a,
        Self: 'a,
    = T::Guard<'a>;
    type BorrowError<'a>
    where
        U: 'a,
        Self: 'a,
    = T::BorrowError<'a>;

    fn try_nonblocking_guarded_borrow(&self) -> Result<Self::Guard<'_>, Self::BorrowError<'_>> {
        T::try_nonblocking_guarded_borrow(self)
    }
}

impl<T: ?Sized, U: ?Sized> NonBlockingGuardedBorrowMut<U> for Box<T>
where
    T: NonBlockingGuardedBorrowMut<U>,
{
    type GuardMut<'a>
    where
        U: 'a,
        Self: 'a,
    = T::GuardMut<'a>;
    type BorrowMutError<'a>
    where
        U: 'a,
        Self: 'a,
    = T::BorrowMutError<'a>;

    fn try_nonblocking_guarded_borrow_mut(
        &self,
    ) -> Result<Self::GuardMut<'_>, Self::BorrowMutError<'_>> {
        T::try_nonblocking_guarded_borrow_mut(self)
    }
}

impl<T: ?Sized, U: ?Sized> NonBlockingGuardedMutBorrowMut<U> for Box<T>
where
    T: NonBlockingGuardedMutBorrowMut<U>,
{
    type MutGuardMut<'a>
    where
        U: 'a,
        Self: 'a,
    = T::MutGuardMut<'a>;
    type MutBorrowMutError<'a>
    where
        U: 'a,
        Self: 'a,
    = T::MutBorrowMutError<'a>;

    fn try_nonblocking_guarded_mut_borrow_mut(
        &mut self,
    ) -> Result<Self::MutGuardMut<'_>, Self::MutBorrowMutError<'_>> {
        T::try_nonblocking_guarded_mut_borrow_mut(self)
    }
}
