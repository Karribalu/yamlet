use std::collections::HashMap;

use crate::schema::schema::Schema;

pub struct ResourceIdentity {
    version: Option<u64>,

    schema_fn: HashMap<String, Schema>,
    // TODO: Add State Upgrader for the resource
}

impl ResourceIdentity {
    pub fn new(version: Option<u64>, schema_fn: HashMap<String, Schema>) -> Self {
        ResourceIdentity { version, schema_fn }
    }
    pub fn version(&self) -> &Option<u64> {
        &self.version
    }
    pub fn schema_fn(&self) -> &HashMap<String, Schema> {
        &self.schema_fn
    }

    pub fn set_version(&mut self, version: u64) {
        self.version = Some(version);
    }

    pub fn set_schema_fn(&mut self, schema_fn: HashMap<String, Schema>) {
        self.schema_fn = schema_fn;
    }
}
