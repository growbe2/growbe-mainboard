/*use rumqtt::{MqttClient, MqttOptions, QoS};
use std::{thread, time::Duration};

fn main() {
    let mqtt_options = MqttOptions::new("test-pubsub1", "localhost", 1883);
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

    for notification in notifications {
        println!("{:?}", notification)
    }
}*/

mod protos;

use rumqtt::{MqttClient, MqttOptions, QoS};
use std::{thread, time::Duration};
use protobuf::Message;

#[link(name="mainboard_driver")]
extern "C" {
    fn fun1(x: cty::c_int);
}


fn main() {
    println!("Hello, world!");

    unsafe {
        fun1(1);
        fun1(5);
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