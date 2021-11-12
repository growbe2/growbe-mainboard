mod protos;
mod comboard;
mod modulestate;

use std::sync::{Mutex, Arc, mpsc::channel};
use comboard::imple;

#[tokio::main]
async fn main() {

    let d = comboard::getComboardClient();

    // channel for the communication between the comboard
    // and the modulestate manager
    let (sender, receiver) = channel::<imple::interface::Module_Config>();
    let (senderState, receiverState) = channel::<imple::interface::ModuleStateChangeEvent>();
    let (senderValue, receiverValue) = channel::<imple::interface::ModuleValueValidationEvent>();

    let comboardTask = d.run(
        comboard::imple::interface::ComboardClientConfig{
        receiverConfig: Arc::new(Mutex::new(receiver)),
        senderStateChange: Arc::new(Mutex::new(senderState)),
        senderValueValidation: Arc::new(Mutex::new(senderValue)),
    });

    let moduleStateTask = modulestate::moduleStateTask(
        receiverState, receiverValue
    );

    tokio::join!(comboardTask, moduleStateTask);
}
