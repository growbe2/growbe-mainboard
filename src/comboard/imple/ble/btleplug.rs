use btleplug::platform::{Adapter, PeripheralId};
use std::collections::HashMap;

use std::sync::mpsc::Receiver;

use tokio::select;
use tokio_stream::StreamExt;
use btleplug::api::{
    Central, Manager as _, Peripheral, ScanFilter, CentralEvent,
    CentralEvent::DeviceDisconnected
};
use btleplug::platform::Manager;

use super::{
	get_devices,
	BLEComboardClient,
	BLEConnectedModule,
	GROWBE_ANDROID_MODULE_SERVICE,
	AND_SUPPORTED_MODULES_ID_CHARACTERISTIC,
	AND_MODULE_CHARACTERISTICS,
	AND_MODULE_ID_CHARACTERISTIC
};


use crate::comboard::imple::channel::{comboard_send_value, comboard_send_state};

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
            select! {
                _ =  peripheral.discover_services() => {},
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(10000)) => {
                    on_module_disconnect(&peripheral.id(), modules);
                    log::error!("timeout discovering service {:?}", peripheral.id());
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
                                log::error!("error reading characteritics {:?}", peripheral.id());
                                peripheral
                                    .disconnect()
                                    .await
                                    .expect("Error disconnecting from BLE peripheral");
                                continue 'peripheral;
                            }
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_millis(3000)) => {
                            log::error!("timeout reading characteristic {:?}", peripheral.id());
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
                log::error!("service cant be found anymore")
            }

       }
    }

}

async fn try_find_growbe_module(
    adapter: &Adapter,
    config: &Option<Vec<String>>,
    modules: &mut HashMap<PeripheralId, BLEConnectedModule>,
    not_modules: &mut HashMap<PeripheralId, i32>,
) {
    // TRY TO FIND GROWBE MODULE
    let peripherals = adapter.peripherals().await.unwrap();
    
    for peripheral in peripherals.iter() {
        let already_is_connected = peripheral.is_connected().await.unwrap();

        if modules.contains_key(&peripheral.id()) || not_modules.contains_key(&peripheral.id()) {
            continue;
        }
        
        if let Some(addrs) = config {
            let addr = peripheral.address().to_string();
            if let Some(_) = addrs.iter().find(|&x| addr.eq(x)) {
            } else {
                continue;
            }
        }

        log::info!(
            "try connection with {:?} is connected: {:?}",
            peripheral.id(), already_is_connected
        );


        // Regarde s'il est pas deja connecter ou si pas deja ignorer
        if !already_is_connected {
            // I'm stuck need to put a fucking select!j
            select! {
                result = peripheral.connect() => {
                    if let Err(err) = result {
                        log::error!("Error connecting to peripheral, skipping: {}", err);
                        continue;
                    }
                },
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(50000)) => {
                    log::error!("timeout connecting");
                    continue;
                }
            }
        }
        let is_connected = select! {
            is_connected = peripheral.is_connected() => {
                is_connected.unwrap()
            },
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(10000)) => {
                log::error!("timeout is connecting");
                continue;
            }
        };
        log::info!(
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

            log::info!(
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
            if !already_is_connected && is_connected {
                log::info!("Disconnecting from peripheral this is not a android module {:?}...", peripheral.id());
                peripheral
                    .disconnect()
                    .await
                    .expect("Error disconnecting from BLE peripheral");
            }
        }
    }
}



impl crate::comboard::imple::interface::ComboardClient for BLEComboardClient {
    fn run(&self, receiver_config: Receiver<crate::comboard::imple::channel::ModuleConfig>) -> tokio::task::JoinHandle<()> {
        let devices = get_devices(self.config_comboard.config.clone());

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
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {}
                }
                select! {
                    _ = tokio::time::sleep(timeout_read) => {

                        if i == 0 {
                            try_find_growbe_module(&adapter, &devices, &mut modules, &mut not_modules).await;
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
