
use crate::comboard::imple::interface::{ModuleStateChangeEvent, ModuleValueValidationEvent};
use std::sync::{Mutex, Arc, mpsc::Sender};

pub struct VirtualComboardClient {}

impl super::interface::ComboardClient for VirtualComboardClient {
    fn run(&self,
        config: super::interface::ComboardClientConfig) -> tokio::task::JoinHandle<()> {
        return tokio::spawn(async move {
            println!("Starting virtual truc mush");
            config.senderStateChange.lock().unwrap().send(ModuleStateChangeEvent{
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
            }).unwrap();
        });
    }
}
