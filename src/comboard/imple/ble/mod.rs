
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub mod btleplug;
#[cfg(target_os = "linux")]
pub mod bluer;



use lazy_static::lazy_static;
use uuid::Uuid;

use super::interface::ComboardClientConfig;

const GROWBE_ANDROID_MODULE_SERVICE: Uuid = uuid::Uuid::from_u128(0xFEEDC0DE00002);
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

pub fn get_devices(str: String) -> Option<Vec<String>> {
    let addr: Vec<String> = str.split(";").map(|x| String::from(x)).collect();
    return if addr.len() == 0 { None } else { Some(addr) };
}

pub fn get_ble_comboard(config: String) -> Box<BLEComboardClient> {
    return Box::new(BLEComboardClient{config_comboard: ComboardClientConfig { config: config }});
}