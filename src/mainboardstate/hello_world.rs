use std::sync::mpsc::Sender;
use crate::protos::board::HelloWord;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

impl crate::modulestate::interface::ModuleValue for crate::protos::board::HelloWord {}
impl crate::modulestate::interface::ModuleValueParsable for crate::protos::board::HelloWord {}

pub async fn task_hello_world(
    sender: Sender<(String, Box<dyn crate::modulestate::interface::ModuleValueParsable>)>,
) -> () {
    let mut hello = HelloWord::new();
    hello.cloudVersion = String::from("UNKNOWN");
    hello.version = String::from(VERSION);
    hello.RTC = super::rtc::get_rtc_format();
    sender.send((String::from("/hello"), Box::new(hello))).unwrap();
}