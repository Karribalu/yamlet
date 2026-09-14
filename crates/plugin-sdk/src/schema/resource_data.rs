use crate::schema::instance_diff::InstanceDiff;
use crate::schema::instance_state::InstanceState;
use crate::schema::resource_timeout::ResourceTimeouts;
use crate::schema::schema::Schema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

// TODO: Add methods to manipulate ResourceData

/// [`ResourceData`] is used to query and set the attributes of a resource
///
/// [`ResourceData`] is the primary argument received for CRUD operations
/// on a resource as well as configuration of a provider.
#[derive(Debug, Deserialize, Serialize)]
pub struct ResourceData {
    schema: BTreeMap<String, Schema>,
    instance_state: InstanceState,
    diff: InstanceDiff,
    timeouts: ResourceTimeouts,
    meta: BTreeMap<String, String>,

    // This is used for temporary storage of the new state while applying the diff
    new_state: InstanceState,
    partial: bool,
    is_new: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum ResourceDataError {
    #[error(
        "Invalid operation provided, Supported operations are: `create`, `read`, `update`, `delete`"
    )]
    InvalidOperation,
}

impl ResourceData {
    pub fn new() -> Self {
        ResourceData {
            schema: Default::default(),
            instance_state: InstanceState::default(),
            diff: InstanceDiff::default(),
            timeouts: ResourceTimeouts::default(),
            meta: Default::default(),
            new_state: InstanceState::default(),
            partial: false,
            is_new: false,
        }
    }

    pub fn set_timeouts(&mut self, timeouts: ResourceTimeouts) {
        self.timeouts = timeouts;
    }

    pub fn get_timeout(&self, key: &str) -> Result<Duration, ResourceDataError> {
        match key {
            "create" => Ok(self.timeouts.get_create_timeout()),
            "delete" => Ok(self.timeouts.get_delete_timeout()),
            "read" => Ok(self.timeouts.get_read_timeout()),
            "update" => Ok(self.timeouts.get_update_timeout()),
            _ => Err(ResourceDataError::InvalidOperation),
        }
    }
    pub fn set_instance_state(&mut self, instance_state: InstanceState) {
        self.instance_state = instance_state;
    }
    pub fn instance_state(&self) -> &InstanceState {
        &self.instance_state
    }

    /// `get` returns the data for the given key or None if the key does not exist in the schema
    ///
    /// If the key exists in the schema but doesn't exist in the configuration,
    /// Then the default value for that type will be returned, For strings it will be an empty string,
    /// for numbers it will be 0, for booleans it will be false,
    ///
    pub fn get(&self, key: &str) -> Option<&String> {
        self.instance_state.attributes().get(key)
    }

    pub fn json_type_resource_state(&self) -> serde_json::Value {
        serde_json::json!(self.instance_state)
    }
}
