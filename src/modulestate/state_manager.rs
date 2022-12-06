use std::collections::HashMap;

pub struct MainboardConnectedModule {
    pub port: i32,
    pub id: String,
    pub board: String,
    pub board_addr: String,
    pub handler_map: std::collections::HashMap<String, tokio_util::sync::CancellationToken>, pub last_value: Option<Box<dyn super::interface::ModuleValueParsable>>,
    pub validator: Box<dyn super::interface::ModuleValueValidator>,
}

pub struct MainboardModuleStateManager {
    pub connected_module: HashMap<String, MainboardConnectedModule>,
}

impl MainboardModuleStateManager {
    pub fn new() -> Self {
        Self {
            connected_module: HashMap::new()
        }
    }

    pub fn get_module_at_index(
        &self,
        board: &String,
        board_addr: &String,
        port: i32,
    ) -> Option<&MainboardConnectedModule> {
        for (_, v) in self.connected_module.iter() {
            if v.port == port && v.board == *board && v.board_addr == *board_addr {
                return Some(&v);
            }
        }
        return None;
    }
    // cheap hack plz can i do better
    pub fn get_module_at_index_mut(
        &mut self,
        board: &String,
        board_addr: &String,
        port: i32,
    ) -> Option<&mut MainboardConnectedModule> {
        let mut id: String = String::from("");
        {
            if let Some(its_a_me_variable) = self.get_module_at_index(board, board_addr, port) {
                id = its_a_me_variable.id.clone();
            }
        }
        return self.connected_module.get_mut(&id);
    }

    pub fn get_connected_modules(&self) -> Vec<String> {
        return Vec::from_iter(self.connected_module.keys().cloned());
    }
}

