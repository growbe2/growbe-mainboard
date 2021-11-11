mod protos;
mod comboard;

use std::sync::mpsc::channel;

#[tokio::main]
async fn main() {

    let d = comboard::getComboardClient();

    let (sender, receiver) = channel();
    let (senderState, receiverState) = channel();
    let (senderValue, receiverValue) = channel();

    let comboardTask = d.run(
        comboard::imple::interface::ComboardClientConfig{
        receiverConfig: receiver,
        senderStateChange: senderState,
        senderValueValidation: senderValue,
    });

    tokio::join!(comboardTask);
}
/*
fn main() {
    println!("Hello, world!");

    let d = comboard::getComboardClient();

    let (sender, receiver) = channel();
    let (senderState, receiverState) = channel();
    let (senderValue, receiverValue) = channel();

    let comboardHandler = d.run(
        comboard::imple::interface::ComboardClientConfig{
        receiverConfig: receiver,
        senderStateChange: senderState,
        senderValueValidation: senderValue,
    }).join();
}
*/