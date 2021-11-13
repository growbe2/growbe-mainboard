
use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};
use std::sync::{Mutex, Arc, mpsc::Sender};

use serde::{Deserialize, Serialize};
use serde_json::Result;


fn defaultIndex() -> i32 {
    return -1;
}

#[derive(Serialize, Deserialize)]
struct VirtualScenarioItem {
    pub eventType: String,
    #[serde(default)] 
    pub port: i32,
    #[serde(default)] 
    pub id: String,
    #[serde(default)] 
    pub state: bool,
    #[serde(default)] 
    pub buffer: Vec<u8>,
    #[serde(default = "defaultIndex")] 
    pub returnIndex: i32,
    #[serde(default)] 
    pub timeout: u64, // in milliseconds
}

#[derive(Serialize, Deserialize)]
struct VirtualScenario {
    pub actions: Vec<VirtualScenarioItem>,
}


fn getConfig(config: &'static str) -> Result<VirtualScenario>  {
    let file = std::fs::File::open(config).expect("Error open file");
    let scenario: VirtualScenario = serde_json::from_reader(file)?;
    Ok(scenario)
}

pub struct VirtualComboardClient {}

impl super::interface::ComboardClient for VirtualComboardClient {

    fn run(&self,
        configComboard: super::interface::ComboardClientConfig) -> tokio::task::JoinHandle<()> {
        return tokio::spawn(async move {
            let config = getConfig("./virtual-comboard.json").expect("Failed to load config for virtual comboard");
            // Read json config file
            let mut i: usize = 0;
            while i < config.actions.len() {
                match config.actions[i].eventType.as_str() { 
                    "state" => {
                        configComboard.senderStateChange.lock().unwrap().send(
                            ModuleStateChangeEvent{
                                port: config.actions[i].port,
                                id: config.actions[i].id.clone(),
                                state: config.actions[i].state,
                            }
                        );
                    },
                    "value" => {
                        let newBuffer = config.actions[i].buffer.clone();
                        configComboard.senderValueValidation.lock().unwrap().send(
                            ModuleValueValidationEvent{
                                port: config.actions[i].port,
                                buffer: newBuffer,
                            }
                        );
                    },
                    _ => {
                        println!("Invalid event type for action");
                    }
                }

                if config.actions[i].timeout > 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(config.actions[i].timeout)).await;
                }

                if config.actions[i].returnIndex > -1 {
                    i = config.actions[i].returnIndex as usize; 
                } else {
                    i += 1;
                }
            }

            println!("End of current scenario virtual comboard");
        });
    }
}
