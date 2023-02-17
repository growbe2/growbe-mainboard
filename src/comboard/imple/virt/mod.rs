use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

use crate::comboard::imple::interface::{
    ModuleStateChangeEvent, ModuleValueValidationEvent, I2C_VIRT_ID,
};

use crate::mainboardstate::error::MainboardError;
use crate::modulestate::interface::ModuleMsg;
use crate::protos::virt::{VirtualScenarioItem, VirtualScenarioItems};

pub fn create_virtual_comboard_cmd() -> (
    Sender<Vec<VirtualScenarioItem>>,
    Receiver<Vec<VirtualScenarioItem>>,
) {
    return tokio::sync::mpsc::channel::<Vec<VirtualScenarioItem>>(10);
}

pub struct VirtualComboardClient {
    pub receiver_config: Option<Receiver<Vec<VirtualScenarioItem>>>,
    pub config_comboard: super::interface::ComboardClientConfig,
}

async fn handle_items(
    sender_module: &Sender<ModuleMsg>,
    config_board: &String,
    item: &mut VirtualScenarioItem,
    map_module: &mut std::sync::Arc<Mutex<HashMap<i32, VirtualScenarioItem>>>,
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
                map_module.lock().await.insert(item.port, item.clone());
            } else {
                map_module.lock().await.remove(&item.port);
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
            /*
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
            }*/
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
        return Some(tokio::time::Duration::from_millis(item.timeout as u64));
    }
    return None;
}

async fn process_item(
    value: Vec<VirtualScenarioItem>,
    sender_module: &Sender<ModuleMsg>,
    config_board: &String,
    mut map_module: &mut std::sync::Arc<Mutex<HashMap<i32, VirtualScenarioItem>>>,
) -> () {
    println!("Values {:?}", value);
    for mut item in value {
        if item.timeout > 0 {
            let sender_module = sender_module.clone();
            let config_board = config_board.clone();
            let mut map_module = std::sync::Arc::clone(&map_module);

            // to not spawn to many task i may need to group all item by
            // timeout duration and schedule them together but this will
            // be for another time
            tokio::spawn(async move {
                handle_items(&sender_module, &config_board, &mut item, &mut map_module).await;
            });
        } else {
            handle_items(&sender_module, &config_board, &mut item, &mut map_module).await;
        }
    }
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
            let mut map_module =
                std::sync::Arc::new(Mutex::new(HashMap::<i32, VirtualScenarioItem>::new()));

            if let Ok(file) = std::fs::read_to_string(config_board.to_string()) {
                match serde_json::from_str::<VirtualScenarioItems>(&file) {
                    Ok(payload) => {
                        process_item(
                            payload.items.to_vec(),
                            &sender_module,
                            &config_board,
                            &mut map_module,
                        )
                        .await;
                    }
                    Err(err) => {
                        log::error!("failed to parse {:?}", err);
                    }
                }
            } else {
                log::error!("failed to read {}", config_board.to_string());
            }

            loop {
                tokio::select! {
                    Some(value) = receiver_config_item.recv() => {
                        process_item(value, &sender_module, &config_board, &mut map_module).await;
                    },
                    module_config = receiver_config.recv() => {
                        let module_config = module_config.unwrap();
                        if let Some(item) = map_module.lock().await.get_mut(&module_config.port) {
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
