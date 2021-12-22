use std::collections::HashMap;

use crate::protos::module::Actor;

/*
pub struct ActorStore {
    pub actors_property: HashMap<String, Actor>,
}

*/
pub fn get_owner<'a>(
    map: &'a HashMap<String, Actor>,
    property: &'static str,
) -> Option<&'a Actor> {
    return map.get(property);
}
/*
pub fn validate_new_owner(
    _map: &mut HashMap<String, Actor>,
    _property: &str, previous_owner: Option<&Actor>,
    new_owner: Option<Actor>
) -> bool {


    return true;
}

impl ActorStore {
    pub fn new() -> Self {
        return ActorStore { actors_property: HashMap::new() }
    }

    pub fn get_owner(&mut self, property: &str) -> Option<&Actor> {
        return self.actors_property.get(property);
    }

    // return if the owner have change or not and store the owner in the map
    pub fn validate_new_owner(&mut self, property: &str, previous_owner: Option<&Actor>, new_owner: Option<Actor>) -> bool {

        

        return true;
    }
}
*/