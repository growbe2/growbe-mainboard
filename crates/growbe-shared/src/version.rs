
const CARGO_PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const SNAPSHOT_COMMIT: Option<&'static str> = option_env!("COMMIT");


lazy_static::lazy_static! {
	pub static ref VERSION: String = {
		if let Some(snapshot) = SNAPSHOT_COMMIT {
			return format!("{}-{}", CARGO_PKG_VERSION, snapshot);
		}
		return CARGO_PKG_VERSION.to_string();
	};
}