use tokio::sync::mpsc::Sender;

use crate::{
    mainboardstate::config::get_configuration_proto,
    plateform::uname::get_host_information,
    protos::board::{HelloWord, RunningComboard},
    socket::ss::SenderPayloadData,
};

use crate::socket::ss::SenderPayload;

use super::error::MainboardError;

use growbe_shared::version::VERSION;

impl crate::modulestate::interface::ModuleValue for crate::protos::board::HelloWord {}
impl crate::modulestate::interface::ModuleValueParsable for crate::protos::board::HelloWord {}

pub async fn task_hello_world(
    sender: Sender<SenderPayload>,
    running_boards: Vec<RunningComboard>,
) -> Result<(), MainboardError> {
    let mut hello = get_hello_world();
    let config = get_configuration_proto();
    hello.boards = running_boards.try_into().unwrap();
    if let Ok(host) = get_host_information() {
        hello.set_host(host);
    } else {
        log::error!("failed to get_host_information()");
    }

    log::info!("hello world starting with version {}", hello.version);
    sender
        .send((
            String::from("/hello"),
            SenderPayloadData::ProtobufMessage(Box::new(hello)),
        ))
        .await?;
    sender
        .send((
            String::from("/config_process"),
            SenderPayloadData::ProtobufMessage(Box::new(config)),
        ))
        .await?;

    return Ok(());
}

pub fn get_hello_world() -> HelloWord {
    let mut hello = HelloWord::new();
    // TODO: cloudVersion is it still relevant ? probably soo but need to be injected in proto code
    hello.cloudVersion = String::from("1.1.4");
    hello.version = VERSION.to_string();
    hello.RTC = super::rtc::get_rtc_format();

    return hello;
}
