
use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};
use std::sync::{Mutex, Arc, mpsc::Sender};

use serde::{Deserialize, Serialize};
use serde_json::Result;

enum VirtualScenarioEvent {
    VALUE = 1,
    STATE = 2
}

#[derive(Serialize, Deserialize)]
struct VirtualScenario {
    pub eventType: VirtualComboardClient,
    pub id: &'static str,
    pub state: bool,
    pub buffer: [u8; 512],
}

fn getConfig(config: &'static str) -> Result<()>  {
    let configStr = std::fs::read_to_string("virtual-comboard.json")?;
    let scenario = serde_json::from_str(&configStr.as_str())?;

    println(scenario.id);

    Ok(())
}

pub struct VirtualComboardClient {}

impl super::interface::ComboardClient for VirtualComboardClient {

    fn run(&self,
        config: super::interface::ComboardClientConfig) -> tokio::task::JoinHandle<()> {
        return tokio::spawn(async move {

            match getConfig("") {
                Ok(_) => println!("OK"),
                Err(_) => println!("Error")
            }
            // Read json config file
           ;
            /*config.senderStateChange.lock().unwrap().send(ModuleStateChangeEvent{
                port: 1,
                id: "AAP000000005",
                state: true,
            }).unwrap();

            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            config.senderValueValidation.lock().unwrap().send(ModuleValueValidationEvent{
                port: 5,
                buffer: [5; 512],
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

            config.senderStateChange.lock().unwrap().send(ModuleStateChangeEvent{
                port: 1,
                id: "AAP000000005",
                state: false,
            }).unwrap();*/
        });
    }
}
