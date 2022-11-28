use std::sync::mpsc::Sender;

use crate::{
    mainboardstate::config::get_configuration_proto,
    plateform::uname::get_host_information,
    protos::board::{HelloWord, RunningComboard},
};

use growbe_shared::version::VERSION;

impl crate::modulestate::interface::ModuleValue for crate::protos::board::HelloWord {}
impl crate::modulestate::interface::ModuleValueParsable for crate::protos::board::HelloWord {}

pub async fn task_hello_world(
    sender: Sender<(
        String,
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
    )>,
    running_boards: Vec<RunningComboard>,
) -> () {
    let mut hello = get_hello_world();
    let config = get_configuration_proto();
    hello.boards = running_boards.try_into().unwrap();
    hello.set_host(get_host_information());
    log::info!("hello world starting with version {}", hello.version);
    sender
        .send((String::from("/hello"), Box::new(hello)))
        .unwrap();
    sender
        .send((String::from("/config"), Box::new(config)))
        .unwrap();
}

pub fn get_hello_world() -> HelloWord {
    let mut hello = HelloWord::new();
    hello.cloudVersion = String::from("1.1.4");
    hello.version = VERSION.to_string();
    hello.RTC = super::rtc::get_rtc_format();

    return hello;
}
