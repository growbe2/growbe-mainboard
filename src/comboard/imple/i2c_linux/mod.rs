 
use std::ffi::CStr;
use std::error::Error;
use crate::comboard::imple::channel::*;
use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};

use tokio::select;
use rppal::gpio::Gpio;

extern fn callback_state_changed(port: i32, id: *const ::std::os::raw::c_char, state: bool) -> () {
    let c_str: &CStr = unsafe { CStr::from_ptr(id) };
    let str_slice = c_str.to_str().unwrap();

    CHANNEL_STATE.0.lock().unwrap().send(
        ModuleStateChangeEvent{
            board: "i2c".to_string(),
            board_addr: "1".to_string(),
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
            board: "i2c".to_string(),
            board_addr: "1".to_string(),
            buffer: buffer.to_vec(),
        }
    ).unwrap();
}

extern fn callback_config(config: *mut super::interface::Module_Config) {
    if !config.is_null() {
        let value = CHANNEL_CONFIG_I2C.1.lock().unwrap().try_recv();
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
        // maybe print something
    }
}

#[link(name="mainboard_driver")]
extern "C" {
    fn register_callback_comboard(
        cb: extern fn(i32,*const ::std::os::raw::c_char,bool) -> (),
        cb1: extern fn(i32, &[u8; 512]),
        cb2: extern fn( *mut super::interface::Module_Config)
    );

    fn comboard_loop_body();
    fn init(device: *const ::std::os::raw::c_char) -> i32;
}

pub struct PIHatControl {}

impl PIHatControl {
    fn enable() -> Result<(), Box<dyn Error>> {
        let mut hat_pin = Gpio::new()?.get(23)?.into_output();

        hat_pin.set_high();
        hat_pin.set_high();
        log::info!("hat is {}", hat_pin.is_set_high());

        return Ok(());
    }

    fn enable_led_hat() {
        tokio::spawn(async move {
            let mut led_pin = Gpio::new().unwrap().get(21).unwrap().into_output();

            let mut b = false;
            
            log::info!("starting led hat");

            let mut hat_pin = Gpio::new().unwrap().get(23).unwrap().into_output();
            hat_pin.set_high();
            log::info!("hat is {}", hat_pin.is_set_high());

            loop {
                select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                        if b {
                            led_pin.set_high();
                        } else {
                            led_pin.set_low();
                        }
                        b = !b
                    }
                }
            }
        });
    }

}

pub struct I2CLinuxComboardClient {
    pub config_comboard: super::interface::ComboardClientConfig
}

impl super::interface::ComboardClient for I2CLinuxComboardClient {
    fn run(&self) -> tokio::task::JoinHandle<()>  {
        let c = std::ffi::CString::new(self.config_comboard.config.as_str()).unwrap();

        PIHatControl::enable().unwrap();
        PIHatControl::enable_led_hat();

        return tokio::spawn(async move {
         unsafe {
            register_callback_comboard(callback_state_changed, callback_value_validation, callback_config);
            if init(c.as_ptr()) == -1 {
                panic!("cannot open comboard device");
            }
         }
         loop {
            unsafe { comboard_loop_body(); }
            //std::thread::sleep(std::time::Duration::from_millis(50));
         }
        });
    }
}