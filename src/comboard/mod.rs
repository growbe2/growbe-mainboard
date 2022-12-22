use crate::{protos::board::RunningComboard};

use self::{imple::interface::ComboardClientConfig};

use tokio::sync::mpsc::Receiver;

use self::imple::virt::VirtualScenarioItem;

#[cfg(feature = "com_ble")]
use self::imple::ble::get_ble_comboard;
#[cfg(feature = "com_ws")]
use self::imple::ws::get_ws_comboard;

pub mod config;
pub mod imple;

#[cfg(all(target_os = "linux", feature = "com_i2c"))]
fn get_comboard_i2c(config: ComboardClientConfig) -> imple::i2c_linux::I2CLinuxComboardClient {
    return imple::i2c_linux::I2CLinuxComboardClient {
        config_comboard: config,
    };
}

fn get_comboard_virt(
    config: ComboardClientConfig,
    receiver: Receiver<Vec<VirtualScenarioItem>>,
) -> imple::virt::VirtualComboardClient {
    return imple::virt::VirtualComboardClient {
        config_comboard: config,
        receiver_config: Some(receiver),
    };
}

pub fn get_comboard_client(
    receiver_virt: Receiver<Vec<VirtualScenarioItem>>,
) -> Vec<(RunningComboard, Box<dyn imple::interface::ComboardClient>)> {
    let mut boards: Vec<(RunningComboard, Box<dyn imple::interface::ComboardClient>)> = vec![];

    let virt = crate::mainboardstate::config::CONFIG.comboards.iter().find(|x| x.imple == "virt");
    let i2c = crate::mainboardstate::config::CONFIG.comboards.iter().find(|x| x.imple == "i2c");
    let ble = crate::mainboardstate::config::CONFIG.comboards.iter().find(|x| x.imple == "ble");
    let ws = crate::mainboardstate::config::CONFIG.comboards.iter().find(|x| x.imple == "ws");

    if let Some(element) = virt {
        {
            let bo = Box::new(get_comboard_virt(
                ComboardClientConfig {
                    config: element.config.clone(),
                },
                receiver_virt,
            ));
            boards.push((
                RunningComboard {
                    imple: element.imple.clone(),
                    addr: element.config.clone(),
                    ..Default::default()
                },
                bo,
            ));
        }
    }

    if let Some(_element) = i2c {
        #[cfg(all(target_os = "linux", feature = "com_i2c"))]
        {
            let bo = Box::new(get_comboard_i2c(ComboardClientConfig {
                config: element.config.clone(),
            }));
            boards.push((
                RunningComboard {
                    imple: element.imple.clone(),
                    addr: element.config.clone(),
                    ..Default::default()
                },
                bo,
            ));
            println!("adding i2c");
        }
        #[cfg(not(feature = "com_i2c"))]
        panic!("i2c comboard not compiled in the version");
    } else {
        println!("no i2c");
    }

    if let Some(_element) = ble {
        #[cfg(feature = "com_ble")]
        {
            let bo = get_ble_comboard(element.config.clone());
            boards.push((
                RunningComboard {
                    imple: element.imple.clone(),
                    addr: element.config.clone(),
                    ..Default::default()
                },
                bo,
            ));
        }
        #[cfg(not(feature = "com_ble"))]
        panic!("ble comboard not compiled in the version");
    }

    if let Some(element) = ws {
        #[cfg(feature = "com_ws")]
        {
            let bo = get_ws_comboard(element.config.clone());
            boards.push((
                RunningComboard {
                    imple: element.imple.clone(),
                    addr: element.config.clone(),
                    ..Default::default()
                },
                bo,
            ));
        }
        #[cfg(not(feature = "com_ws"))]
        panic!("ws comboard not compiled in the version");
    }

    return boards;
}
