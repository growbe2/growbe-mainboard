use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;
use std::time::Duration;

use crate::comboard::imple::interface::{
    ModuleStateChangeEvent, ModuleValueValidationEvent, I2C_VIRT_ID,
};

use crate::comboard::imple::channel::*;

use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    pub static ref VIRTUAL_COMBOARD_CMD: (
        Mutex<Sender<Vec<VirtualScenarioItem>>>,
        Mutex<Receiver<Vec<VirtualScenarioItem>>>
    ) = {
        let (sender, receiver) = std::sync::mpsc::channel::<Vec<VirtualScenarioItem>>();
        return (Mutex::new(sender), Mutex::new(receiver));
    };
}

fn default_index() -> i32 {
    return -1;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VirtualScenarioItem {
    pub event_type: String,
    #[serde(default)]
    pub port: i32,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub state: bool,
    #[serde(default)]
    pub buffer: Vec<u8>,
    #[serde(default = "default_index")]
    pub return_index: i32,
    #[serde(default)]
    pub timeout: u64, // in milliseconds
}

#[derive(Serialize, Deserialize, Default)]
struct VirtualScenario {
    pub actions: Vec<VirtualScenarioItem>,
}

fn get_config(config: &String) -> VirtualScenario {
    if let Ok(file) = std::fs::File::open(config) {
        if let Ok(scenario) = serde_json::from_reader(file) {
            return scenario;
        }
    }
    return VirtualScenario::default();
}

pub struct VirtualComboardClient {
    pub config_comboard: super::interface::ComboardClientConfig,
}

impl super::interface::ComboardClient for VirtualComboardClient {
    fn run(
        &self,
        receiver_config: Receiver<crate::comboard::imple::channel::ModuleConfig>,
    ) -> tokio::task::JoinHandle<Result<(), ()>> {
        let sender_value = CHANNEL_VALUE.0.lock().unwrap().clone();
        let sender_state = CHANNEL_STATE.0.lock().unwrap().clone();

        let config_board = self.config_comboard.config.clone();

        return tokio::spawn(async move {
            // PORT : MODULE_ID
            let mut map_module = HashMap::<i32, VirtualScenarioItem>::new();

            let mut config = get_config(&config_board);
            // Read json config file
            let mut i: usize = 0;
            let mut waiting: Option<std::time::Instant> = None;

            let receiver = VIRTUAL_COMBOARD_CMD.1.lock().unwrap();

            loop {
                if let Ok(mut value) = receiver.recv_timeout(Duration::from_millis(10)) {
                    log::info!("receive new action for virtual board {}", config.actions.len());
                    config.actions.append(&mut value);
                    log::info!("receive new action for virtual board {}", config.actions.len());
                }
                if let Ok(module_config) = receiver_config.recv_timeout(Duration::from_millis(10)) {
                    if let Some(item) = map_module.get_mut(&module_config.port) {
                        match &item.id[..3] {
                            "AAP" | "AAB" => {
                                let new_buffer = if module_config.data.len() == 8 {
                                    let mut new_buffer = module_config.data.clone();
                                    for i in 0..8 {
                                        if new_buffer[i] == 255 {
                                            new_buffer[i] = item.buffer[i];
                                        } else {
                                            item.buffer[i] = new_buffer[i];
                                        }
                                    }
                                    new_buffer
                                } else { vec![] };

                                sender_value
                                    .send(ModuleValueValidationEvent {
                                        board: I2C_VIRT_ID.to_string(),
                                        board_addr: config_board.clone(),
                                        port: item.port,
                                        buffer: new_buffer,
                                    })
                                    .unwrap();
                            }
                            _ => {}
                        }
                    }
                }
                while i < config.actions.len() {
                    // check if we have event to process config change
                    let config_request = receiver_config.try_recv();
                    if config_request.is_ok() {
                        let config = config_request.unwrap();
                        log::debug!("virtual comboard apply config {:?}", config.data);

                        sender_value
                            .send(ModuleValueValidationEvent {
                                board: I2C_VIRT_ID.to_string(),
                                board_addr: config_board.clone(),
                                port: config.port,
                                buffer: config.data,
                            })
                            .unwrap();
                    }

                    if waiting.is_none() {
                        let typ = config.actions[i].id[..3].to_string();
                        match config.actions[i].event_type.as_str() {
                            "state" => {
                                if config.actions[i].state {
                                    match typ.as_str() {
                                        "AAP" | "AAB" => {
                                            config.actions[i].buffer = vec![0, 0, 0, 0, 0, 0, 0, 0];
                                        }
                                        _ => {}
                                    };
                                    map_module
                                        .insert(config.actions[i].port, config.actions[i].clone());
                                } else {
                                    map_module.remove(&config.actions[i].port);
                                };
                                sender_state
                                    .send(ModuleStateChangeEvent {
                                        board: I2C_VIRT_ID.to_string(),
                                        board_addr: config_board.clone(),
                                        port: config.actions[i].port,
                                        id: config.actions[i].id.clone(),
                                        state: config.actions[i].state,
                                    })
                                    .unwrap();
                                if config.actions[i].buffer.len() > 2 {
                                    let new_buffer = config.actions[i].buffer.clone();
                                    sender_value
                                        .send(ModuleValueValidationEvent {
                                            board: I2C_VIRT_ID.to_string(),
                                            board_addr: config_board.clone(),
                                            port: config.actions[i].port,
                                            buffer: new_buffer,
                                        })
                                        .unwrap();
                                }
                            }
                            "value" => {
                                let new_buffer = config.actions[i].buffer.clone();
                                sender_value
                                    .send(ModuleValueValidationEvent {
                                        board: I2C_VIRT_ID.to_string(),
                                        board_addr: config_board.clone(),
                                        port: config.actions[i].port,
                                        buffer: new_buffer,
                                    })
                                    .unwrap();
                            }
                            _ => {
                                log::error!(
                                    "invalid event type for action : {}",
                                    config.actions[i].event_type
                                );
                            }
                        }

                        if config.actions[i].timeout > 0 {
                            waiting = Some(std::time::Instant::now());
                            println!("sleeping ");
                            //tokio::time::sleep(tokio::time::Duration::from_millis(config.actions[i].timeout)).await;
                        }

                        if config.actions[i].return_index > -1 {
                            i = config.actions[i].return_index as usize;
                        } else {
                            i += 1;
                        }
                    } else {
                        if waiting.is_some()
                            && waiting.unwrap().elapsed()
                                > std::time::Duration::from_millis(config.actions[i].timeout)
                        {
                            println!("done waiting");
                            waiting = None;
                        }
                    }
                }
            }
        });
    }
}
