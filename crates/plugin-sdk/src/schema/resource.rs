use crate::schema::{
    resource_data::ResourceData, resource_identity::ResourceIdentity,
    resource_timeout::ResourceTimeouts, schema::Schema,
};
use std::collections::{BTreeMap, HashMap};

type SchemaFn = Box<dyn Fn() -> HashMap<String, Schema>>;

type CreateFn = Box<dyn Fn(&mut ResourceData, &Option<serde_json::Value>) -> Result<(), String>>;

type ReadFn = Box<dyn Fn(&mut ResourceData, &Option<serde_json::Value>) -> Result<(), String>>;

type UpdateFn = Box<dyn Fn(&mut ResourceData, &Option<serde_json::Value>) -> Result<(), String>>;

type DeleteFn = Box<dyn Fn(&mut ResourceData, &Option<serde_json::Value>) -> Result<(), String>>;

/// [`Resource`] is the most basic unit of a yamlet model.
///   - Managed `Resource`: An infrastructure component with a schema, lifecycle
///     operations such as create, read, update, and delete
///     (CRUD), and optional implementation details such as
///     import support, upgrade state support, and difference
///   - BYO (Bring Your Own) `Resource`: Also known as a data source. An infrastructure component
///     with a schema and only the read lifecycle operation.
pub struct Resource {
    /// [`schema`] defines the attributes and their properties for this resource.
    /// This field or schema_fn must be provided for all resource concepts.
    schema: BTreeMap<String, Schema>,

    /// [`schema_fn`] is an optional function that returns the schema for this resource.
    /// This field or schema must be provided for all resource concepts.Use this field insted of schema when you don't
    /// want to store the information in-memory through our the lifecycle of the resource.
    schema_fn: Option<SchemaFn>,

    /// `schema_version` is the version of the resource's schema definition.
    /// This field is None when the resource is not managed.
    schema_version: Option<u64>,

    /// `identity` defines the identity information for this resource.
    /// This includes the versioned schema definitions and state upgrader functions.
    /// This applies only to managed resources.
    /// This field is optional
    identity: Option<ResourceIdentity>,

    /// `create` is the function that implements the create lifecycle operation for this resource.
    /// This field is required for managed resources and must be None for BYO resources.
    /// The [`ResourceData`] argument provides access to the resource's configuration and state.
    /// The `Option<serde_json::Value>` argument provides access to any additional parameters.
    create: Option<CreateFn>,

    /// `read` is the function that implements the read lifecycle operation for this resource.
    /// This field is required for all resource concepts.
    /// The [`ResourceData`] argument provides access to the resource's configuration and state.
    /// The `Option<serde_json::Value>` argument provides access to any additional parameters.
    /// The provider can signal yamlet that the managed resource is no longer present by returning an empty id without returning an error.
    /// BYO resources that are designed to return a state for a component should conventionally return an error if the component is not found.
    read: ReadFn,

    /// `update` is the function that implements the update lifecycle operation for this resource.
    /// The [`ResourceData`] argument provides access to the resource's configuration and state.
    /// The `Option<serde_json::Value>` argument provides access to any additional parameters.
    /// The provider can signal yamlet that the managed resource is no longer present by returning an empty id without returning an error.
    /// This field can be optional for managed resources but force_new option must be enabled in the Schema which will destroy and recreate the resource on any changes.
    update: Option<UpdateFn>,

    /// `delete` is the function that implements the delete lifecycle operation for this resource.
    /// This field is required for managed resources and must be None for BYO resources.
    /// The [`ResourceData`] argument provides access to the resource's configuration and state.
    /// The `Option<serde_json::Value>` argument provides access to any additional parameters.
    delete: Option<DeleteFn>,

    /// [`timeouts`] defines the timeouts for the various lifecycle operations of this resource.
    /// This field is optional, The default timeouts will be used if not provided. i.e. 5 minutes for read, 30 minutes for create and update, and 60 minutes for delete.
    timeouts: Option<ResourceTimeouts>,

    /// `description` is a human-readable description of the resource.
    /// This field is optional and can be used to provide additional context about the resource.
    description: Option<String>,
}
