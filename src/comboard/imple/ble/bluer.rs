
use bluer::{gatt::remote::Characteristic, AdapterEvent, Device, Result};
use futures::{pin_mut, StreamExt};

use super::{GROWBE_ANDROID_MODULE_SERVICE, get_devices};



async fn find_our_service(device: &Device, config: &Option<Vec<String>>) -> Result<Option<Vec<Characteristic>>> {
	let addr = device.address();
	let uuids = device.uuids().await?.unwrap_or_default();

	if let Some(addrs) = config {
        if let None = addrs.iter().find(|&x| addr.to_string().eq(x)) {
			return Ok(None);
        }
    }

	log::info!("found device with our services : {} {:?} {}", addr, uuids, GROWBE_ANDROID_MODULE_SERVICE);
	if uuids.contains(&GROWBE_ANDROID_MODULE_SERVICE) {
		
		if !device.is_connected().await? {
			log::info!("trying to connect to {}", addr);
			let mut retries = 3;
			loop {
				match device.connect().await {
					Ok(()) => break,
					Err(err) if retries > 0 => {
						log::warn!("connect error: {}", &err);
						retries -= 1;
					},
					Err(err) => return Err(err),
				}
			}
			log::info!("connected to {}", addr);
		} else {
			log::info!("already connected to {}", addr);
		}
		
		tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

		for service in device.services().await? {
			let uuid = service.uuid().await?;
			if uuid == GROWBE_ANDROID_MODULE_SERVICE {
				let mut characterisitcs: Vec<Characteristic> = vec![];
				for char in service.characteristics().await? {
					characterisitcs.push(char);
				}

				return Ok(Some(characterisitcs));
			}
		}

		println!("FAFA");
	}

	return Ok(None);
}


async fn bt_task(config: String) -> bluer::Result<()> {

	let devices = get_devices(config);


	let session = bluer::Session::new().await?;
	let adapter = session.default_adapter().await?;

	adapter.set_powered(true).await?;

	{
		log::info!(
			"Starting discovery on Bluetooth adapter {} with address {}",
			adapter.name(),
			adapter.address().await?
		);

		let discover = adapter.discover_devices().await?;

		pin_mut!(discover);

		while let Some(evt) = discover.next().await {
			match evt {
				AdapterEvent::DeviceAdded(addr) => {
					let device = adapter.device(addr)?;
					match find_our_service(&device, &devices).await {
						Ok(Some(chars)) => {
							log::info!("found chars {:?}", chars);
						}
						Ok(None) => {
						},
						Err(err) => {
							log::error!("{:?}", err);
						}
					}

				}
				AdapterEvent::DeviceRemoved(addr) => {
                    println!("Device removed {}", addr);
                }
                _ => (),
			}
		}
	}
	Ok(())
}

impl crate::comboard::imple::interface::ComboardClient for super::BLEComboardClient {
    fn run(&self) -> tokio::task::JoinHandle<()> {
		let config = self.config_comboard.config.clone();
        return tokio::spawn(async move {
			match bt_task(config).await {
				Ok(_) => {},
				Err(e) => log::error!("{:?}", e),
			}
		});
	}
}