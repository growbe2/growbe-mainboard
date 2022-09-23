
lazy_static::lazy_static! {
    static ref ID: std::sync::Mutex<String> = std::sync::Mutex::new(get_id());
}

fn get_id() -> String {
    // for x86 use a id present in the config
    return String::from(crate::config::CONFIG.id.as_str()).to_uppercase();
}

pub fn get() -> String {
    return ID.lock().unwrap().clone();
}