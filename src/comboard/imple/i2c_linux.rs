 
use std::ffi::CStr;
use std::str;


extern fn callback_state_changed(port: i32, id: *const ::std::os::raw::c_char, state: bool) -> () {
    let c_str: &CStr = unsafe { CStr::from_ptr(id) };
    let str_slice: &str = c_str.to_str().unwrap();
    println!("{} {} {}",port, str_slice, state)
}

extern fn callback_value_validation(port: i32, buffer: &[::std::os::raw::c_char; 512]) -> () {
    println!("Validation {}, first 5 byte, {} {} {} {} {}", port, buffer[0], buffer[1], buffer[2], buffer[3], buffer[4])
}

extern fn callback_config(config: *mut super::interface::Module_Config) {
    println!("muttable");
    if !config.is_null() {
        unsafe {
            (*config).port = 50;
        }
    } 
}

#[link(name="mainboard_driver")]
extern "C" {
    fn register_callback(
        cb: extern fn(i32,*const ::std::os::raw::c_char,bool) -> (),
        cb1: extern fn(i32, &[::std::os::raw::c_char; 512]),
        cb2: extern fn( *mut super::interface::Module_Config)
    );

    fn comboard_loop_body();
}

pub struct I2CLinuxComboardClient {}

impl super::interface::ComboardClient for I2CLinuxComboardClient {
    fn run(&self, config: super::interface::ComboardClientConfig) -> std::thread::JoinHandle<()> {
        return std::thread::spawn(|| {
         unsafe {
             register_callback(callback_state_changed, callback_value_validation, callback_config);
         }
         loop {
            unsafe { comboard_loop_body(); }
            std::thread::sleep(std::time::Duration::from_secs(1));
         }
        });
    }
}
