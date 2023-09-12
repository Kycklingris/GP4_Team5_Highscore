pub const unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
	::core::slice::from_raw_parts(
		(p as *const T) as *const u8,
		::core::mem::size_of::<T>()
	)
}

pub unsafe fn any_from_u8_slice<T: Sized + Clone>(bytes: &[u8]) -> T {
	let reference = & *(bytes as *const [u8] as *const T);

	reference.clone()
}