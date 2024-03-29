pub mod calibration;

use protobuf::Message;

use crate::protos::module::{
    CalibrationError, CalibrationStep, SOILCalibrationStep, SOILCalibrationStepEvent,
    SOILModuleConfig, SOILModuleData,
};
use crate::socket::ss::SenderPayloadData;
use crate::utils::validation::difference_of;

use self::calibration::transform_value_with_calibration;

pub struct AASValidator {
    pub option_config: Option<SOILModuleConfig>,
    pub calibration_process: Option<calibration::CalibrationProcess>,
}

impl AASValidator {
    pub fn new() -> AASValidator {
        return AASValidator {
            option_config: None,
            calibration_process: None,
        };
    }
}

impl crate::modulestate::interface::ModuleValue for SOILModuleConfig {}

impl crate::modulestate::interface::ModuleValueParsable for SOILModuleConfig {}

impl crate::modulestate::interface::ModuleValue for SOILModuleData {}

impl crate::modulestate::interface::ModuleValueParsable for SOILModuleData {}

fn two_u8_to_u16(b1: u8, b2: u8) -> u16 {
    return ((b2 as u16) << 8) | b1 as u16;
}

impl crate::modulestate::interface::ModuleValueValidator for AASValidator {
    fn convert_to_value(
        &mut self,
        value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent,
    ) -> Result<
        Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        crate::modulestate::interface::ModuleError,
    > {
        let mut data = SOILModuleData::new();

        if value_event.buffer.len() >= 72 {
            data.p0 = two_u8_to_u16(value_event.buffer[0], value_event.buffer[1]) as i32;
            data.p1 = two_u8_to_u16(value_event.buffer[10], value_event.buffer[11]) as i32;
            data.p2 = two_u8_to_u16(value_event.buffer[20], value_event.buffer[21]) as i32;
            data.p3 = two_u8_to_u16(value_event.buffer[30], value_event.buffer[31]) as i32;
            data.p4 = two_u8_to_u16(value_event.buffer[40], value_event.buffer[41]) as i32;
            data.p5 = two_u8_to_u16(value_event.buffer[50], value_event.buffer[51]) as i32;
            data.p6 = two_u8_to_u16(value_event.buffer[60], value_event.buffer[61]) as i32;
            data.p7 = two_u8_to_u16(value_event.buffer[70], value_event.buffer[71]) as i32;
            data.timestamp = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32;
            data.valuetype = "raw".to_string();
        }

        if self.calibration_process.is_some() {
            self.calibration_process
                .as_mut()
                .unwrap()
                .on_value(data.clone());
            return Ok(Box::new(data));
        }

        if self.option_config.is_some() {
            transform_value_with_calibration(&mut data, self.option_config.as_ref().unwrap());
        }

        return Ok(Box::new(data));
    }

    fn remove_config(
        &mut self,
        _actor: crate::protos::module::Actor,
    ) -> Result<(), crate::modulestate::interface::ModuleError> {
        self.option_config = None;
        return Ok(());
    }

    fn apply_parse_config(
        &mut self,
        port: i32,
        _t: &str,
        data: std::sync::Arc<Vec<u8>>,
        _sender_comboard_config: &tokio::sync::mpsc::Sender<
            crate::comboard::imple::channel::ModuleConfig,
        >,
        _map_handler: &mut std::collections::HashMap<String, tokio_util::sync::CancellationToken>,
        _actor: crate::protos::module::Actor,
    ) -> Result<
        (
            Box<dyn protobuf::Message>,
            crate::comboard::imple::channel::ModuleConfig,
        ),
        crate::modulestate::interface::ModuleError,
    > {
        let config = SOILModuleConfig::parse_from_bytes(&data);
        if config.is_err() {
            return Err(crate::modulestate::interface::ModuleError::new()
                .message(config.err().unwrap().to_string()));
        }

        let config = config.unwrap();

        self.option_config = Some(config.clone());

        return Ok((
            Box::new(config),
            crate::comboard::imple::channel::ModuleConfig {
                port: port,
                data: [255; 8].try_into().unwrap(),
            },
        ));
    }

    fn have_data_change(
        &self,
        current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>,
        last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>,
    ) -> (
        bool,
        Vec<crate::modulestate::alarm::model::ValueChange<f32>>,
    ) {
        let current = current.as_any().downcast_ref::<SOILModuleData>().unwrap();
        let last = last.as_any().downcast_ref::<SOILModuleData>().unwrap();

        let mut vec = Vec::new();

        if difference_of(current.p0, last.p0, 1) {
            vec.push(crate::modulestate::alarm::model::ValueChange::<f32> {
                property: "p0".to_string(),
                current_value: current.p0 as f32,
                previous_value: last.p0 as f32,
            });
        }
        if difference_of(current.p1, last.p1, 1) {
            vec.push(crate::modulestate::alarm::model::ValueChange::<f32> {
                property: "p1".to_string(),
                current_value: current.p1 as f32,
                previous_value: last.p1 as f32,
            });
        }
        if difference_of(current.p2, last.p2, 1) {
            vec.push(crate::modulestate::alarm::model::ValueChange::<f32> {
                property: "p2".to_string(),
                current_value: current.p2 as f32,
                previous_value: last.p2 as f32,
            });
        }
        if difference_of(current.p3, last.p3, 1) {
            vec.push(crate::modulestate::alarm::model::ValueChange::<f32> {
                property: "p3".to_string(),
                current_value: current.p3 as f32,
                previous_value: last.p3 as f32,
            });
        }
        if difference_of(current.p4, last.p4, 1) {
            vec.push(crate::modulestate::alarm::model::ValueChange::<f32> {
                property: "p4".to_string(),
                current_value: current.p4 as f32,
                previous_value: last.p4 as f32,
            });
        }
        if difference_of(current.p5, last.p5, 1) {
            vec.push(crate::modulestate::alarm::model::ValueChange::<f32> {
                property: "p5".to_string(),
                current_value: current.p5 as f32,
                previous_value: last.p5 as f32,
            });
        }
        if difference_of(current.p6, last.p6, 1) {
            vec.push(crate::modulestate::alarm::model::ValueChange::<f32> {
                property: "p6".to_string(),
                current_value: current.p6 as f32,
                previous_value: last.p6 as f32,
            });
        }
        if difference_of(current.p7, last.p7, 1) {
            vec.push(crate::modulestate::alarm::model::ValueChange::<f32> {
                property: "p7".to_string(),
                current_value: current.p7 as f32,
                previous_value: last.p7 as f32,
            });
        }
        if difference_of(current.timestamp, last.timestamp, 60) {
            return (true, vec);
        }

        return (vec.len() > 0, vec);
    }

    fn handle_command_validator(
        &mut self,
        cmd: &str,
        module_id: &String,
        data: std::sync::Arc<Vec<u8>>,
        sender_response: tokio::sync::oneshot::Sender<crate::protos::message::ActionResponse>,
        sender_socket: &tokio::sync::mpsc::Sender<crate::socket::ss::SenderPayload>,
        actor: crate::protos::module::Actor,
    ) -> Result<
        Option<Vec<crate::modulestate::interface::ModuleStateCmd>>,
        crate::modulestate::interface::ModuleError,
    > {
        let mut event = SOILCalibrationStepEvent::new();
        match cmd {
            "startCalibration" => {
                if let None = self.calibration_process {
                    self.calibration_process = Some(calibration::CalibrationProcess::new());
                    event = self.calibration_process.as_ref().unwrap().get_status();
                } else {
                    let mut process = self.calibration_process.as_mut().unwrap();
                    process.ex = CalibrationError::ALREADY_STARTED_ERROR;
                    event = process.get_status();
                }
            }
            "setCalibration" => {
                let data = SOILCalibrationStep::parse_from_bytes(&data).unwrap();
                if let Some(process) = self.calibration_process.as_mut() {
                    if data.requested_step == CalibrationStep::READY_CALIBRATION {
                        process.stop_record();
                    } else {
                        process.start_record(data.requested_step);
                    }
                    event = process.get_status();
                } else {
                    log::error!("failed to get calibration process for setCalibration");
                }
            }
            "terminateCalibration" => {
                if let Some(process) = self.calibration_process.as_mut() {
                    match process.terminate() {
                        Ok(config) => {
                            self.calibration_process = None;
                            let config_bytes = config.write_to_bytes().unwrap();

                            sender_socket.try_send((
                                format!("/m/{}/config_updated", module_id),
                                SenderPayloadData::ProtobufMessage(Box::new(config)),
                            ));

                            let cmd = crate::modulestate::interface::ModuleStateCmd {
                                cmd: "mconfig".into(),
                                topic: format!("/{}", module_id),
                                data: std::sync::Arc::new(config_bytes),
                                sender: sender_response,
                                actor,
                            };
                            return Ok(Some(vec![cmd]));
                        }
                        Err(_err) => {
                            log::error!("failed to terminate calibration for {}", module_id);
                        }
                    }
                } else {
                }
            }
            "cancelCalibration" => {
                self.calibration_process = None;
                return Ok(None);
            }
            "statusCalibration" => {
                if let Some(process) = self.calibration_process.as_mut() {
                    event = process.get_status();
                } else {
                }
            }
            _ => {
                return Err(crate::modulestate::interface::ModuleError::new()
                    .status(crate::modulestate::interface::CMD_NOT_SUPPORTED));
            }
        }
        sender_socket.try_send((
            format!("/m/{}/calibrationEvent", module_id.as_str()),
            SenderPayloadData::ProtobufMessage(Box::new(event)),
        ));
        return Ok(None);
    }

    fn edit_ownership(
        &mut self,
        config: Box<dyn protobuf::Message>,
        request: crate::protos::module::ModuleActorOwnershipRequest,
        actor: &crate::protos::module::Actor,
    ) -> Result<Box<dyn protobuf::Message>, crate::modulestate::interface::ModuleError> {
        return Ok(config);
    }
}
