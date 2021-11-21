 
use std::ffi::CStr;
use std::str;

use crate::comboard::imple::channel::*;
use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};


extern fn callback_state_changed(port: i32, id: *const ::std::os::raw::c_char, state: bool) -> () {
    let c_str: &CStr = unsafe { CStr::from_ptr(id) };
    let str_slice: &str = c_str.to_str().unwrap();

    CHANNEL_STATE.0.lock().unwrap().send(
        ModuleStateChangeEvent{
            port: port,
            id: String::from(str_slice),
            state: state,
        }
    ).unwrap();
}

extern fn callback_value_validation(port: i32, buffer: &[u8; 512]) -> () {
    CHANNEL_VALUE.0.lock().unwrap().send(
        ModuleValueValidationEvent{
            port: port,
            buffer: buffer.to_vec(),
        }
    ).unwrap();
}

extern fn callback_config(config: *mut super::interface::Module_Config) {
    if !config.is_null() {
        unsafe {
            (*config).port = 50;
        }
    } else {
        unsafe { (*config).port = -1; }
    }
}

#[link(name="mainboard_driver")]
extern "C" {
    fn register_callback(
        cb: extern fn(i32,*const ::std::os::raw::c_char,bool) -> (),
        cb1: extern fn(i32, &[u8; 512]),
        cb2: extern fn( *mut super::interface::Module_Config)
    );

    fn comboard_loop_body();
    fn init(device: *const ::std::os::raw::c_char) -> i32;
}

pub struct I2CLinuxComboardClient {}

impl super::interface::ComboardClient for I2CLinuxComboardClient {
    fn run(&self,
        config: super::interface::ComboardClientConfig) -> tokio::task::JoinHandle<()>  {
        let c = std::ffi::CString::new(config.config.as_str()).unwrap();
        return tokio::spawn(async move {
         unsafe {
            register_callback(callback_state_changed, callback_value_validation, callback_config);
            init(c.as_ptr());
         }
         loop {
            unsafe { comboard_loop_body(); }
            std::thread::sleep(std::time::Duration::from_secs(1));
         }
        });
    }
}
