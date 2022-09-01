
pub struct PhysicalRelay {
    pub sender: std::sync::mpsc::Sender<crate::comboard::imple::channel::ModuleConfig>,
    pub port: i32,
    pub action_port: usize,
}


impl super::State for PhysicalRelay {
    fn set_state(&mut self, state: u8) -> Result<(),()> {
        let mut buffer = [255; 8];
        super::f(&self.action_port, &mut buffer,state);
        self.sender.send(crate::comboard::imple::channel::ModuleConfig{
            port: self.port,
            data: buffer.try_into().unwrap()
        }).unwrap();
        return Ok(());
    }
}

impl super::Relay for PhysicalRelay {
    fn id(&self) -> String {
        return format!("{}", self.action_port);
    }
    fn clone_me(&self) -> Box<dyn super::Relay> {
        return Box::new(PhysicalRelay{
            sender: self.sender.clone(),
            action_port: self.action_port,
            port: self.port,
        });
    }
}

pub struct ActionPortUnion  {
    pub is_array: bool,
    pub port: usize,
    pub ports: Vec<usize>,
} 

impl ActionPortUnion {
    pub fn new_port(port: usize) -> Self {
        return ActionPortUnion { is_array: false, port: port, ports: vec![] }
    }

    pub fn new_ports(ports: Vec<usize>) -> Self {
        return ActionPortUnion { is_array: true, ports: ports, port: 0 };
    }
}

pub struct BatchPhysicalRelay {
    pub sender: std::sync::mpsc::Sender<crate::comboard::imple::channel::ModuleConfig>,
    pub buffer: [u8; 8],
    pub port: i32,

    // if true , when state is set, send it to the comboard
    pub auto_send: bool,

    pub action_port: ActionPortUnion,
}

impl super::State for BatchPhysicalRelay {
    fn set_state(&mut self, state: u8) -> Result<(),()> {
        if self.action_port.is_array {
            self.action_port.ports.iter().for_each(|x| self.buffer[*x] = state);
        } else {
            self.buffer[self.action_port.port] = state;
        }
        if self.auto_send {
            self.sender.send(crate::comboard::imple::channel::ModuleConfig{
                port: self.port,
                data: self.buffer.try_into().unwrap()
            }).unwrap();

            self.buffer = [255; 8];
        }
        return Ok(());
    }
}

impl super::Relay for BatchPhysicalRelay {
    fn id(&self) -> String {
        if self.action_port.is_array {
            return format!("{:?}", self.action_port.ports);
        }
        return format!("{}", self.action_port.port);
    }
    fn clone_me(&self) -> Box<dyn super::Relay> {
        if self.action_port.is_array {
            return Box::new(BatchPhysicalRelay{
                sender: self.sender.clone(),
                port: self.port,
                auto_send: true,
                buffer: [255; 8],
                action_port: ActionPortUnion { ports: self.action_port.ports.clone(), port: 0, is_array: true, },
            })
        } else {
            return Box::new(PhysicalRelay{
                sender: self.sender.clone(),
                port: self.port,
                action_port: self.action_port.port,
            });
        }
    }
}

impl super::BatchRelay for BatchPhysicalRelay {
    fn execute(&self) -> Result<(),()> {
        self.sender.send(crate::comboard::imple::channel::ModuleConfig{
            port: self.port,
            data: self.buffer.try_into().unwrap()
        }).unwrap();
        return Ok(());
    }
}