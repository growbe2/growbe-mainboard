use super::controller::store::EnvControllerStore;
use super::state_manager::MainboardModuleStateManager;
use crate::comboard::imple::interface::ModuleValueValidationEvent;
use crate::mainboardstate::error::MainboardError;
use std::sync::mpsc::Sender;

pub fn handle_module_value<'a>(
    manager: &mut MainboardModuleStateManager,
    value: &ModuleValueValidationEvent,
    sender_socket: &Sender<(String, Box<dyn super::interface::ModuleValueParsable>)>,
    alarm_validator: &mut super::alarm::validator::AlarmFieldValidator,
    alarm_store: &super::alarm::store::ModuleAlarmStore,
    env_controller: &mut EnvControllerStore,
) -> Result<(), MainboardError> {
    let reference_connected_module_option =
        manager.get_module_at_index_mut(&value.board, &value.board_addr, value.port);
    if reference_connected_module_option.is_none() {
        log::error!(
            "receive value for port {} but module is not in the store",
            value.port
        );
        return Err(MainboardError::not_found("module_port", ""));
    }
    let reference_connected_module = reference_connected_module_option.unwrap();

    let on_change = |value| -> () {
        if let Err(err) = sender_socket.send((
            String::from(format!("/m/{}/data", reference_connected_module.id)),
            value,
        )) {
            log::error!("failed to send module data : {:?}", err);
        }
    };

    match reference_connected_module.validator.convert_to_value(value) {
        Ok(sensor_value) => {
            if let Some(last_value) = reference_connected_module.last_value.as_ref() {
                let change = reference_connected_module
                    .validator
                    .have_data_change(&sensor_value, last_value);
                if change.0 == true {
                    on_change(sensor_value);
                    if let Ok(previous_value) =
                        reference_connected_module.validator.convert_to_value(value)
                    {
                        reference_connected_module.last_value = Some(previous_value);

                        log::debug!(
                            "data have changed for {} len {}",
                            reference_connected_module.id.as_str(),
                            change.1.len()
                        );

                        if change.1.len() > 0 {
                            let module_value_change = super::alarm::model::ModuleValueChange::<f32> {
                                module_id: reference_connected_module.id.clone(),
                                changes: change.1,
                            };
                            alarm_validator
                                .on_module_value_change(&module_value_change)
                                .iter()
                                .for_each(|(event, state)| {
                                    if let Err(err) = sender_socket.send((
                                        format!("/m/{}/alarm", event.moduleId),
                                        Box::new(event.clone_me()),
                                    )) {
                                        log::error!("failed to send alarm state : {:?}", err);
                                    }
                                    if let Err(err) = alarm_store.update_alarm_state(
                                        event.moduleId.as_str(),
                                        event.property.as_str(),
                                        &state,
                                    ) {
                                        log::error!("failed to update alarm state : {:?}", err);
                                    }
                                });
                        }
                    }
                }
            } else {
                on_change(sensor_value);

                if let Ok(previous_value) =
                    reference_connected_module.validator.convert_to_value(value)
                {
                    reference_connected_module.last_value = Some(previous_value);
                }
            }
        }
        Err(e) => log::error!("convert to value error : {}", e),
    }

    return Ok(());
}
