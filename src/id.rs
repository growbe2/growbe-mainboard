
#[cfg(target_arch = "arm")]
pub fn get_id() -> String {
    // use the mac address of ETH0 of the rasberry pi
    // to create a unique id
    // cat /sys/class/net/eth0/address
}

#[cfg(target_arch = "x86_64")]
pub fn get_id() -> String {
    return String::from(format!("V123456"));
}