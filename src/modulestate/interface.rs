pub trait ModuleValue {}
pub trait ModuleValueParsable: ModuleValue + protobuf::Message {}

impl ModuleValue for crate::protos::module::ModuleData {}
impl ModuleValueParsable for crate::protos::module::ModuleData {}

#[derive(Debug, Clone)]
pub struct ModuleError {
    pub message: String,
    pub module_id: String,
    pub port: i32,
}

impl ModuleError {
    pub fn new() -> ModuleError {
        return ModuleError{
            message: String::from(""),
            module_id: String::from(""),
            port: -1,
        };
    }

    pub fn message(mut self, message: String) -> ModuleError {
       self.message = message;
       self 
    }

    pub fn module_id(mut self, module_id: String) -> ModuleError {
        self.module_id = module_id;
        self
    }

    pub fn port(mut self, port: i32) -> ModuleError {
        self.port = port;
        self
    }
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}] at {} : {}", self.module_id, self.port, self.message.as_str())
    }
}

pub trait ModuleValueValidator {
    // need to be option result
    fn convert_to_value(&self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn ModuleValueParsable>, ModuleError>;

    fn have_data_change(&self, current: &Box<dyn ModuleValueParsable>, last: &Box<dyn ModuleValueParsable>) -> (bool, Vec<super::alarm::model::ValueChange<i32>>);

    // need to be option result
    fn apply_parse_config(&mut self, port: i32, t: char, data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        map_handler: & mut std::collections::HashMap<i32, tokio_util::sync::CancellationToken>,
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), ModuleError>;
}