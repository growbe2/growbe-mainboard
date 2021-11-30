
use std::process::Command;


pub fn get_currnet_ssid() -> String {
    let result = Command::new("iwgetid").args(["-r"]).output();
    return match result {
        Ok(value) => String::from_utf8_lossy(&value.stdout).to_string(),
        Err(e) => panic!("{}", e)
    };
}


pub fn get_curret_ssid_strength() -> i32 {
    // TODO implementation is shit , should return a struct with most of the info that
    // i can gather and search in file lines for row matching ssid
    // let ssid = get_currnet_ssid();

    let file = std::fs::read_to_string("/proc/net/wireless").unwrap();
    let mut lines = file.lines();
    lines.next();
    lines.next();

    let wlan_line = lines.next().unwrap();

    let mut elements = wlan_line.split_whitespace();
    elements.next();
    elements.next();
    elements.next();

    let strength_str = elements.next().unwrap().replace(".", "");
    let strength = strength_str.parse::<i32>().unwrap();

    return strength;
}