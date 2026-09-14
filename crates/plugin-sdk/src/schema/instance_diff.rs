use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// [`InstanceDiff`] is the diff of a resource between one state and another
/// The attributes are flattened with . notation
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstanceDiff {
    attributes: BTreeMap<String, ResourceAttrDiff>,
    destroy: bool,
    identity: HashMap<String, String>, // Used to track the resource identity changes
}

impl InstanceDiff {
    pub fn new() -> Self {
        InstanceDiff {
            attributes: BTreeMap::new(),
            destroy: false,
            identity: HashMap::new(),
        }
    }

    pub fn attributes(&self) -> &BTreeMap<String, ResourceAttrDiff> {
        &self.attributes
    }

    pub fn destroy(&self) -> bool {
        self.destroy
    }

    pub fn identity(&self) -> &HashMap<String, String> {
        &self.identity
    }
}

/// [`ResourceAttrDiff`] is the diff of a single attribute of a resource between one state and another
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceAttrDiff {
    /// OLD value of the attribute
    pub old: String,
    /// NEW value of the attribute
    pub new: String,
    /// True if the attribute is new
    pub new_computed: bool,
    /// True if the attribute is being removed
    pub new_removed: bool,
    /// True if the change requires a new resource
    pub requires_new: bool,
    /// True, if the attribute is sensitive, The UI should hide the value
    pub sensitive: bool,
    /// Type of the attribute, Whether it is provided by the user or computed by the provider
    pub diff_attr_type: DiffType,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum DiffType {
    Provided,
    Computed,
}
