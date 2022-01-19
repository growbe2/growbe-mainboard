use std::sync::mpsc::Sender;
use rumqtt::Header;

use crate::protos::board::HelloWord;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const SNAPSHOT_COMMIT: Option<&'static str> = option_env!("SHAPSHOT_COMMIT");

impl crate::modulestate::interface::ModuleValue for crate::protos::board::HelloWord {}
impl crate::modulestate::interface::ModuleValueParsable for crate::protos::board::HelloWord {}

pub async fn task_hello_world(
    sender: Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
) -> () {
    let hello = get_hello_world();
    log::info!("hello world starting with version {}", hello.version);
    sender.send((String::from("/hello"), Box::new(hello))).unwrap();
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