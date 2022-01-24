 
use std::ffi::CStr;

use crate::comboard::imple::channel::*;
use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};


extern fn callback_state_changed(port: i32, id: *const ::std::os::raw::c_char, state: bool) -> () {
    let c_str: &CStr = unsafe { CStr::from_ptr(id) };
    let str_slice = c_str.to_str().unwrap();

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
        let value = CHANNEL_CONFIG.1.lock().unwrap().try_recv();
        if value.is_ok() {
            let v = value.unwrap();
            unsafe {
                (*config).port = v.port;
                (*config).buffer = v.buffer;
            }
        } else {
            unsafe {
                (*config).port = -1;
            }
        }
    } else {
        log::error!("callback_config is null")
        // maybe print something
    }
}

extern fn log_my_ass(id: *const ::std::os::raw::c_char) {
    let c_str: &CStr = unsafe { CStr::from_ptr(id) };
    let str_slice = c_str.to_str().unwrap();
    log::debug!("{}", str_slice);
}

#[link(name="mainboard_driver")]
extern "C" {
    fn register_callback_comboard(
        cb: extern fn(i32,*const ::std::os::raw::c_char,bool) -> (),
        cb1: extern fn(i32, &[u8; 512]),
        cb2: extern fn( *mut super::interface::Module_Config),
        cb3: extern fn(*const ::std::os::raw::c_char)
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
            register_callback_comboard(callback_state_changed, callback_value_validation, callback_config, log_my_ass);
            if init(c.as_ptr()) == -1 {
                panic!("cannot open comboard device");
            }
         }
         loop {
            unsafe { comboard_loop_body(); }
            std::thread::sleep(std::time::Duration::from_millis(50));
         }
        });
    }
}
