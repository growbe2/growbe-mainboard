use crate::mainboardstate::error::MainboardError;

pub trait ModuleValue {}
pub trait ModuleValueParsable: ModuleValue + protobuf::Message {}

impl ModuleValue for crate::protos::module::ModuleData {}
impl ModuleValueParsable for crate::protos::module::ModuleData {}

pub const MODULE_NOT_FOUND: u32 = 404;
pub const SENDER_NOT_FOUND: u32 = 405;
pub const CMD_NOT_SUPPORTED: u32 = 400;

#[derive(Debug, Clone)]
pub struct ModuleError {
    pub message: String,
    pub module_id: String,
    pub port: i32,
    pub status: u32,
}

pub fn conv_err(module_id: String) -> impl Fn(MainboardError) -> ModuleError {
    return move |err| {
        ModuleError::new().message(err.message).module_id(module_id.clone())
    }
}

impl ModuleError {
    pub fn new() -> ModuleError {
        return ModuleError {
            message: String::from(""),
            module_id: String::from(""),
            port: -1,
            status: 0,
        };
    }

    pub fn not_found(module_id: &str) -> ModuleError {
        return ModuleError {
            message: String::from("module not found"),
            module_id: module_id.to_string(),
            port: -1,
            status: MODULE_NOT_FOUND,
        };
    }

    pub fn from_protobuf_err(module_id: &str, err: protobuf::ProtobufError) -> Self {
        return Self {
            message: err.to_string(),
            module_id: module_id.to_string(),
            port: -1,
            status: MODULE_NOT_FOUND,
        };
    }

    pub fn from_rusqlite_err(module_id: &str, err: rusqlite::Error) -> Self {
        return Self {
            message: err.to_string(),
            module_id: module_id.to_string(),
            port: -1,
            status: MODULE_NOT_FOUND,
        };
   }

    pub fn sender_not_found(module_id: &str) -> ModuleError {
        return ModuleError {
            message: String::from("sender not found"),
            module_id: module_id.to_string(),
            port: -1,
            status: SENDER_NOT_FOUND,
        };
    }

    pub fn message(mut self, message: String) -> ModuleError {
        self.message = message;
        self
    }

    pub fn status(mut self, status: u32) -> ModuleError {
        self.status = status;
        self
    }

    pub fn module_id(mut self, module_id: String) -> ModuleError {
        self.module_id = module_id;
        self
    }
}

impl From<rusqlite::Error> for ModuleError {
    fn from(value: rusqlite::Error) -> Self {
         return Self {
            message: value.to_string(),
            module_id: "".to_string(),
            port: -1,
            status: SENDER_NOT_FOUND,
        };
    }
}



pub struct ModuleStateCmd {
    pub cmd: &'static str,
    pub topic: String,
    pub data: std::sync::Arc<Vec<u8>>,
    pub sender: std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}] at {} : {}",
            self.module_id,
            self.port,
            self.message.as_str()
        )
    }
}

pub trait Downcast {
    fn as_any(self: &'_ Self) -> &'_ dyn std::any::Any
    where
        Self: 'static;

    fn as_any_mut(self: &'_ mut Self) -> &'_ mut dyn std::any::Any
    where
        Self: 'static;

    // others if needed
}
impl<T> Downcast for T {
    fn as_any(self: &'_ Self) -> &'_ dyn std::any::Any
    where
        Self: 'static,
    {
        self
    }

    fn as_any_mut(self: &'_ mut Self) -> &'_ mut dyn std::any::Any
    where
        Self: 'static,
    {
        self
    }

    // ...
}

pub trait ModuleValueValidator: Downcast {
    // need to be option result
    fn convert_to_value(
        &mut self,
        value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent,
    ) -> Result<Box<dyn ModuleValueParsable>, ModuleError>;

    fn have_data_change(
        &self,
        current: &Box<dyn ModuleValueParsable>,
        last: &Box<dyn ModuleValueParsable>,
    ) -> (bool, Vec<super::alarm::model::ValueChange<f32>>);

    fn handle_command_validator(
        &mut self,
        cmd: &str,
        module_id: &String,
        data: std::sync::Arc<Vec<u8>>,
        sender_response: &std::sync::mpsc::Sender<crate::protos::message::ActionResponse>,
        sender_socket: &std::sync::mpsc::Sender<(String, Box<dyn ModuleValueParsable>)>,
    ) -> Result<Option<Vec<ModuleStateCmd>>, ModuleError>;

    // need to be option result
    fn apply_parse_config(
        &mut self,
        port: i32,
        t: &str,
        data: std::sync::Arc<Vec<u8>>,
        sender_comboard_config: &std::sync::mpsc::Sender<
            crate::comboard::imple::channel::ModuleConfig,
        >,
        map_handler: &mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
    ) -> Result<
        (
            Box<dyn protobuf::Message>,
            crate::comboard::imple::channel::ModuleConfig,
        ),
        ModuleError,
    >;

    fn remove_config(&mut self) -> Result<(), ModuleError>;
}
