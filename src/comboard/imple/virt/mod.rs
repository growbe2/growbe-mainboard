use std::sync::mpsc::Receiver;

use crate::comboard::imple::interface::{
    ModuleStateChangeEvent, ModuleValueValidationEvent, I2C_VIRT_ID,
};

use crate::comboard::imple::channel::*;

use serde::{Deserialize, Serialize};

fn default_index() -> i32 {
    return -1;
}

#[derive(Serialize, Deserialize)]
struct VirtualScenarioItem {
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

#[derive(Serialize, Deserialize)]
struct VirtualScenario {
    pub actions: Vec<VirtualScenarioItem>,
}

fn get_config(config: &String) -> serde_json::Result<VirtualScenario> {
    let file = std::fs::File::open(config).expect(config);
    let scenario: VirtualScenario = serde_json::from_reader(file)?;
    Ok(scenario)
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
            let config = get_config(&config_board).expect(&config_board);
            // Read json config file
            let mut i: usize = 0;
            let mut waiting: Option<std::time::Instant> = None;

            // TODO reable with new channel
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
                    match config.actions[i].event_type.as_str() {
                        "state" => {
                            sender_state
                                .send(ModuleStateChangeEvent {
                                    board: I2C_VIRT_ID.to_string(),
                                    board_addr: config_board.clone(),
                                    port: config.actions[i].port,
                                    id: config.actions[i].id.clone(),
                                    state: config.actions[i].state,
                                })
                                .unwrap();
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
                        waiting = None;
                    }
                }
            }

            log::info!("end of current scenario virtual comboard");

            Ok(())
        });
    }
}
