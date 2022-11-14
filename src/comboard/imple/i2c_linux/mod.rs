 
pub mod channel;

use std::ffi::CStr;
use std::error::Error;
use std::sync::mpsc::Receiver;
use crate::comboard::imple::channel::*;
use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};

use self::channel::{CHANNEL_CONFIG_I2C, Module_Config};

use tokio::select;
use rppal::gpio::Gpio;


extern fn callback_state_changed(port: i32, id: *const ::std::os::raw::c_char, state: bool) -> () {
    let c_str: &CStr = unsafe { CStr::from_ptr(id) };
    let str_slice = c_str.to_str().unwrap();

    CHANNEL_STATE.0.lock().unwrap().send(
        ModuleStateChangeEvent{
            board: "i2c".to_string(),
            board_addr: "/dev/i2c-1".to_string(),
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
            board_addr: "/dev/i2c-1".to_string(),
            buffer: buffer.to_vec(),
        }
    ).unwrap();
}

extern fn callback_config(config: *mut channel::Module_Config) {
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
        cb2: extern fn( *mut channel::Module_Config)
    );

    // starting 
    fn comboard_loop_body(starting_port: i32, ending_port: i32);
    fn init(device: *const ::std::os::raw::c_char) -> i32;
}

pub struct PIHatControl {}

impl PIHatControl {
    fn enable() -> Result<(), Box<dyn Error>> {
        let mut hat_pin = Gpio::new()?.get(23)?.into_output();

        hat_pin.set_high();
        hat_pin.set_high();
        log::info!("hat for board {}", hat_pin.is_set_high());

        return Ok(());
    }

    fn enable_led_hat() {
        tokio::spawn(async move {
            let mut led_pin = Gpio::new().unwrap().get(21).unwrap().into_output();

            let mut b = false;
            
            let mut hat_pin = Gpio::new().unwrap().get(23).unwrap().into_output();
            hat_pin.set_high();
            log::info!("hat for led {}", hat_pin.is_set_high());

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
    fn run(&self, receiver_config: Receiver<crate::comboard::imple::channel::ModuleConfig>) -> tokio::task::JoinHandle<Result<(), ()>>  {
        let str_config: Vec<String> = self.config_comboard.config.clone().split(":").map(|x| x.to_string()).collect();

        let device = str_config.get(0).unwrap().clone();

        let starting_port: i32 = if let Some(item) = str_config.get(1) { item.parse().unwrap() } else { 0 };

        let ending_port: i32 = if let Some(item) = str_config.get(2) { item.parse().unwrap() } else { 8 };

        let c = std::ffi::CString::new(device.as_str()).unwrap();

        log::info!("Starting comboard with config {} {}:{}", device, starting_port, ending_port);

        match PIHatControl::enable() {
            Ok(_) => PIHatControl::enable_led_hat(),
            Err(_) => {}
        }

        return tokio::spawn(async move {
         unsafe {
            register_callback_comboard(callback_state_changed, callback_value_validation, callback_config);
            if init(c.as_ptr()) == -1 {
                panic!("cannot open comboard device");
            }
         }
         loop {
            unsafe {
                if let Ok(value) = receiver_config.try_recv() {
                    let v: [u8; 8] = value.data.try_into().unwrap();
                    CHANNEL_CONFIG_I2C.0.lock().unwrap().send(Module_Config{
                        port: value.port,
                        buffer: v,
                    }).unwrap();
                }

                comboard_loop_body(starting_port, ending_port);
            }
         }
         Ok(())
        });
    }
}
