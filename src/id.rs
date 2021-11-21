
lazy_static::lazy_static! {
    static ref ID: std::sync::Mutex<String> = std::sync::Mutex::new(get_id());
}

#[cfg(target_arch = "arm")]
fn get_id() -> String {
    let file_path = format!("/sys/class/net/{}/address","wlan0");
    let file = std::fs::read_to_string(file_path).unwrap();
    return String::from((&file[9..17])).replace(":", "");
}

#[cfg(target_arch = "x86_64")]
fn get_id() -> String {
    // for x86 use a id present in the config
    return String::from(crate::mainboardstate::config::CONFIG.id.as_str());
}


pub fn get() -> String {
    return ID.lock().unwrap().clone();
}