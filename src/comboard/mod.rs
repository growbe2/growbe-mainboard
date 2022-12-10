use crate::protos::board::RunningComboard;

use self::imple::interface::ComboardClientConfig;

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

#[cfg(feature = "com_virt")]
fn get_comboard_virt(config: ComboardClientConfig) -> imple::virt::VirtualComboardClient {
    return imple::virt::VirtualComboardClient {
        config_comboard: config,
    };
}

pub fn get_comboard_client() -> Vec<(RunningComboard, Box<dyn imple::interface::ComboardClient>)> {
    let mut boards: Vec<(RunningComboard, Box<dyn imple::interface::ComboardClient>)> = vec![];

    for element in crate::mainboardstate::config::CONFIG.comboards.iter() {
        let mut board: Vec<Box<dyn imple::interface::ComboardClient>> = vec![];

        if element.imple == "virt" {
            #[cfg(feature = "com_virt")]
            board.push(Box::new(get_comboard_virt(ComboardClientConfig {
                config: element.config.clone(),
            })));
            #[cfg(not(feature = "com_virt"))]
            panic!("virtual comboard not compiled in the version");
        } else if element.imple == "i2c" {
            #[cfg(all(target_os = "linux", feature = "com_i2c"))]
            board.push(Box::new(get_comboard_i2c(ComboardClientConfig {
                config: element.config.clone(),
            })));
            #[cfg(not(feature = "com_i2c"))]
            panic!("i2c comboard not compiled in the version");
        } else if element.imple == "ble" {
            #[cfg(feature = "com_ble")]
            board.push(get_ble_comboard(element.config.clone()));
            #[cfg(not(feature = "com_ble"))]
            panic!("ble comboard not compiled in the version");
        } else if element.imple == "ws" {
            #[cfg(feature = "com_ws")]
            board.push(get_ws_comboard(element.config.clone()));
            #[cfg(not(feature = "com_ws"))]
            panic!("ws comboard not compiled in the version");
        };

        if board.len() == 1 {
            boards.push((
                RunningComboard {
                    imple: element.imple.clone(),
                    addr: element.config.clone(),
                    ..Default::default()
                },
                board.pop().unwrap(),
            ));
        } else {
            panic!(
                "comboard is not supported {} : either it does not exists or it's not build in",
                element.imple
            );
        }
    }

    return boards;
}
