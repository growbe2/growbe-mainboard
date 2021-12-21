pub mod calibration;


use protobuf::Message;

use crate::protos::module::{SOILModuleData, SOILModuleConfig, SOILCalibrationStepEvent, CalibrationError, SOILCalibrationStep, CalibrationStep};
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

impl super::interface::ModuleValue for SOILModuleConfig {}

impl super::interface::ModuleValueParsable for SOILModuleConfig {}

impl super::interface::ModuleValue for SOILModuleData {}

impl super::interface::ModuleValueParsable for SOILModuleData {}

impl super::interface::ModuleValueValidator for AASValidator {

    fn convert_to_value(&mut self, value_event: &crate::comboard::imple::interface::ModuleValueValidationEvent) -> Result<Box<dyn super::interface::ModuleValueParsable>, super::interface::ModuleError> {
        let mut data = SOILModuleData::new();

        if value_event.buffer.len() > 350 {
            data.p0 = value_event.buffer[0] as i32;
            data.p1 = value_event.buffer[50] as i32;
            data.p2 = value_event.buffer[100] as i32;
            data.p3 = value_event.buffer[150] as i32;
            data.p4 = value_event.buffer[200] as i32;
            data.p5 = value_event.buffer[250] as i32;
            data.p6 = value_event.buffer[300] as i32;
            data.p7 = value_event.buffer[350] as i32;
            data.timestamp = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() as i32;
        }

        if self.calibration_process.is_some() {
            self.calibration_process.as_mut().unwrap().on_value(data.clone());
            return Err(super::interface::ModuleError::new());
        }

        if self.option_config.is_some() {
            transform_value_with_calibration(&mut data, self.option_config.as_ref().unwrap());
        }

        return Ok(Box::new(data));
    }
    
    fn apply_parse_config(&mut self, port: i32, _t: char, data: std::sync::Arc<Vec<u8>>, _sender_comboard_config: & std::sync::mpsc::Sender<crate::comboard::imple::interface::Module_Config>,
        _map_handler: & mut std::collections::HashMap<i32, tokio_util::sync::CancellationToken>
    ) -> Result<(Box<dyn protobuf::Message>, crate::comboard::imple::interface::Module_Config), super::interface::ModuleError> {
        let config: SOILModuleConfig = SOILModuleConfig::parse_from_bytes(&data).unwrap();

        self.option_config = Some(config.clone());

        return Ok((
            Box::new(config),
            crate::comboard::imple::interface::Module_Config{
                port: port,
                buffer: [255; 8],
            },
        ));
    }

    fn have_data_change(&self, current: &Box<dyn crate::modulestate::interface::ModuleValueParsable>, last: &Box<dyn crate::modulestate::interface::ModuleValueParsable>) -> (bool, Vec<super::alarm::model::ValueChange<i32>>) {
        let current = current.as_any().downcast_ref::<SOILModuleData>().unwrap();
        let last = last.as_any().downcast_ref::<SOILModuleData>().unwrap();

        let mut vec = Vec::new();

        if difference_of(current.p0, last.p0, 2) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "p0".to_string(), current_value: current.p0, previous_value: last.p0});
        }
        if difference_of(current.p1, last.p1, 2) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "p1".to_string(), current_value: current.p1, previous_value: last.p1});
        }
        if difference_of(current.p2, last.p2, 2) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "p2".to_string(), current_value: current.p2, previous_value: last.p2});
        }
        if difference_of(current.p3, last.p3, 2) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "p3".to_string(), current_value: current.p3, previous_value: last.p3});
        }
        if difference_of(current.p4, last.p4, 2) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "p4".to_string(), current_value: current.p4, previous_value: last.p4});
        }
        if difference_of(current.p5, last.p5, 2) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "p5".to_string(), current_value: current.p5, previous_value: last.p5});
        }
        if difference_of(current.p6, last.p6, 2) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "p6".to_string(), current_value: current.p6, previous_value: last.p6});
        }
        if difference_of(current.p7, last.p7, 2) {
            vec.push(super::alarm::model::ValueChange::<i32>{property: "p7".to_string(), current_value: current.p7, previous_value: last.p7});
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
        sender_socket: & std::sync::mpsc::Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    ) -> Result<(Option<Vec<super::interface::ModuleStateCmd>>), ()> {
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
            },
            "setCalibration" => {
                let data = SOILCalibrationStep::parse_from_bytes(&data).unwrap();
                if let Some(process) = self.calibration_process.as_mut() {
                    if (data.requested_step == CalibrationStep::READY_CALIBRATION) {
                        process.stop_record();
                    } else {
                        process.start_record(data.requested_step);
                    }
                    event = process.get_status();
                } else {
                    log::error!("failed to get calibration process for setCalibration");
                }
            },
            "terminateCalibration" => {
                if let Some(process) = self.calibration_process.as_mut() {
                    match process.terminate() {
                        Ok(config) => {
                            self.calibration_process = None;
                            let config_bytes = config.write_to_bytes().unwrap();

                            sender_socket.send((format!("/m/{}/config_updated", module_id), Box::new(config))).unwrap();

                            let cmd = super::interface::ModuleStateCmd{
                                cmd: "mconfig",
                                topic: format!("/{}", module_id),
                                data: std::sync::Arc::new(config_bytes),
                            };
                            return Ok(Some(vec![cmd]));
                        },
                        Err(err) => {
                            log::error!("failed to terminate calibration for {}", module_id);
                        }
                    }
                } else {

                }
            },
            "cancelCalibration" => {
                self.calibration_process = None;
                return Ok((None));
            },
            "statusCalibration" =>  {
                if let Some(process) = self.calibration_process.as_mut() {
                    event = process.get_status();
                } else {
                }
            }
            _ => {
                return Err(());
            }
        }
        sender_socket.send((format!("/m/{}/calibrationEvent", module_id.as_str()), Box::new(event))).unwrap();
        return Ok((None));
    }
}