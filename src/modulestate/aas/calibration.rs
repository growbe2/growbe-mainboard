
use crate::protos::module::{SOILModuleData, SOILProbeConfig};
use crate::protos::module::{CalibrationError, CalibrationStep, CalibrationStepStatus, SOILModuleConfig, SOILCalibrationStart, SOILCalibrationStep, SOILCalibrationStepEvent};

impl crate::modulestate::interface::ModuleValue for SOILCalibrationStepEvent {}
impl crate::modulestate::interface::ModuleValueParsable for SOILCalibrationStepEvent {}

#[derive(Debug, Clone)]
pub struct CalibrationEx {

}

impl CalibrationEx {
    fn new() -> CalibrationEx {
        return CalibrationEx{};
    }
}


pub fn get_value_property_with_calibration(
    value: i32,
    config: &SOILProbeConfig,
) -> i32 {
    if value < 10 {
        return -1;
    }
    let m: f32 = (100.0f32) / ((config.high - config.low) as f32);
    let b: f32 = -1.0f32 * (m * config.low as f32);
    let mut v = ((m * (value as f32)) + b) as i32;
    if v > 100 {
        v = 100;
    } else if v < 0 {
        v = 0;
    }
    return v;
}

pub fn transform_value_with_calibration(
    data: &mut SOILModuleData,
    config: &SOILModuleConfig,
) -> () {
    data.p0 = get_value_property_with_calibration(data.p0, config.get_p0());
    data.p1 = get_value_property_with_calibration(data.p1, config.get_p1());
    data.p2 = get_value_property_with_calibration(data.p2, config.get_p2());
    data.p3 = get_value_property_with_calibration(data.p3, config.get_p3());
    data.p4 = get_value_property_with_calibration(data.p4, config.get_p4());
    data.p5 = get_value_property_with_calibration(data.p5, config.get_p5());
    data.p6 = get_value_property_with_calibration(data.p6, config.get_p6());
    data.p7 = get_value_property_with_calibration(data.p7, config.get_p7());
}

pub struct CalibrationProcess {
    // gonna need to send for mqtt message

    // gonna need to store my steps
    pub previous_step: CalibrationStep,
    pub current_step: CalibrationStep,

    pub state: CalibrationStepStatus,

    pub ex: CalibrationError,

    // gonna need to store the value for each step
    pub values_low: Vec<SOILModuleData>,
    pub values_high: Vec<SOILModuleData>,
}


// While the calibration processs is processing the module
// will stop producing the data to the rest of the application
impl CalibrationProcess {

    pub fn new() -> Self {
        return CalibrationProcess{
            previous_step: CalibrationStep::READY_CALIBRATION,
            current_step: CalibrationStep::READY_CALIBRATION,
            state: CalibrationStepStatus::AWAITING_STEP_STATUS,
            ex: CalibrationError::NONE_ERROR,
            values_low: vec![],
            values_high: vec![],
        };
    }

    // handle the value accordinling to the step
    pub fn on_value(&mut self, data: SOILModuleData) -> () {
        if self.current_step == CalibrationStep::LOW_CALIBRATION {
            self.values_low.push(data);
        } else if self.current_step == CalibrationStep::HIGH_CALIBRATION {
            self.values_high.push(data);
        }
    }

    // start the recording of the current step
    pub fn start_record(&mut self, step: CalibrationStep) -> () {
        if step == CalibrationStep::LOW_CALIBRATION {
            self.values_low.clear();
        } else if step == CalibrationStep::HIGH_CALIBRATION {
            self.values_high.clear();
        }
        self.previous_step = self.current_step;
        self.current_step = step;
    }

    pub fn get_status(&self) -> SOILCalibrationStepEvent {
        let mut event = SOILCalibrationStepEvent::new();
        event.set_status(self.state);
        event.set_step(self.current_step);
        event.set_erro(self.ex);
        self.values_low.iter().for_each(|v| event.low.push(v.clone()));
        self.values_high.iter().for_each(|v| event.high.push(v.clone()));
        return event;
    }

    // stop the current step recording
    pub fn stop_record(&mut self) -> () {
        self.previous_step = self.current_step;
        if (self.values_high.len() > 0 && self.values_low.len() > 0) {
            self.current_step = CalibrationStep::WAITING_CONFIRMATION_CALIBRATION;
        } else {
            self.current_step = CalibrationStep::READY_CALIBRATION;
        }
    }

    // return the config
    pub fn terminate(&self) -> Result<SOILModuleConfig, CalibrationEx>  {
        if self.current_step == CalibrationStep::WAITING_CONFIRMATION_CALIBRATION {
            let mut config = SOILModuleConfig::new();
            // fait la moyenne des arrays
            let mut total_item = 0;

            let value_low = self.average_values(&self.values_low);
            let value_high = self.average_values(&self.values_high);

            self.handle_property_config(config.mut_p0(), value_low.get_p0(), value_high.get_p0());
            self.handle_property_config(config.mut_p1(), value_low.get_p1(), value_high.get_p1());
            self.handle_property_config(config.mut_p2(), value_low.get_p2(), value_high.get_p2());
            self.handle_property_config(config.mut_p3(), value_low.get_p3(), value_high.get_p3());
            self.handle_property_config(config.mut_p4(), value_low.get_p4(), value_high.get_p4());
            self.handle_property_config(config.mut_p5(), value_low.get_p5(), value_high.get_p5());
            self.handle_property_config(config.mut_p6(), value_low.get_p6(), value_high.get_p6());
            self.handle_property_config(config.mut_p7(), value_low.get_p7(), value_high.get_p7());

            return Ok(config);
        }
        return Err(CalibrationEx::new())
    }

    fn handle_property_config(&self, property: &mut SOILProbeConfig, value_low: i32, value_high: i32) -> () {
        property.set_low(value_low);
        property.set_high(value_high);
    }
    
    fn average_values(&self, values: &Vec<SOILModuleData>) -> SOILModuleData {
        let mut data = SOILModuleData::new();
        let len = values.len() as i32;
        values.iter().for_each(|f| {
            data.p0 += f.p0;
            data.p1 += f.p1;
            data.p2 += f.p2;
            data.p3 += f.p3;
            data.p4 += f.p4;
            data.p5 += f.p5;
            data.p6 += f.p6;
            data.p7 += f.p7;
        });
        data.p0 = data.p0 / len;
        data.p1 = data.p1 / len;
        data.p2 = data.p2 / len;
        data.p3 = data.p3 / len;
        data.p4 = data.p4 / len;
        data.p5 = data.p5 / len;
        data.p6 = data.p6 / len;
        data.p7 = data.p7 / len;

        return data;
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::protos::module::SOILModuleData;

    fn from_tuple(values: &[i32; 8]) -> SOILModuleData {
        let mut data = SOILModuleData::new();
        data.p0 = values[0];
        data.p1 = values[1];
        data.p2 = values[2];
        data.p3 = values[3];
        data.p4 = values[4];
        data.p5 = values[5];
        data.p6 = values[6];
        data.p7 = values[7];
        return data;
    }

    #[test]
    fn calibration_process_succesful() {

        let values_low: Vec<SOILModuleData> = vec![
            [24,24,24,24,0,0,0,0]
        ].iter().map(from_tuple).collect();

        let values_high: Vec<SOILModuleData> = vec![
            [300,300,300,300,0,0,0,0]
        ].iter().map(from_tuple).collect();

        let mut process = CalibrationProcess::new();

        assert_eq!(process.current_step, CalibrationStep::READY_CALIBRATION);

        process.start_record(CalibrationStep::LOW_CALIBRATION);

            values_low.iter().for_each(|x| {
                process.on_value(x.clone());
            });

        process.stop_record();

        assert_eq!(process.current_step, CalibrationStep::READY_CALIBRATION);

        process.start_record(CalibrationStep::HIGH_CALIBRATION);

            values_high.iter().for_each(|x| {
                process.on_value(x.clone());
            });

        process.stop_record();

        assert_eq!(process.current_step, CalibrationStep::WAITING_CONFIRMATION_CALIBRATION);

        let config_result = process.terminate().unwrap();


    }
}