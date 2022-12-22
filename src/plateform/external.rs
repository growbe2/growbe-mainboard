

pub fn is_systemd() -> bool {
    if let Ok(value) = std::env::var("INVOCATION_ID") {
        return !value.is_empty();
    }
    return false;
}
