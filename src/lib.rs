//! This crate provides a `recycle` extension method for Vec.
//! It's intended to change the type of the Vec while "recycling"
//! the underlying allocation. This is a trick that is useful especially
//! when storing data with short lifetimes in Vec:
//! ```
//! # use std::error::Error;
//! # use recycle_vec::VecExt;
//! #
//! # struct Stream;
//! #
//! # impl Stream {
//! #     fn new() -> Self {
//! #         Stream
//! #     }
//! #
//! #     fn next(&mut self) -> Option<&[u8]> {
//! #         Some(&b"hoge"[..])
//! #     }
//! # }
//! #
//! # fn process(input: &[Object<'_>]) -> Result<(), Box<dyn Error>> {
//! #     Ok(())
//! # }
//! #
//! # struct Object<'a> {
//! #     reference: &'a [u8],
//! # }
//! #
//! # fn deserialize<'a>(input: &'a [u8], output: &mut Vec<Object<'a>>) -> Result<(), Box<dyn Error>> {
//! #     output.push(Object { reference: input });
//! #     Ok(())
//! # }
//! #
//! # fn processor() -> Result<(), Box<dyn Error>> {
//! #    let mut stream = Stream::new();
//! #    
//!     let mut objects: Vec<Object<'static>> = Vec::new();
//!
//!     while let Some(byte_chunk) = stream.next() { // byte_chunk only lives this scope
//!         let mut objects_temp: Vec<Object<'_>> = objects.recycle();
//!
//!         // Zero-copy parsing; Object has references to chunk
//!         deserialize(byte_chunk, &mut objects_temp)?;
//!         process(&objects_temp)?;
//!
//!         objects = objects_temp.recycle();
//!     } // byte_chunk lifetime ends
//! #
//! #    Ok(())
//! # }
//! ```
//! # Notes about safety
//! This crate uses internally `unsafe` to achieve it's functionality.
//! However, it provides a safe interface. To achieve safety, it does
//! the following precautions:
//! 1. It truncates the `Vec` to zero length, dropping all the values.
//! This ensures that no values of arbitrary types are transmuted
//! accidentally.
//! 2. It checks that the sizes and alignments of the source and target
//! types match. This ensures that the underlying block of memory backing
//! `Vec` is compatible layout-wise.
//! 3. It creates a new `Vec` value using `from_raw_parts`, instead of
//! transmuting, an operation whose soundness would be questionable.

#![no_std]

extern crate alloc;
use alloc::vec::Vec;

/// A trait that provides an API for recycling Vec's internal buffers
pub trait VecExt<T> {
    /// Allows re-interpreting the type of a Vec to reuse the allocation.
    /// The vector is emptied and any values contained in it will be dropped.
    /// The target type must have the same size and alignment as the source type.
    /// This API doesn't transmute any values of T to U, because it makes sure
    /// to empty the vector before any unsafe operations.
    ///
    /// # Panics
    /// Panics if the size or alignment of the source and target types don't match.
    /// **Note about stabilization:** This contract is enforceable at compile-time,
    /// so we'll want to wait until const asserts become stable and modify this
    /// API to cause a compile error instead of panicking before stabilizing it.
    fn recycle<U>(self) -> Vec<U>;
}

impl<T> VecExt<T> for Vec<T> {
    fn recycle<U>(mut self) -> Vec<U> {
        self.truncate(0);
        // TODO make these const asserts once it becomes possible
        assert!(core::mem::size_of::<T>() == core::mem::size_of::<U>());
        assert!(core::mem::align_of::<T>() == core::mem::align_of::<U>());
        let cap = self.capacity();
        let ptr = self.as_mut_ptr() as *mut U;
        core::mem::forget(self);
        unsafe { Vec::from_raw_parts(ptr, 0, cap) }
    }
}

/// Tests that `recycle` successfully re-interprets the type to have different lifetime from the original
#[test]
fn test_recycle_lifetime() {
    use crate::alloc::string::ToString;
    let s_1 = "foo".to_string();
    let mut buf = Vec::with_capacity(100);
    {
        let mut buf2 = buf;
        let s_2 = "foo".to_string();
        buf2.push(s_2.as_str());

        assert_eq!(buf2.len(), 1);
        assert_eq!(buf2.capacity(), 100);

        buf = buf2.recycle();
    }
    buf.push(s_1.as_str());
}

/// Tests that `recycle` successfully re-interprets the type itself
#[test]
fn test_recycle_type() {
    use crate::alloc::string::ToString;
    let s = "foo".to_string();
    let mut buf = Vec::with_capacity(100);
    {
        let mut buf2 = buf.recycle();

        let mut i = Vec::new();
        i.push(1);
        i.push(2);
        i.push(3);

        buf2.push(i.as_slice());

        assert_eq!(buf2.len(), 1);
        assert_eq!(buf2.capacity(), 100);

        buf = buf2.recycle();
    }
    buf.push(s.as_str());
}

/// Tests that `recycle` successfully panics with incompatible sizes
#[test]
#[should_panic]
fn test_recycle_incompatible_size() {
    let mut buf = Vec::with_capacity(100);
    buf.push(1_u16);
    {
        let mut buf2 = buf.recycle();
        buf2.push(1_u32);
        buf = buf2.recycle();
    }
    buf.push(1_u16);
}

/// Tests that `recycle` successfully panics with incompatible alignments
#[test]
#[should_panic]
fn test_recycle_incompatible_alignment() {
    let mut buf = Vec::with_capacity(100);
    buf.push([0_u16, 1_u16]);
    {
        let mut buf2 = buf.recycle();
        buf2.push(1_u32);
        buf = buf2.recycle();
    }
    buf.push([0_u16, 1_u16]);
}
