
use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};


use serde::{Deserialize, Serialize};
use serde_json::Result;


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


fn get_config(config: &'static str) -> Result<VirtualScenario>  {
    let file = std::fs::File::open(config).expect("Error open file");
    let scenario: VirtualScenario = serde_json::from_reader(file)?;
    Ok(scenario)
}

pub struct VirtualComboardClient {}

impl super::interface::ComboardClient for VirtualComboardClient {

    fn run(&self,
        config_comboard: super::interface::ComboardClientConfig) -> tokio::task::JoinHandle<()> {
        return tokio::spawn(async move {
            let config = get_config("./virtual-comboard.json").expect("Failed to load config for virtual comboard");
            // Read json config file
            let mut i: usize = 0;
            let mut waiting: Option<std::time::Instant> = None;
            while i < config.actions.len() {

                // check if we have event to process config change
                let config_request = config_comboard.receiver_config.lock().unwrap().try_recv();
                if config_request.is_ok() {
                    let config = config_request.unwrap();
                    println!("Virtual comboard apply config {:?}", config.buffer);

                    config_comboard.sender_value_validation.lock().unwrap().send(
                        ModuleValueValidationEvent{
                            port: config.port,
                            buffer: Vec::from(config.buffer)
                        }
                    ).unwrap();
                }

                if waiting.is_none() {
                    match config.actions[i].event_type.as_str() { 
                        "state" => {
                            config_comboard.sender_state_change.lock().unwrap().send(
                                ModuleStateChangeEvent{
                                    port: config.actions[i].port,
                                    id: config.actions[i].id.clone(),
                                    state: config.actions[i].state,
                                }
                            ).unwrap();
                        },
                        "value" => {
                            let new_buffer = config.actions[i].buffer.clone();
                            config_comboard.sender_value_validation.lock().unwrap().send(
                                ModuleValueValidationEvent{
                                    port: config.actions[i].port,
                                    buffer: new_buffer,
                            }
                            ).unwrap();
                        },
                        _ => {
                            println!("Invalid event type for action");
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
                    if waiting.is_some() && waiting.unwrap().elapsed() > std::time::Duration::from_millis(config.actions[i].timeout) {
                        waiting = None;
                    }
                }
            }

            println!("End of current scenario virtual comboard");
        });
    }
}
