use std::fs::DirEntry;


lazy_static::lazy_static! {
    static ref ID: std::sync::Mutex<String> = std::sync::Mutex::new(get_id());
}

const IF_PATH: &'static str = "/sys/class/net/";
const ADDR_SUB_PATH: &'static str = "/address";

fn read_id(if_name: &str) -> String {
    let file_path = format!("{}{}{}", IF_PATH, if_name, ADDR_SUB_PATH);
    let file = std::fs::read_to_string(file_path).unwrap();
    return String::from(&file[9..17]).replace(":", "");
}

// try to use the mac address of wlan0 if not use the last interface found
fn get_id() -> String {
    use std::fs;

    match fs::read_dir(IF_PATH) {
        Ok(paths) => {
            let paths = paths.map(|x| x.unwrap()).collect::<Vec<DirEntry>>();
            if let Some(path) = paths.iter().find(|x| {
                return x.file_name().eq("wlan0");
            }) {
                return read_id(path.file_name().to_str().unwrap())
            }
            if let Some(path) = paths.get(paths.len() - 1) {
                return read_id(path.file_name().to_str().unwrap())
            } else {
                panic!("cannot get first entry from network interface");
            }
        },
        Err(err) => panic!("cannot list network interface to determined id: {}", err)
    }
}


pub fn get() -> String {
    return ID.lock().unwrap().clone();
}