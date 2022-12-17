use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::comboard::imple::interface::{
    ModuleStateChangeEvent, ModuleValueValidationEvent, I2C_VIRT_ID,
};

use crate::mainboardstate::error::MainboardError;
use crate::modulestate::interface::ModuleMsg;

use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    pub static ref VIRTUAL_COMBOARD_CMD: (
        Mutex<Sender<Vec<VirtualScenarioItem>>>,
        Mutex<Receiver<Vec<VirtualScenarioItem>>>
    ) = {
        let (sender, receiver) = tokio::sync::mpsc::channel::<Vec<VirtualScenarioItem>>(10);
        return (Mutex::new(sender), Mutex::new(receiver));
    };
}

pub fn create_virtual_comboard_cmd() -> (
    Sender<Vec<VirtualScenarioItem>>,
    Receiver<Vec<VirtualScenarioItem>>,
) {
    return tokio::sync::mpsc::channel::<Vec<VirtualScenarioItem>>(10);
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
    pub receiver_config: Option<Receiver<Vec<VirtualScenarioItem>>>,
    pub config_comboard: super::interface::ComboardClientConfig,
}

async fn handle_items(
    sender_module: &Sender<ModuleMsg>,
    config_board: &String,
    item: &mut VirtualScenarioItem,
    map_module: &mut HashMap<i32, VirtualScenarioItem>,
) -> Option<std::time::Duration> {

    let typ = item.id[..3].to_string();
    match item.event_type.as_str() {
        "state" => {
            if item.state {
                match typ.as_str() {
                    "AAP" | "AAB" => {
                        item.buffer = vec![0, 0, 0, 0, 0, 0, 0, 0];
                    }
                    _ => {}
                };
                map_module.insert(item.port, item.clone());
            } else {
                map_module.remove(&item.port);
            };
            sender_module
                .send(ModuleMsg::State(ModuleStateChangeEvent {
                    board: I2C_VIRT_ID.to_string(),
                    board_addr: config_board.clone(),
                    port: item.port,
                    id: item.id.clone(),
                    state: item.state,
                }))
                .await
                .map_err(|x| MainboardError::from_error(x.to_string()))
                .unwrap();
            if item.buffer.len() > 2 {
                let new_buffer = item.buffer.clone();
                sender_module
                    .send(ModuleMsg::Value(ModuleValueValidationEvent {
                        board: I2C_VIRT_ID.to_string(),
                        board_addr: config_board.clone(),
                        port: item.port,
                        buffer: new_buffer,
                    }))
                    .await
                    .map_err(|x| MainboardError::from_error(x.to_string()))
                    .unwrap();
            }
        }
        "value" => {
            let new_buffer = item.buffer.clone();
            sender_module
                .send(ModuleMsg::Value(ModuleValueValidationEvent {
                    board: I2C_VIRT_ID.to_string(),
                    board_addr: config_board.clone(),
                    port: item.port,
                    buffer: new_buffer,
                }))
                .await
                .map_err(|x| MainboardError::from_error(x.to_string()))
                .unwrap();
        }
        _ => {
            log::error!("invalid event type for action : {}", item.event_type);
        }
    }

    if item.timeout > 0 {
        return Some(tokio::time::Duration::from_millis(item.timeout));
    }
    return None;
}

impl super::interface::ComboardClient for VirtualComboardClient {
    fn run(
        &mut self,
        sender_module: Sender<ModuleMsg>,
        mut receiver_config: tokio::sync::mpsc::Receiver<
            crate::comboard::imple::channel::ModuleConfig,
        >,
    ) -> tokio::task::JoinHandle<Result<(), ()>> {
        let config_board = self.config_comboard.config.clone();

        let mut receiver_config_item = self.receiver_config.take().unwrap();

        return tokio::spawn(async move {
            // PORT : MODULE_ID
            let mut map_module = HashMap::<i32, VirtualScenarioItem>::new();

            loop {
                tokio::select! {
                    value = receiver_config_item.recv() => {
                        let value = value.unwrap();
                        for mut item in value {
                            handle_items(&sender_module, &config_board, &mut item, &mut map_module).await;
                        }
                    },
                    module_config = receiver_config.recv() => {
                        let module_config = module_config.unwrap();
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
                                } else {
                                    vec![]
                                };

                                sender_module
                                    .send(ModuleMsg::Value(ModuleValueValidationEvent {
                                        board: I2C_VIRT_ID.to_string(),
                                        board_addr: config_board.clone(),
                                        port: item.port,
                                        buffer: new_buffer,
                                    }))
                                    .await
                                    .map_err(|x| MainboardError::from_error(x.to_string()))
                                    .unwrap();
                            }
                            _ => {}
                        }
                     }
                    },
                }
            }
        });
    }
}
