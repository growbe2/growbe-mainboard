
pub struct VirtualComboardClient {}

impl super::interface::ComboardClient for VirtualComboardClient {
    fn run(&self, config: super::interface::ComboardClientConfig) -> std::thread::JoinHandle<()> {
        return std::thread::spawn(|| {

        });
    }
}
