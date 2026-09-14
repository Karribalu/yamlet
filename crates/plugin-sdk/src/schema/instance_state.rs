use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{schema::instance_diff::InstanceDiff, utils::constants::YAMLET_UNKNOWN_VARIABLE_VALUE};

/// [`InstanceState`] is used to track the unique state information of a resource
/// This contains the dotted notation attributes and their values
#[derive(Serialize, Deserialize, Debug, Default, Clone, Eq, PartialEq)]
pub struct InstanceState {
    /// A unique `id` for the resource. This is opaque to Yamlet.
    /// and is only meant as a lookup mechanism for the providers
    id: String,

    /// `attributes` is used to store the resource attributes
    attributes: HashMap<String, String>,

    /// `identity` is used to store the resource identity information
    identity: HashMap<String, String>,
}

impl InstanceState {
    pub fn new() -> Self {
        InstanceState {
            id: String::new(),
            attributes: HashMap::new(),
            identity: HashMap::new(),
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    pub fn identity(&self) -> &HashMap<String, String> {
        &self.identity
    }

    pub fn set(
        &mut self,
        id: String,
        attributes: HashMap<String, String>,
        identity: HashMap<String, String>,
    ) {
        self.id = id;
        self.attributes = attributes;
        self.identity = identity;
    }

    pub fn clear(&mut self) {
        self.id.clear();
        self.attributes.clear();
        self.identity.clear();
    }

    /// Merges the given [`InstanceDiff`] into the current [`InstanceState`]
    /// and returns a new [`InstanceState`]
    /// If the new value is marked as `new_computed`, it sets the value to a placeholder constant [`YAMLET_UNKNOWN_VARIABLE_VALUE`]
    pub fn merge_diff(&self, diff: &InstanceDiff) -> InstanceState {
        let mut result = self.clone();

        for (k, v) in self.attributes.iter() {
            result.attributes.insert(k.clone(), v.clone());
        }

        for (k, v) in diff.attributes().iter() {
            if v.new_removed {
                result.attributes.remove(k);
                continue;
            }
            if v.new_computed {
                result
                    .attributes
                    .insert(k.clone(), YAMLET_UNKNOWN_VARIABLE_VALUE.to_string());
                continue;
            }
            result.attributes.insert(k.clone(), v.new.clone());
        }

        result
    }
}
