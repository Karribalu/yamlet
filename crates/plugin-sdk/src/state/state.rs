use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /**
     *  The version of the state file protocol version.
     */
    pub version: String,
    /**
     * Yamlet version used to generate this state file.
     */
    pub yamlet_version: String,
    /**
     * Serial is incremented on any operation that modifies
     * the State file. It is used to detect potentially conflicting
     * updates.
     */
    pub serial: String,

    /**
     * Lineage is set when a new, blank state is created and then
     * never updated. This allows us to determine whether the serials
     * of two states can be meaningfully compared.
     * Apart from the guarantee that collisions between two lineages
     * are very unlikely, this typing is opaque and external callers
     * should only compare lineage strings byte-for-byte for equality.
     */
    pub lineage: String,
    /**
        Outputs track the values of outputs from the pack
    */
    pub outputs: BTreeMap<String, Output>,
    /**
    A Breadth-first traversal of the resource tree.
    */
    pub resources: Vec<Resource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Output {
    pub value: serde_json::Value,
    #[serde(rename = "type")]
    pub type_def: serde_json::Value,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceMode {
    #[serde(rename = "managed")]
    Managed,
    #[serde(rename = "imported")]
    Imported,
    #[serde(rename = "byo")]
    Byo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    pub mode: ResourceMode,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub name: String,
    pub provider: String,
    pub instances: Vec<Instance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    pub schema_version: String, // The schema version of the instance.
    pub attributes: BTreeMap<String, serde_json::Value>,
    pub sensitive_attributes: BTreeMap<String, serde_json::Value>,
}
