


pub struct PhysicalRelay {
    pub sender: std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    pub port: i32,
    pub action_port: usize,
}


impl super::State for PhysicalRelay {
    fn set_state(&mut self, state: u8) -> Result<(),()> {
        let mut buffer = [255; 8];
        super::f(&self.action_port, &mut buffer,state);
        self.sender.send(crate::comboard::imple::interface::Module_Config{
            port: self.port,
            buffer: buffer
        }).unwrap();
        return Ok(());
    }
}

impl super::Relay for PhysicalRelay {
    fn id(&self) -> String {
        return format!("{}", self.action_port);
    }
    fn clone(&self) -> Box<dyn super::Relay> {
        return Box::new(PhysicalRelay{
            sender: self.sender.clone(),
            action_port: self.action_port,
            port: self.port,
        });
    }
}

pub struct BatchPhysicalRelay {
    pub sender: std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
    pub buffer: [u8; 8],
    pub port: i32,

    pub action_port: usize,
}

impl super::State for BatchPhysicalRelay {
    fn set_state(&mut self, state: u8) -> Result<(),()> {
        self.buffer[self.action_port] = state;
        return Ok(());
    }
}

impl super::Relay for BatchPhysicalRelay {
    fn id(&self) -> String {
        return format!("{}", self.action_port);
    }
    fn clone(&self) -> Box<dyn super::Relay> {
        return Box::new(PhysicalRelay{
            sender: self.sender.clone(),
            port: self.port,
            action_port: self.action_port,
        });
    }
}

impl super::BatchRelay for BatchPhysicalRelay {
    fn execute(&self) -> Result<(),()> {
        self.sender.send(crate::comboard::imple::interface::Module_Config{
            port: self.port,
            buffer: self.buffer
        }).unwrap();
        return Ok(());
    }
}