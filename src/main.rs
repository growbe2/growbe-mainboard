mod protos;

use rumqtt::{MqttClient, MqttOptions, QoS};
use std::{thread, time::Duration};
use protobuf::Message;
use futures::executor::block_on;

use std::ffi::CStr;
use std::str;

#[repr(C)]
pub struct Module_Config {
    pub port: cty::c_int,
    pub buffer: [cty::uint8_t; 512],
}

extern fn callback_state_changed(port: i32, id: *const ::std::os::raw::c_char, state: bool) -> () {
    let c_str: &CStr = unsafe { CStr::from_ptr(id) };
    let str_slice: &str = c_str.to_str().unwrap();
    println!("{} {} {}",port, str_slice, state)
}

extern fn callback_value_validation(port: i32, buffer: &[::std::os::raw::c_char; 512]) -> () {
    println!("Validation {}, first 5 byte, {} {} {} {} {}", port, buffer[0], buffer[1], buffer[2], buffer[3], buffer[4])
}

extern fn callback_config(config: *mut Module_Config) {
    println!("muttable");
    if !config.is_null() {
        unsafe {
            (*config).port = 50;
        }
    } 
}

#[link(name="mainboard_driver")]
extern "C" {
    fn register_callback(
        cb: extern fn(i32,*const ::std::os::raw::c_char,bool) -> (),
        cb1: extern fn(i32, &[::std::os::raw::c_char; 512]),
        cb2: extern fn( *mut Module_Config)
    );

    fn comboard_loop_body();
}

async fn thread_comboard() {
        unsafe {
            register_callback(callback_state_changed, callback_value_validation, callback_config);
        }
            for _x in [0; 3] {
                thread::sleep(Duration::from_secs(1));
                unsafe {
                    comboard_loop_body();
                }
            }
}


fn main() {
    println!("Hello, world!");

    block_on(thread_comboard());

    println!("C interloop working");

    /*
    let mut out_msg = protos::board::GrowbeMainboardConfig::new();
    out_msg.hearthBeath = 5;

    let out_bytes: Vec<u8> = out_msg.write_to_bytes().unwrap();

    let in_msg = protos::board::GrowbeMainboardConfig::parse_from_bytes(&out_bytes).unwrap();

    assert_eq!(out_msg.hearthBeath, in_msg.hearthBeath);

    println!("Protobuf parsing working");

    let mqtt_options = MqttOptions::new("test-pubsub1", "broker.dev.growbe.ca", 1883);
    let (mut mqtt_client, notifications) = MqttClient::start(mqtt_options).unwrap();
      
    mqtt_client.subscribe("hello/world", QoS::AtLeastOnce).unwrap();
    let sleep_time = Duration::from_secs(1);
    thread::spawn(move || {
        for i in 0..2 {
            let payload = format!("publish {}", i);
            thread::sleep(sleep_time);
            mqtt_client.publish("hello/world", QoS::AtLeastOnce, false, payload).unwrap();
        }
    });

    println!("MQTT client working");

    for notification in notifications {
        println!("{:?}", notification)
    }
    */

}