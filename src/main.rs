mod protos;
mod comboard;

//use rumqtt::{MqttClient, MqttOptions, QoS};
//use std::{thread, time::Duration};
//use protobuf::Message;
use std::sync::mpsc::channel;
use futures::executor::block_on;
use crate::comboard::imple::interface::ComboardClient;


fn main() {
    println!("Hello, world!");

    let d = comboard::getComboardClient();

    let (sender, receiver) = channel();
    let (senderState, receiverState) = channel();
    let (senderValue, receiverValue) = channel();

    d.run(comboard::imple::interface::ComboardClientConfig{
        receiverConfig: receiver,
        senderStateChange: senderState,
        senderValueValidation: senderValue,
    }).join().unwrap();

    
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