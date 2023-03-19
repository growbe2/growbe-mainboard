mod channel;

use crate::comboard::imple::i2c_linux::channel::*;
use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};
use crate::mainboardstate::error::MainboardError;
use crate::modulestate::interface::ModuleMsg;
use std::error::Error;
use std::ffi::CStr;

use self::channel::{Module_Config, CHANNEL_CONFIG_I2C};

use regex::Regex;
use rppal::gpio::Gpio;
use tokio::select;

lazy_static::lazy_static! {
    static ref I2C_DEVICE_INDEX: Regex = Regex::new("i2c-([0-9])").unwrap();
}

extern "C" fn callback_state_changed(
    device: i32,
    port: i32,
    id: *const ::std::os::raw::c_char,
    state: bool,
) -> () {
    let c_str: &CStr = unsafe { CStr::from_ptr(id) };
    let str_slice = c_str.to_str().unwrap();

    CHANNEL_STATE
        .0
        .try_lock()
        .unwrap()
        .try_send(ModuleStateChangeEvent {
            board: "i2c".to_string(),
            board_addr: format!("/dev/i2c-{}", device),
            port,
            id: String::from(str_slice),
            state,
        })
        .map_err(|x| MainboardError::from_error(x.to_string()))
        .unwrap();
}

extern "C" fn callback_value_validation(device: i32, port: i32, buffer: &[u8; 512]) -> () {
    CHANNEL_VALUE
        .0
        .try_lock()
        .unwrap()
        .try_send(ModuleValueValidationEvent {
            port,
            board: "i2c".to_string(),
            board_addr: format!("/dev/i2c-{}", device),
            buffer: buffer.to_vec(),
        })
        .map_err(|x| MainboardError::from_error(x.to_string()))
        .unwrap();
}

extern "C" fn callback_config(_device: i32, config: *mut channel::Module_Config) {
    if !config.is_null() {
        let value = CHANNEL_CONFIG_I2C.1.try_lock().unwrap().try_recv();
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

#[link(name = "mainboard_driver")]
extern "C" {
    fn register_callback_comboard(
        cb: extern "C" fn(i32, i32, *const ::std::os::raw::c_char, bool) -> (),
        cb1: extern "C" fn(i32, i32, &[u8; 512]),
        cb2: extern "C" fn(i32, *mut channel::Module_Config),
    );

    // starting
    fn comboard_loop_body(device: i32, starting_port: i32, ending_port: i32);
    fn init(device: *const ::std::os::raw::c_char) -> i32;
}

pub struct PIHatControl {}

impl PIHatControl {
    pub fn enable() -> Result<(), Box<dyn Error>> {
        let mut hat_pin = Gpio::new()?.get(23)?.into_output();

        hat_pin.set_high();
        hat_pin.set_high();
        log::info!("hat for board {}", hat_pin.is_set_high());

        return Ok(());
    }

    pub fn disable() -> Result<(), Box<dyn Error>> {
        let mut hat_pin = Gpio::new()?.get(23)?.into_output();

        hat_pin.set_low();
        hat_pin.set_low();

        log::info!("hat for board {}", hat_pin.is_set_low());

        return Ok(());
    }

    fn enable_led_hat() {
        tokio::spawn(async move {
            let mut led_pin = Gpio::new().unwrap().get(21).unwrap().into_output();

            let mut b = false;

            //let mut hat_pin = Gpio::new().unwrap().get(23).unwrap().into_output();
            //hat_pin.set_high();
            //log::info!("hat for led {}", hat_pin.is_set_high());

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
    pub config_comboard: super::interface::ComboardClientConfig,
}

impl super::interface::ComboardClient for I2CLinuxComboardClient {
    fn run(
        &mut self,
        sender_module: tokio::sync::mpsc::Sender<ModuleMsg>,
        mut receiver_config: tokio::sync::mpsc::Receiver<
            crate::comboard::imple::channel::ModuleConfig,
        >,
    ) -> tokio::task::JoinHandle<Result<(), ()>> {
        let str_config: Vec<String> = self
            .config_comboard
            .config
            .clone()
            .split(":")
            .map(|x| x.to_string())
            .collect();

        let device = str_config.get(0).unwrap().clone();

        let starting_port: i32 = if let Some(item) = str_config.get(1) {
            item.parse().unwrap()
        } else {
            0
        };

        let ending_port: i32 = if let Some(item) = str_config.get(2) {
            item.parse().unwrap()
        } else {
            8
        };

        let c = std::ffi::CString::new(device.as_str()).unwrap();

        let device_index: i32 = if let Some(matches) = I2C_DEVICE_INDEX.captures(device.as_str()) {
            println!("{:?}", matches);
            if let Some(matc) = matches.get(1) {
                matc.as_str().parse::<i32>().unwrap()
            } else {
                1
            }
        } else {
            1
        };

        log::info!(
            "Starting comboard with config {} {} {}:{}",
            device,
            device_index,
            starting_port,
            ending_port
        );

        match PIHatControl::enable() {
            Ok(_) => PIHatControl::enable_led_hat(),
            Err(_) => {}
        }

        return tokio::task::spawn(async move {
            /*
            match PIHatControl::disable() {
                Ok(_) => {}
                Err(_) => {}
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            match PIHatControl::enable() {
                Ok(_) => PIHatControl::enable_led_hat(),
                Err(_) => {}
            }
            */

            unsafe {
                register_callback_comboard(
                    callback_state_changed,
                    callback_value_validation,
                    callback_config,
                );
                if init(c.as_ptr()) == -1 {
                    panic!("cannot open comboard device");
                }
            }
            // Start a thread
            //
            //
            let d_i = device_index;
            let s_p = starting_port;
            let e_p = ending_port;

            std::thread::spawn(move || loop {
                unsafe {
                    comboard_loop_body(d_i, s_p, e_p);
                }
            });

            let mut receiver_value = CHANNEL_VALUE.1.lock().await;
            let mut receiver_state = CHANNEL_STATE.1.lock().await;

            loop {
                select! {
                    Some(value) = receiver_config.recv() => {
                        let v: [u8; 8] = value.data.try_into().unwrap();
                        CHANNEL_CONFIG_I2C
                            .0
                            .lock()
                            .await
                            .send(Module_Config {
                                port: value.port,
                                buffer: v,
                            })
                            .await
                            .unwrap();
                    },
                    Some(value) = receiver_value.recv() => {
                        sender_module.send(ModuleMsg::Value(value)).await.unwrap();
                    },
                    Some(value) = receiver_state.recv() => {
                        sender_module.send(ModuleMsg::State(value)).await.unwrap();
                    }

                }
            }
        });
    }
}
