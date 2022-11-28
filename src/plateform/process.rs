use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,

    pub stat: String,
}

#[allow(dead_code)]
pub fn get_process_info() -> ProcessInfo {
    let pid = nix::unistd::getpid().as_raw() as u32;

    let file_path = format!("/proc/{}/stat", pid);
    let file_content = std::fs::read_to_string(file_path).unwrap();

    return ProcessInfo {
        pid,
        stat: file_content,
    };
}
