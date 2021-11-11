

pub struct VirtualComboardClient {}

impl super::interface::ComboardClient for VirtualComboardClient {
    fn run(&self, config: super::interface::ComboardClientConfig) -> tokio::task::JoinHandle<()> {
        return tokio::spawn(async {
            println!("Starting virtual truc mush");
        });
    }
}
