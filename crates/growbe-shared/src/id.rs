use std::fs::DirEntry;

lazy_static::lazy_static! {
    static ref ID: std::sync::Mutex<String> = std::sync::Mutex::new(get_id().unwrap());
}

const IF_PATH: &'static str = "/sys/class/net/";
const ADDR_SUB_PATH: &'static str = "/address";

fn read_id(if_name: &str) -> Result<String, ()> {
    let file_path = format!("{}{}{}", IF_PATH, if_name, ADDR_SUB_PATH);
    let file = std::fs::read_to_string(&file_path).map_err(|_e| {
        return ();
    })?;
    if file.len() >= 17 {
        return Ok(String::from(&file[9..17]).replace(":", ""));
    }
    return Err(());
}

// try to use the mac address of wlan0 if not use the last interface found
fn get_id() -> Result<String, ()> {
    use std::fs;

    match fs::read_dir(IF_PATH) {
        Ok(paths) => {
            let paths = paths.map(|x| x.unwrap()).collect::<Vec<DirEntry>>();
            if let Some(path) = paths.iter().find(|x| {
                return x.file_name().eq("wlan0");
            }) {
                return read_id(path.file_name().to_str().unwrap());
            }
            // TODO: filter bad interface like vpn
            for path in paths.iter().rev() {
                if let Ok(id) = read_id(path.file_name().to_str().unwrap()) {
                    return Ok(id);
                }
            }
            log::error!("cannot get first entry from network interface");
            return Err(());
        }
        Err(err) => {
            log::error!("cannot list network interface to determined id: {}", err);
            return Err(());
        }
    }
}

pub fn get() -> String {
    return ID.lock().unwrap().clone();
}
