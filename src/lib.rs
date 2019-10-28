/// A trait that provides an API for recycling Vec's internal buffers
trait VecExt<T> {

	/// Allows re-interpret the type of a Vec to reuse the allocation.
	/// The vector is emptied in and any values contained in it will be dropped.
	/// The target type must have the same size and alignment as the source type.
	/// This API doesn't transmute any values of T to U, because it makes sure
	/// to empty the vector before any unsafe operations.
	/// 
	/// # Panics
	/// Panics if the size or alignment of the source and target types don't match.
	/// **Note about stabilization:** This contract is enforceable at compile-time,
	/// so we'll want to wait until const asserts become stable and modify this
	/// API to cause a compile error instead of panicking before stabilizing it.
	fn recycle<U>(self) -> Vec<U> {
		self.truncate(0);
		// TODO make these const asserts once it becomes possible
		assert!(std::mem::size_of<T>() == std::mem::size_of<U>());
		assert!(std::mem::align_of<T>() == std::mem::align_of<U>());
		let cap = self.capacity();
		let ptr = self.as_mut_ptr();
		unsafe { std::vec::from_raw_parts(ptr, 0, cap) }
	}
}
