use std::collections::HashMap;

use std::sync::{Arc, Mutex};

use protobuf::Message;
use tokio::sync::watch::{channel, Receiver, Sender};
use tokio_util::sync::CancellationToken;

use crate::mainboardstate::error::MainboardError;
use crate::modulestate::alarm::model::ModuleAlarmState;
use crate::modulestate::alarm::store::ModuleAlarmStore;
use crate::modulestate::state_manager::MainboardModuleStateManager;
use crate::protos::env_controller::{
    EnvironmentControllerConfiguration_oneof_implementation, RessourceType,
};
use crate::socket::ss::SenderSocket;
use crate::store::database::get_many_field_from_table;
use crate::{
    modulestate::alarm::model::ModuleValueChange,
    protos::{alarm::FieldAlarmEvent, env_controller::EnvironmentControllerConfiguration},
};

use super::controller_trait::EnvControllerTask;
use super::context::Context;
use super::imple::static_controller::StaticControllerImplementation;
use super::module_command::ModuleCommandSender;

impl crate::modulestate::interface::ModuleValue for EnvironmentControllerConfiguration {}
impl crate::modulestate::interface::ModuleValueParsable for EnvironmentControllerConfiguration {}

struct StoreEnvControllerTask {
    cancellation_token: CancellationToken,
    handler: tokio::task::JoinHandle<Result<(), MainboardError>>,
    config: EnvironmentControllerConfiguration,
}

pub struct EnvControllerStore {

    tasks: HashMap<String, StoreEnvControllerTask>,

    conn: Arc<Mutex<rusqlite::Connection>>,

    sender: SenderSocket,

    pub alarm_senders: HashMap<String, (Sender<FieldAlarmEvent>, Receiver<FieldAlarmEvent>)>,
    pub value_senders: HashMap<
        String,
        (
            Sender<ModuleValueChange<f32>>,
            Receiver<ModuleValueChange<f32>>,
        ),
    >,
}

impl EnvControllerStore {
    pub fn new(
        conn: Arc<Mutex<rusqlite::Connection>>,
        socket: SenderSocket,
    ) -> Self {
        return Self {
            conn,
            sender: socket,
            tasks: HashMap::new(),
            alarm_senders: HashMap::new(),
            value_senders: HashMap::new(),
        };
    }

    // TO INITIALIZE

    pub fn on_alarm_created(
        &mut self,
        module_id: &str,
        property: &str,
        module_state_manager: &MainboardModuleStateManager,
        module_alarm_store: &ModuleAlarmStore,
        state: Option<ModuleAlarmState<f32>>,
        no_validate: bool,
    ) -> Result<(), MainboardError> {
        let key = format!("{}:{}", module_id, property);
        if self.alarm_senders.contains_key(&key) {
            return Err(MainboardError::from_error(format!(
                "on_alarm_created already exists : {}",
                key
            )));
        }

        let mut event = FieldAlarmEvent {
                moduleId: module_id.into(),
                property: property.into(),
                ..Default::default()
        };
        if let Some(state) = state {
            event.currentZone = state.zone;
            event.currentValue = state.current_value;
            event.previousValue = state.previous_value;
        }

        self.alarm_senders.insert(
            key,
            channel(event),
        );

        log::debug!("creating field alarm event channel for {}:{}", module_id, property);

        if no_validate == false {
            return self.validate_creation(module_state_manager, module_alarm_store);
        }
        return Ok(());
    }

    pub fn on_alarm_deleted(
        &mut self,
        module_id: &str,
        property: &str,
        module_state_manager: &MainboardModuleStateManager,
        module_alarm_store: &ModuleAlarmStore,
    ) -> Result<(), MainboardError> {
        let key = format!("{}:{}", module_id, property);
        self.alarm_senders.remove(&key);
        log::debug!("deleting field alarm event channel for {}:{}", module_id, property);
        return self.validate_removale(module_state_manager, module_alarm_store);
    }

    pub fn on_module_connected(
        &mut self,
        module_id: &str,
        module_state_manager: &MainboardModuleStateManager,
        module_alarm_store: &ModuleAlarmStore,
    ) -> Result<(), MainboardError> {
        if self.value_senders.contains_key(module_id) {
            return Err(MainboardError::from_error(format!(
                "on_module_connected already exists : {}",
                module_id
            )));
        }

        self.value_senders.insert(
            module_id.into(),
            channel(ModuleValueChange::<f32> {
                module_id: module_id.into(),
                changes: vec![],
            }),
        );

        log::debug!("creating module value event channel for {}", module_id);

        return self.validate_creation(module_state_manager, module_alarm_store);
    }

    pub fn on_module_disconnected(
        &mut self,
        module_id: &str,
        module_state_manager: &MainboardModuleStateManager,
        module_alarm_store: &ModuleAlarmStore,
    ) -> Result<(), MainboardError> {
        self.value_senders.remove(module_id);
        log::debug!("deleting module value event channel for {}", module_id);
        // clear alarm link to the module
        let mut to_delete = vec![];
        for (k,_) in self.alarm_senders.iter() {
            if k.contains(module_id) {
                to_delete.push(k.clone());
            }
        }
        for k in to_delete { 
            self.alarm_senders.remove(&k);
            log::debug!("removing field alarm event channel for {}", k);
        }
        
        return self.validate_removale(module_state_manager, module_alarm_store);
    }

    pub fn register_controller(
        &mut self,
        module_state_manager: &MainboardModuleStateManager,
        module_alarm_store: &ModuleAlarmStore,
        config: EnvironmentControllerConfiguration,
    ) -> Result<bool, MainboardError> {
        if self.existing(&config) {
            return Err(MainboardError::from_error(format!("already existing")));
        }

        self.can_be_created(&config)?;

        // save to database
        crate::store::database::store_field_from_table(
            &self.conn,
            "environment_controller",
            &config.id,
            "config",
            Box::new(config.clone()),
        )?;


        log::debug!("environment_controller {} has been register to the database", config.id);

        if self.can_be_initialize(&module_state_manager, &module_alarm_store, &config) {
            return self.create_task(&config).map(|_| true);
        }

        log::debug!("environment_controller {} cannot be started for now, missing ressources", config.id);

        return Ok(false);
    }

    pub fn unregister_controller(&mut self, id: &str) -> Result<(), MainboardError> {
        if let Some(value) = self.tasks.remove(id) {
            value.cancellation_token.cancel();
            log::debug!("environment_controller {} task has been cancelled", id);
        }

        let res = crate::store::database::store_delete_key(&self.conn, "environment_controller", id);
        if res.is_ok() {
            log::debug!("environment_controller {} has been unregister to the database", id);
        }
        return res;
    }

    fn validate_creation(
        &mut self,
        module_state_manager: &MainboardModuleStateManager,
        module_alarm_store: &ModuleAlarmStore,
    ) -> Result<(), MainboardError> {
        // Get the list of environment_controller
        for config in get_many_field_from_table(
            &self.conn,
            "environment_controller",
            EnvironmentControllerConfiguration::parse_from_bytes,
        )? {
            if !self.tasks.contains_key(&config.id) {
                if self.can_be_initialize(module_state_manager, module_alarm_store, &config) {
                    if let Err(err) = self.create_task(&config).map(|_| true) {
                        return Err(err);
                    }
                }
            }
        }

        return Ok(());
    }

    fn validate_removale(
        &mut self,
        module_state_manager: &MainboardModuleStateManager,
        module_alarm_store: &ModuleAlarmStore,
    ) -> Result<(), MainboardError> {
        let mut removed = vec![];
        for (k, v) in self.tasks.iter() {
            if !self.can_be_initialize(module_state_manager, module_alarm_store, &v.config) {
                removed.push(k.clone());
            }
        }
        for key in removed {
            if let Some(value) = self.tasks.remove(&key) {
                value.cancellation_token.cancel();
            }
        }

        return Ok(());
    }

    fn create_task(
        &mut self,
        config: &EnvironmentControllerConfiguration,
    ) -> Result<(), MainboardError> {
        let ctx = self.create_context(config)?;
        let entry = StoreEnvControllerTask {
            config: config.clone(),
            cancellation_token: ctx.cancellation_token.clone(),
            handler: self.start_task_implementation(&config, ctx)?,
        };

        log::debug!("environment_controller task has been started {} running {}", config.get_id(), !entry.handler.is_finished());

        tokio::task::spawn(async {});
        self.tasks.insert(config.id.clone(), entry);

        return Ok(());
    }

    fn start_task_implementation(
        &self,
        config: &EnvironmentControllerConfiguration,
        ctx: Context,
    ) -> Result<tokio::task::JoinHandle<Result<(), MainboardError>>, MainboardError> {
        if let Some(implementation) = &config.implementation {
            match implementation {
                EnvironmentControllerConfiguration_oneof_implementation::field_static(_) => {
                    let handler = StaticControllerImplementation::new().run(ctx);
                    return handler;
                }
                _ => {
                    return Err(MainboardError::from_error(format!(
                        "implementation is missing"
                    )));
                }
            }
        } else {
            return Err(MainboardError::from_error(format!(
                "implementation is missing"
            )));
        }
    }

    fn create_context(&self, config: &EnvironmentControllerConfiguration) -> Result<Context, MainboardError> {
        let mut alarm_receivers = HashMap::new();
        let mut value_receivers = HashMap::new();

        for obs in config.observers.iter() {
            let key = format!("{}:{}", obs.get_id(), obs.get_property());
            if let Some(sr) = self.alarm_senders.get(&key) {
                alarm_receivers.insert(key.clone(), sr.1.clone());
            } else {
               return Err(MainboardError::from_error(format!("failed to get field alarm event receive")));
            }

            if let Some(sr) = self.value_senders.get(obs.get_id()) {
                value_receivers.insert(key, sr.1.clone());
            } else {
               return Err(MainboardError::from_error(format!("failed to get field value event receive")));
            }
        }


        Ok(Context {
            config: config.clone(),
            cancellation_token: CancellationToken::new(),
            module_command_sender: ModuleCommandSender::new(),
            alarm_receivers,
            value_receivers,
            sender_socket: self.sender.clone(),
        })
    }

    // check in the database if we already exists
    fn existing(&self, config: &EnvironmentControllerConfiguration) -> bool {
        let result = crate::store::database::get_field_from_table(
            &self.conn,
            "environment_controller",
            &config.id,
            EnvironmentControllerConfiguration::parse_from_bytes,
        );
        return if result.is_ok() { true } else { false };
    }

    // validate that all the wanted module are not
    // already taken , wont implement for now
    fn can_be_created(
        &self,
        config: &EnvironmentControllerConfiguration,
    ) -> Result<(), MainboardError> {
        if config.implementation.is_none() {
            return Err(MainboardError::from_error("implementaton missing".into()));
        }
        return Ok(());
    }

    // validated that the controller can be created
    // that all required module are present
    fn can_be_initialize(
        &self,
        module_state_manager: &MainboardModuleStateManager,
        module_alarm_store: &ModuleAlarmStore,
        config: &EnvironmentControllerConfiguration,
    ) -> bool {
        let list_modules = module_state_manager.get_connected_modules();
        for obs in config.observers.iter() {
            if !list_modules.contains(&obs.id) {
                return false;
            }
            // check if we also have alarm
            if let Err(err) =
                module_alarm_store.get_alarm_for_module_property(&obs.id, &obs.property)
            {
                log::warn!("failed to get alarm for module property : {:?}", err);
                return false;
            }
        }
        for obs in config.actors.iter() {
            if obs.get_field_type() == RessourceType::ACTOR_MODULE
                && !list_modules.contains(&obs.id)
            {
                return false;
            }
            // check if we are not reserve by anything else
        }

        return true;
    }
}

#[cfg(test)]
mod tests {

    use std::{time::Duration, sync::mpsc::channel};

    use protobuf::RepeatedField;

    use crate::{
        modulestate::{modules::get_module_validator, state_manager::MainboardConnectedModule},
        protos::{
            alarm::FieldAlarm,
            env_controller::{MObserver, SCConditionActor, StaticControllerImplementation},
        }, socket::ss::SenderPayload,
    };

    use super::*;

    fn init(uid: &str) -> (
        MainboardModuleStateManager,
        EnvControllerStore,
        ModuleAlarmStore,
    ) {
        let conn_database = Arc::new(Mutex::new(crate::store::database::init(Some(
            format!("./database_test_env_controller_{}.sqlite", uid),
        ))));

        let (ss, rs) = channel::<SenderPayload>();

        let msm = MainboardModuleStateManager::new();
        let ecs = EnvControllerStore::new(conn_database.clone(), ss.clone().into());
        let mas = ModuleAlarmStore::new(conn_database);

        clear_store(&ecs);

        return (msm, ecs, mas);
    }

    fn clear_store(store: &EnvControllerStore) {
        store
            .conn
            .lock()
            .unwrap()
            .execute_batch(
                "BEGIN; DELETE FROM environment_controller; DELETE FROM module_field_alarm; COMMIT;",
            )
            .unwrap();
    }

    fn add_fake_module(
        msm: &mut MainboardModuleStateManager,
        mas: &mut ModuleAlarmStore,
        ecs: &mut EnvControllerStore,
        id: &str
    ) {
        let cm = MainboardConnectedModule {
            port: 0,
            id: id.to_string(),
            board: "i2c".to_string(),
            board_addr: "0".to_string(),
            handler_map: HashMap::new(),
            last_value: None,
            validator: get_module_validator(&id[0..3]).unwrap(),
        };
        msm.connected_module.insert(id.to_string(), cm);
        ecs.on_module_connected(id, msm, mas).unwrap();
    }

    fn disconect_module(
        msm: &mut MainboardModuleStateManager,
        mas: &mut ModuleAlarmStore,
        ecs: &mut EnvControllerStore,
        id: &str
    ) {
        msm.connected_module.remove(id);
        ecs.on_module_disconnected(id, msm, mas).unwrap();
    }

    fn add_observer(
        config: &mut EnvironmentControllerConfiguration,
        name: &str,
        id: &str,
        property: &str,
    ) {
        config.mut_observers().push(MObserver {
            name: name.to_string(),
            id: id.to_string(),
            property: property.to_string(),
            field_type: crate::protos::env_controller::RessourceType::ACTOR_MODULE,
            ..Default::default()
        });
    }

    fn add_alarm(
        store: &ModuleAlarmStore,
        msm: &mut MainboardModuleStateManager,
        ecs: &mut EnvControllerStore,
        id: &str, property: &str
    ) {
        let mut field_alarm = FieldAlarm::new();
        field_alarm.moduleId = id.into();
        field_alarm.property = property.into();
        field_alarm.mut_low().value = 10.;
        field_alarm.mut_low().offset = 1.;
        field_alarm.mut_high().value = 20.;
        field_alarm.mut_high().offset = 1.;

        store.add_alarm_field(&field_alarm).unwrap();
        ecs.on_alarm_created(id, property,  msm, store, None, false).unwrap();
    }

    fn remove_alarm(
        msm: &mut MainboardModuleStateManager,
        mas: &mut ModuleAlarmStore,
        ecs: &mut EnvControllerStore,
        id: &str,
        property: &str,
    ) {
        let mut field_alarm = FieldAlarm::new();
        field_alarm.moduleId = id.into();
        field_alarm.property = property.into();
 
        mas.remove_alarm_field(&field_alarm).unwrap();
        ecs.on_alarm_deleted(id, property, msm, mas).unwrap();
    }

    fn add_static(config: &mut EnvironmentControllerConfiguration, condition: SCConditionActor) {
        let mut conditions = RepeatedField::new();
        conditions.push(condition);
        config.set_field_static(StaticControllerImplementation {
            conditions,
            ..Default::default()
        });
    }

    #[test]
    fn env_controller_module_not_connected() {
        let (msm, mut ecs, mas) = init("not_connected");
        let mut config = EnvironmentControllerConfiguration::new();
        config.set_id("test".to_string());
        add_observer(&mut config, "obs", "AAA0000003", "airTemperature");
        add_static(&mut config, SCConditionActor::default());

        let is_starting = ecs.register_controller(&msm, &mas, config.clone()).unwrap();

        assert_eq!(is_starting, false);
        assert_eq!(ecs.existing(&config), true);
        assert_eq!(ecs.tasks.get(&config.id).is_some(), false);
    }

    #[tokio::test]
    async fn env_controller_module_connected_no_alarm() {
        let (mut msm, mut ecs,mut mas) = init("no_alarm");

        add_fake_module(&mut msm, &mut mas, &mut ecs, "AAA0000003");

        let mut config = EnvironmentControllerConfiguration::new();
        config.set_id("test".to_string());
        add_observer(&mut config, "obs", "AAA0000003", "airTemperature");
        add_static(&mut config, SCConditionActor::default());

        let is_starting = ecs.register_controller(&msm, &mas, config.clone()).unwrap();

        assert_eq!(is_starting, false);
        assert_eq!(ecs.existing(&config), true);
        assert_eq!(ecs.tasks.get(&config.id).is_some(), false);
    }

    #[tokio::test]
    async fn env_controller_module_connected() {
        let (mut msm, mut ecs,mut mas) = init("connected");

        add_fake_module(&mut msm, &mut mas, &mut ecs, "AAA0000003");

        let mut config = EnvironmentControllerConfiguration::new();
        config.set_id("test".to_string());
        add_alarm(&mas, &mut msm, &mut ecs, "AAA0000003", "airTemperature");
        add_observer(&mut config, "obs", "AAA0000003", "airTemperature");
        add_static(
            &mut config,
            SCConditionActor {
                actor_id: "".into(),
                observer_id: "".into(),
                actions: HashMap::new(),
                ..Default::default()
            },
        );

        let is_starting = ecs.register_controller(&msm, &mas, config.clone()).unwrap();
        let entry = ecs.tasks.get(&config.id).unwrap();

        assert_eq!(is_starting, true);
        assert_eq!(ecs.existing(&config), true);
        assert_eq!(entry.handler.is_finished(), false);

        entry.cancellation_token.cancel();
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(entry.handler.is_finished(), true);
    }

    #[tokio::test]
    async fn env_controller_start_after_module_alarm_added() {
        let (mut msm, mut ecs,mut mas) = init("alarm_added");

        let mut config = EnvironmentControllerConfiguration::new();
        config.set_id("test".to_string());
        add_observer(&mut config, "obs", "AAA0000003", "airTemperature");
        add_static(
            &mut config,
            SCConditionActor {
                actor_id: "".into(),
                observer_id: "".into(),
                actions: HashMap::new(),
                ..Default::default()
            },
        );

        let is_starting = ecs.register_controller(&msm, &mas, config.clone()).unwrap();

        assert_eq!(is_starting, false);

        add_fake_module(&mut msm, &mut mas, &mut ecs, "AAA0000003");

        // start should not be started stil
        let is_starting = ecs.tasks.contains_key(&config.id);

        assert_eq!(is_starting, false);

        add_alarm(&mas, &mut msm, &mut ecs, "AAA0000003", "airTemperature");

        let is_starting = ecs.tasks.contains_key(&config.id);

        assert_eq!(is_starting, true);
         
    }

    #[tokio::test]
    async fn env_controller_stop_after_module_or_alarm_removed() {
        let (mut msm, mut ecs,mut mas) = init("or_alarm_removed");

        add_fake_module(&mut msm, &mut mas, &mut ecs, "AAA0000003");
        add_alarm(&mas, &mut msm, &mut ecs, "AAA0000003", "airTemperature");
        add_alarm(&mas, &mut msm, &mut ecs, "AAA0000003", "humidity");

        let mut config = EnvironmentControllerConfiguration::new();
        config.set_id("test".to_string());
        add_observer(&mut config, "obs", "AAA0000003", "airTemperature");
        add_static(
            &mut config,
            SCConditionActor {
                actor_id: "".into(),
                observer_id: "".into(),
                actions: HashMap::new(),
                ..Default::default()
            },
        );

        let is_starting = ecs.register_controller(&msm, &mas, config.clone()).unwrap();
        assert_eq!(is_starting, true);

        disconect_module(&mut msm, &mut mas, &mut ecs, "AAA0000003");
        
        let is_starting = ecs.tasks.contains_key(&config.id);
        assert_eq!(is_starting, false);

        add_alarm(&mas, &mut msm, &mut ecs, "AAA0000003", "airTemperature");
        add_fake_module(&mut msm, &mut mas, &mut ecs, "AAA0000003");
        let is_starting = ecs.tasks.contains_key(&config.id);
        assert_eq!(is_starting, true);

        remove_alarm(&mut msm, &mut mas, &mut ecs, "AAA0000003", "humidity");
        let is_starting = ecs.tasks.contains_key(&config.id);
        assert_eq!(is_starting, true);

        remove_alarm(&mut msm, &mut mas, &mut ecs, "AAA0000003", "airTemperature");
        let is_starting = ecs.tasks.contains_key(&config.id);
        assert_eq!(is_starting, false);
    }

    #[tokio::test]
    async fn env_controller_unregister_config() {
        let (mut msm, mut ecs,mut mas) = init("unregister_controller");

        add_fake_module(&mut msm, &mut mas, &mut ecs, "AAA0000003");
        add_alarm(&mas, &mut msm, &mut ecs, "AAA0000003", "airTemperature");

        let mut config = EnvironmentControllerConfiguration::new();
        config.set_id("test".to_string());
        add_observer(&mut config, "obs", "AAA0000003", "airTemperature");
        add_static(
            &mut config,
            SCConditionActor {
                actor_id: "".into(),
                observer_id: "".into(),
                actions: HashMap::new(),
                ..Default::default()
            },
        );

        let is_starting = ecs.register_controller(&msm, &mas, config.clone()).unwrap();
        assert_eq!(is_starting, true);

        ecs.unregister_controller(&config.id).unwrap();

        let is_starting = ecs.tasks.contains_key(&config.id);
        assert_eq!(is_starting, false);
    }

}
