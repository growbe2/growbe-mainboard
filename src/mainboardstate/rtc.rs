use chrono::prelude::*;

const DATE_FORMAT_STR: &'static str = "%H:%M:%S %m/%d/%Y";

pub fn get_rtc_format() -> String {
    return get_rtc().format(DATE_FORMAT_STR).to_string();    
}

pub fn get_rtc() -> chrono::DateTime<Local> {
    return Local::now();
}

#[cfg(target_arch = "x86_64")]
pub fn set_rtc(rtc: crate::protos::message::RTCTime) -> () {
    println!("Calling set RTC does nothing {:?}", rtc);
} 

#[cfg(target_arch = "arm")]
pub fn set_rtc(str: String) -> () {
    // change the time of the clock
} 