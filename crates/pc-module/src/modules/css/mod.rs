use std::thread;

use crate::{
    channel::CHANNEL_VALUE,
    protos::module::{ComputerStatsData, CpuData, CpuLoadData, MemoryData, NetworkData},
};

use super::ModuleClient;

use protobuf::Message;
use systemstat::Platform;
pub struct SystemStatsModule {}

impl SystemStatsModule {
    pub fn new() -> Self {
        SystemStatsModule {}
    }
}

impl ModuleClient for SystemStatsModule {
    fn run(
        &self,
        _receiver_config: std::sync::mpsc::Receiver<crate::channel::ModuleConfig>,
    ) -> tokio::task::JoinHandle<Result<(), ()>> {
        return tokio::spawn(async move {
            log::info!("starting module for SystemStats");
            let sys = systemstat::System::new();

            let mut data = ComputerStatsData::new();

            loop {
                if let Ok(uptime) = sys.uptime() {
                    data.set_uptime(uptime.as_secs_f32());
                }

                if let Ok(cpu) = sys.cpu_load_aggregate() {
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                    let cpu = cpu.done().unwrap();
                    let mut cpu_data = CpuData::new();
                    cpu_data.set_user(cpu.user * 100.0);
                    cpu_data.set_idle(cpu.idle * 100.);
                    cpu_data.set_interrupt(cpu.interrupt * 100.);
                    cpu_data.set_nice(cpu.nice * 100.);
                    cpu_data.set_system(cpu.system * 100.);
                    cpu_data.set_temp(sys.cpu_temp().unwrap());
                    data.set_cpu(cpu_data);
                }

                if let Ok(load) = sys.load_average() {
                    let mut cpu_load = CpuLoadData::new();
                    cpu_load.set_five(load.five);
                    cpu_load.set_one(load.one);
                    cpu_load.set_fifteen(load.fifteen);
                    data.set_loadavg(cpu_load);
                }

                if let Ok(swap) = sys.swap() {
                    let mut swap_data = MemoryData::new();

                    swap_data.set_free(swap.free.as_u64() as f32);
                    swap_data.set_total(swap.total.as_u64() as f32);

                    data.set_swap(swap_data);
                }

                if let Ok(mem) = sys.memory() {
                    let mut mem_data = MemoryData::new();

                    mem_data.set_free(mem.free.as_u64() as f32);
                    mem_data.set_total(mem.total.as_u64() as f32);

                    data.set_memory(mem_data);
                }

                if let Ok(netifs) = sys.networks() {
                    data.set_netifs(
                        netifs
                            .values()
                            .filter(|x| x.name.starts_with("w") || x.name.starts_with("e"))
                            .map(|x| {
                                let mut netif = NetworkData::new();

                                let stat = sys.network_stats(&x.name).unwrap();

                                netif.set_rx_bytes(stat.rx_bytes.as_u64() as f32);
                                netif.set_tx_bytes(stat.tx_bytes.as_u64() as f32);

                                netif
                            })
                            .collect(),
                    );
                }

                CHANNEL_VALUE
                    .0
                    .lock()
                    .unwrap()
                    .send(("CSS".to_string(), data.write_to_bytes().unwrap()))
                    .unwrap();

                tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
            }
        });
    }
}
