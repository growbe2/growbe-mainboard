use btleplug::platform::{Adapter, PeripheralId};
use lazy_static::lazy_static;
use std::collections::HashMap;

use tokio::select;
use tokio_stream::StreamExt;
use btleplug::api::{
    Central, Manager as _, Peripheral, ScanFilter, CentralEvent,
    CentralEvent::DeviceDisconnected
};
use btleplug::platform::Manager;

use uuid::Uuid;

use crate::comboard::imple::channel::{comboard_send_value, comboard_send_state};

const GROWBE_ANDROID_MODULE_SERVICE: Uuid = uuid::uuid!("00000000-0000-0000-0000-ffff00000001");
const AND_SUPPORTED_MODULES_ID_CHARACTERISTIC: Uuid = uuid::uuid!("00000000-0000-0000-0000-ffff00000000");
const AND_MODULE_ID_CHARACTERISTIC: Uuid = uuid::uuid!("00000000-0000-0000-0000-ffff00000002");
const AND_POSITION_CHARACTERISTIC: Uuid = uuid::uuid!("00000000-0000-0000-0000-ffff00000003");
const AND_ACCELERATION_CHARACTERISTIC: Uuid = uuid::uuid!("00000000-0000-0000-0000-ffff00000004");
const AND_LIGHT_CHARACTERISTIC: Uuid = uuid::uuid!("00000000-0000-0000-0000-ffff00000005");
const AND_PRESSURE_CHARACTERISTIC: Uuid = uuid::uuid!("00000000-0000-0000-0000-ffff00000006");

lazy_static! {
    static ref AND_MODULE_CHARACTERISTICS: Vec<Uuid> = vec![
        AND_POSITION_CHARACTERISTIC,
        AND_ACCELERATION_CHARACTERISTIC,
        AND_PRESSURE_CHARACTERISTIC,
        AND_LIGHT_CHARACTERISTIC
    ];
}


#[derive(Debug)]
pub struct BLEConnectedModule {
    pub id: String,
    pub activated_modules: Vec<String>,
    pub supported_modules: Vec<String>
}

pub struct BLEComboardClient {
    pub config_comboard: super::interface::ComboardClientConfig,
}

fn on_module_disconnect(d: &PeripheralId, modules: &mut HashMap<PeripheralId, BLEConnectedModule>) {
    if let Some(module) = modules.get(d) {
        for (i, m) in module.activated_modules.iter().enumerate() {
            let module_id = m.clone() + &module.id;
            comboard_send_state("ble".to_string(), "0".to_string(), i as i32, module_id, false).unwrap();
        }
    }
    modules.remove(&d);
}

async fn read_connected_modules(
    adapter: &Adapter,
    modules: &mut HashMap<PeripheralId, BLEConnectedModule>,
) {
    let peripherals = adapter.peripherals().await.unwrap();

    'peripheral: for peripheral in peripherals.iter() {
        
        if let Some(module) = modules.get(&peripheral.id()) {
            println!("READING FROM ONE");

            select! {
                _ =  peripheral.discover_services() => {},
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(4000)) => {
                    on_module_disconnect(&peripheral.id(), modules);
                    println!("Disconnecting from peripheral {:?}...", peripheral.id());
                    peripheral
                        .disconnect()
                        .await
                        .expect("Error disconnecting from BLE peripheral");
                    continue 'peripheral;
                }
            }

            let mut found_service = false;

            for service in peripheral.services() {
                if service.uuid != GROWBE_ANDROID_MODULE_SERVICE {
                    continue;
                }
                found_service = true;
                for characteristic in service.characteristics {
                    let value = select! {
                        v = peripheral.read(&characteristic) => { 
                            if let Ok(value) = v {
                                value
                            } else {
                                on_module_disconnect(&peripheral.id(), modules);
                                println!("Disconnecting from peripheral {:?}...", peripheral.id());
                                peripheral
                                    .disconnect()
                                    .await
                                    .expect("Error disconnecting from BLE peripheral");
                                continue 'peripheral;
                            }
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(3000)) => {
                            on_module_disconnect(&peripheral.id(), modules);
                            peripheral
                                .disconnect()
                                .await
                                .expect("Error disconnecting from BLE peripheral");
                            continue 'peripheral;
                        }
                    };

                    for (i, v) in AND_MODULE_CHARACTERISTICS.iter().enumerate() {
                        if *v == characteristic.uuid {
                            comboard_send_value("ble".to_string(), "0".to_string(), i as i32, value.clone()).unwrap();
                        }
                    }
                }
            }

            if !found_service {
                // Service not longer exists
                println!("SERVICE IS NOT THERE ANYMORE");
            }

       }
    }

}

async fn try_find_growbe_module(
    adapter: &Adapter,
    modules: &mut HashMap<PeripheralId, BLEConnectedModule>,
    not_modules: &mut HashMap<PeripheralId, i32>,
) {
    // TRY TO FIND GROWBE MODULE
    let peripherals = adapter.peripherals().await.unwrap();

    println!("{} peripherals", peripherals.len());

    for peripheral in peripherals.iter() {
        let is_connected = peripheral.is_connected().await.unwrap();

        if modules.contains_key(&peripheral.id()) || not_modules.contains_key(&peripheral.id()) {
            continue;
        }

        println!(
            "Peripheral {:?} is connected: {:?}",
            peripheral.id(), is_connected
        );


        // Regarde s'il est pas deja connecter ou si pas deja ignorer
        if !is_connected {
            println!("Connecting to peripheral {:?}...", &peripheral.id());
            // I'm stuck need to put a fucking select!j
            select! {
                result = peripheral.connect() => {
                    println!("CONNECTED");
                    if let Err(err) = result {
                        eprintln!("Error connecting to peripheral, skipping: {}", err);
                        continue;
                    }
                },
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(5000)) => {
                    eprintln!("Error is connecting");
                    continue;
                }
            }
        }
        let is_connected = select! {
            is_connected = peripheral.is_connected() => {
                is_connected.unwrap()
            },
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                eprintln!("Error is connected");
                continue;
            }
        };
        println!(
            "Now connected ({:?}) to peripheral {:?}...",
            is_connected, peripheral.id()
        );
        peripheral.discover_services().await.unwrap();

        let mut is_and_module = false;
        let mut module_id: String = String::new();
        let mut supported_module: Vec<String> = vec![];

        for service in peripheral.services() {
            if !service.uuid.eq(&GROWBE_ANDROID_MODULE_SERVICE) {
                continue;
            }

            is_and_module = true;

            println!(
                "Service UUID {}, primary: {}",
                service.uuid, service.primary
            );
            for characteristic in service.characteristics {

                if characteristic.uuid == AND_MODULE_ID_CHARACTERISTIC {
                    let value = select! {
                        v = peripheral.read(&characteristic) => {
                            v.unwrap()
                        },
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000)) => {
                            log::error!("Error timeout reading {}", characteristic.uuid);
                            is_and_module = false;
                            continue;
                        }
                    };
                    module_id = std::str::from_utf8(&value).unwrap().to_string();
                }

                if characteristic.uuid == AND_SUPPORTED_MODULES_ID_CHARACTERISTIC {
                    let value = select! {
                        v = peripheral.read(&characteristic) => {
                            v.unwrap()
                        },
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000)) => {
                            log::error!("Error timeout reading {}", characteristic.uuid);
                            is_and_module = false;
                            continue;
                        }

                    };
                    let str = std::str::from_utf8(&value).unwrap().to_string();
                    let split_str =  str.split(";");
                    supported_module = split_str.into_iter().map(|x| x.to_string()).collect();
                }
            }
        }
        if is_and_module {
            log::info!("Adding module {:?} with sub-modules : {:?}", module_id.clone(), supported_module);
            modules.insert(peripheral.id(), BLEConnectedModule {id: module_id.clone(), activated_modules: supported_module.clone(), supported_modules: supported_module.clone() });
            for (i, module_type) in supported_module.iter().enumerate() {
                let id = module_type.clone() + &module_id;
                comboard_send_state("ble".to_string(), "0".to_string(), i as i32, id.clone(), true).unwrap();
            }
        } else {
            // not_modules.insert(peripheral.id(), 0);
            if is_connected {
                println!("Disconnecting from peripheral {:?}...", peripheral.id());
                peripheral
                    .disconnect()
                    .await
                    .expect("Error disconnecting from BLE peripheral");
            }
        }
    }
}



impl super::interface::ComboardClient for BLEComboardClient {
    fn run(&self) -> tokio::task::JoinHandle<()> {

        return tokio::spawn(async move {
            let manager = Manager::new().await.unwrap();
            let adapter_list = manager.adapters().await.unwrap();
            if adapter_list.is_empty() {
                eprintln!("No Bluetooth adapters found");
            }

            let timeout_read = tokio::time::Duration::from_secs(10);

            let adapter = adapter_list.first().unwrap();

            // Map with the modules connected that i read data from
            let mut modules: HashMap<PeripheralId, BLEConnectedModule> = HashMap::new();
            // Map for the module 
            let mut not_modules: HashMap<PeripheralId, i32> = HashMap::new();

            log::debug!(
                "Starting scan on {}...",
                adapter.adapter_info().await.unwrap()
            );
            adapter
                .start_scan(ScanFilter::default())
                .await
                .expect("Can't scan BLE adapter for connected devices...");

            let mut i = 0;

            let mut events = adapter.events().await.unwrap().filter(|x| match x { CentralEvent::DeviceDisconnected(d) => true, _ => false});

            loop {
                select! {
                    Some(event) = events.next() => {
                        match event {
                            DeviceDisconnected(d) => {
                                println!("Disconnect {:?}", d);
                                on_module_disconnect(&d, &mut modules);
                            },
                            _ => {}
                        }
                    },
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(40)) => {}
                }
                select! {
                    _ = tokio::time::sleep(timeout_read) => {

                        if i == 0 {
                            try_find_growbe_module(&adapter, &mut modules, &mut not_modules).await;
                        }

                        read_connected_modules(&adapter, &mut modules).await;

                        if i > 5 {
                            i = 0;
                        } else {
                            i += 1;
                        }
                    },
                }
            }
        });
    }
}
