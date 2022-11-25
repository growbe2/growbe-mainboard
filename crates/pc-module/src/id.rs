
pub fn get() -> String {
	return format!("000{}", growbe_shared::id::get().to_uppercase());
}
