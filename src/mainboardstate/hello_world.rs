use std::sync::mpsc::Sender;

use crate::{protos::{board::{HelloWord, RunningComboard,}}, mainboardstate::config::get_configuration_proto};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const SNAPSHOT_COMMIT: Option<&'static str> = option_env!("COMMIT");

impl crate::modulestate::interface::ModuleValue for crate::protos::board::HelloWord {}
impl crate::modulestate::interface::ModuleValueParsable for crate::protos::board::HelloWord {}


pub async fn task_hello_world(
    sender: Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
    running_boards: Vec<RunningComboard>,
) -> () {
    let mut hello = get_hello_world();
    let config = get_configuration_proto();
    hello.boards = running_boards.try_into().unwrap();
    log::info!("hello world starting with version {}", hello.version);
    sender.send((String::from("/hello"), Box::new(hello))).unwrap();
    sender.send((String::from("/config"), Box::new(config))).unwrap();
}


pub fn get_hello_world() -> HelloWord {
    let mut hello = HelloWord::new();
    hello.cloudVersion = String::from("1.1.4");
    if SNAPSHOT_COMMIT.is_some() {
        hello.version = format!("{}-{}", VERSION, SNAPSHOT_COMMIT.unwrap())
    } else {
        hello.version = String::from(VERSION);
    }
    hello.RTC = super::rtc::get_rtc_format();

    return hello;
}

