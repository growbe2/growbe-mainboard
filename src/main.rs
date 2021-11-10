mod protos;

use rumqtt::{MqttClient, MqttOptions, QoS};
use std::{thread, time::Duration};
use protobuf::Message;


extern fn callback(a: i32) -> () {
    println!("{}",a)
}

#[link(name="mainboard_driver")]
extern "C" {
    fn register_callback(cb: extern fn(i32) -> ());
}


fn main() {
    println!("Hello, world!");

    unsafe {
        register_callback(callback);
    }

    println!("C interloop working");

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
        for i in 0..100 {
            let payload = format!("publish {}", i);
            thread::sleep(sleep_time);
            mqtt_client.publish("hello/world", QoS::AtLeastOnce, false, payload).unwrap();
        }
    });

    println!("MQTT client working");

    for notification in notifications {
        println!("{:?}", notification)
    }

}