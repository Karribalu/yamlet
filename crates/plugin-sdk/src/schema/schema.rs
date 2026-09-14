use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

type SchemaResult = Result<(), SchemaValidationError>;
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum SchemaValidationError {
    #[error("Type mismatch error: {0}")]
    TypeMismatch(String),
}

// Do we need Set in this schema?
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValueType {
    #[serde(rename = "string")]
    TypeString,
    #[serde(rename = "int")]
    TypeInt,
    #[serde(rename = "float")]
    TypeFloat,
    #[serde(rename = "bool")]
    TypeBool,
    #[serde(rename = "list")]
    TypeList,
    #[serde(rename = "object")]
    TypeObject,
}
type SchemaDefaultFn = Box<dyn Fn() -> Option<SchemaElem>>;
type SchemaValidateFn = Box<dyn Fn(&SchemaElem) -> bool>;
type SchemaStateFn = Box<dyn Fn() -> String>;
#[derive(Serialize, Deserialize)]
pub struct Schema {
    /// The typing type must be one of the following:
    /// - `TypeString` - string
    /// - `TypeInt` - i64
    /// - `TypeFloat` - f64
    /// - `TypeBool` - bool
    /// - `TypeList` - vec
    /// - `TypeObject` - map
    /// If
    value_type: ValueType,

    /// The value of the resource schema
    elem: SchemaElem,

    /// `schema_version` is the version of the resource's schema definition.
    /// This field is None when the resource is not manager
    schema_version: Option<u64>,

    /// Minimum number of items in the typing type of array
    min_items: Option<u64>,

    /// Maximum number of items in the typing type of array
    max_items: Option<u64>,

    /// [`default`] indicates a value to set if this attribute is not set in the configuration
    /// `default` cannot be used with [`default_fn`] or [`required`].
    /// default is only supported if the value_type is String, Int, Float, Bool
    default: Option<SchemaElem>,

    /// Validation function to check if the typing is valid
    #[serde(skip)]
    validate_fn: Option<SchemaValidateFn>,

    /// Default typing for the field when not provided
    ///
    /// TODO: Do we need error support here?
    #[serde(skip)]
    default_fn: Option<SchemaDefaultFn>,

    /// A human-readable description of the attribute, Which will be used for documentation
    description: Option<String>,

    /// [`state_fn`] is a function called to change the value of this before
    /// storing it in the state (and likewise before comparing for diffs).
    /// The use for this is, for example, with large strings, you may want
    /// to simply store the hash of it.
    #[serde(skip)]
    state_fn: Option<SchemaStateFn>,

    /// [`conflicts_with`] is a list of attributes that cannot be set at the same time.
    /// This implements validation logic declaratively withing the schema and can trigger earlier in Yamlet operations
    ///
    /// Only absolute attribute paths, Ones starting with the top level attribute names, are supported,
    /// For TypeList (if [`max_items`] is greater than 1), TypeMap, or TypeSet
    /// attributes.
    /// To reference an attribute under a single configuration block
    /// (`TypeList` MaxItems of 1), the syntax is
    /// "parent_block_name.0.child_attribute_name".
    conflicts_with: Option<Vec<String>>,

    /// [`exactly_one_of`] is a list of attributes where exactly one must be set.
    /// It will return an error if none or more than one are set.
    /// This implements validation logic declaratively within the schema and can trigger earlier in Yamlet operations
    ///
    exactly_one_of: Option<Vec<String>>,

    /// [`atleast_one_of`] is a list of attributes where at least one must be set.
    /// It will return an error if none are set.
    /// This implements validation logic declaratively within the schema and can trigger earlier in Yamlet operations
    ///
    atleast_one_of: Option<Vec<String>>,

    /// [`required_with`] is a list of attributes where if this attribute is set,
    /// the listed attributes must also be set.
    /// This implements validation logic declaratively within the schema and can trigger earlier in Yamlet operations
    ///
    required_with: Option<Vec<String>>,

    /// [`sensitive`] marks the attribute as sensitive, which will prevent it from being displayed
    sensitive: bool,

    /// [`optional`] marks the attribute as optional, which means it does not need to be set in configuration
    optional: bool,

    /// [`required`] marks the attribute as required, which means it must be set in configuration
    required: bool,

    /// [`computed`] marks the attribute as computed, which means it is set by the provider
    computed: bool,

    /// [`force_new`] marks the attribute as force_new, which means changes to this attribute will require resource recreation
    force_new: bool,
}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        // Compare all fields except the function pointers
        // Function pointers cannot be compared, so we skip them
        self.value_type == other.value_type
            && self.elem == other.elem
            && self.schema_version == other.schema_version
            && self.min_items == other.min_items
            && self.max_items == other.max_items
            && self.default == other.default
            && self.description == other.description
            && self.conflicts_with == other.conflicts_with
            && self.exactly_one_of == other.exactly_one_of
            && self.atleast_one_of == other.atleast_one_of
            && self.required_with == other.required_with
            && self.sensitive == other.sensitive
            && self.optional == other.optional
            && self.required == other.required
            && self.computed == other.computed
            && self.force_new == other.force_new
        // Note: validate_fn, default_fn, and state_fn are not compared
        // as function pointers cannot be compared for equality
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum SchemaElem {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Schema>),
    Object(BTreeMap<String, Schema>),
    Null,
}

impl SchemaElem {
    pub fn type_name(&self) -> &str {
        match self {
            SchemaElem::String(_) => "string",
            SchemaElem::Int(_) => "int",
            SchemaElem::Float(_) => "float",
            SchemaElem::Bool(_) => "bool",
            SchemaElem::List(_) => "list",
            SchemaElem::Object(_) => "object",
            SchemaElem::Null => "null",
        }
    }
}

pub struct SchemaBuilder {
    schema: Schema,
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self {
            schema: Schema {
                value_type: ValueType::TypeObject,
                elem: SchemaElem::Null,
                schema_version: None,
                min_items: None,
                max_items: None,
                default: None,
                validate_fn: None,
                default_fn: None,
                description: None,
                state_fn: None,
                conflicts_with: None,
                exactly_one_of: None,
                atleast_one_of: None,
                required_with: None,
                sensitive: false,
                optional: false,
                required: false,
                computed: false,
                force_new: false,
            },
        }
    }

    pub fn value_type(mut self, vt: ValueType) -> Self {
        self.schema.value_type = vt;
        self
    }

    pub fn elem(mut self, elem: SchemaElem) -> Self {
        self.schema.elem = elem;
        self
    }

    pub fn schema_version(mut self, version: u64) -> Self {
        self.schema.schema_version = Some(version);
        self
    }

    pub fn min_items(mut self, min: u64) -> Self {
        self.schema.min_items = Some(min);
        self
    }

    pub fn max_items(mut self, max: u64) -> Self {
        self.schema.max_items = Some(max);
        self
    }

    pub fn default(mut self, d: SchemaElem) -> Self {
        self.schema.default = Some(d);
        self
    }

    pub fn validate_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&SchemaElem) -> bool + 'static,
    {
        self.schema.validate_fn = Some(Box::new(f));
        self
    }

    pub fn default_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Option<SchemaElem> + 'static,
    {
        self.schema.default_fn = Some(Box::new(f));
        self
    }

    pub fn description(mut self, desc: String) -> Self {
        self.schema.description = Some(desc);
        self
    }

    pub fn state_fn<F>(mut self, f: F) -> Self
    where
        F: Fn() -> String + 'static,
    {
        self.schema.state_fn = Some(Box::new(f));
        self
    }

    pub fn conflicts_with(mut self, conflicts: Vec<String>) -> Self {
        self.schema.conflicts_with = Some(conflicts);
        self
    }

    pub fn exactly_one_of(mut self, exactly_one: Vec<String>) -> Self {
        self.schema.exactly_one_of = Some(exactly_one);
        self
    }

    pub fn atleast_one_of(mut self, atleast_one: Vec<String>) -> Self {
        self.schema.atleast_one_of = Some(atleast_one);
        self
    }

    pub fn required_with(mut self, required: Vec<String>) -> Self {
        self.schema.required_with = Some(required);
        self
    }

    pub fn sensitive(mut self, sensitive: bool) -> Self {
        self.schema.sensitive = sensitive;
        self
    }

    pub fn optional(mut self, optional: bool) -> Self {
        self.schema.optional = optional;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.schema.required = required;
        self
    }

    pub fn computed(mut self, computed: bool) -> Self {
        self.schema.computed = computed;
        self
    }

    pub fn force_new(mut self, force_new: bool) -> Self {
        self.schema.force_new = force_new;
        self
    }

    pub fn build(self) -> Schema {
        self.schema
    }
}

impl Schema {
    pub fn default_value(&self) -> Option<SchemaElem> {
        if let Some(default_fn) = &self.default_fn {
            default_fn()
        } else {
            None
        }
    }

    pub fn validate_schema(&self) -> SchemaResult {
        self.validate_value_type(&mut String::from(""))?;

        Ok(())
    }

    fn validate_value_type(&self, path: &mut String) -> SchemaResult {
        match self.value_type {
            ValueType::TypeString => match &self.elem {
                SchemaElem::String(_) => Ok(()),
                _ => Err(SchemaValidationError::TypeMismatch(format!(
                    "Expected string type but found {:?} for {}",
                    self.elem.type_name(),
                    path
                ))),
            },
            ValueType::TypeInt => match &self.elem {
                SchemaElem::Int(_) => Ok(()),
                _ => Err(SchemaValidationError::TypeMismatch(format!(
                    "Expected int type but found {:?} for {}",
                    self.elem.type_name(),
                    path
                ))),
            },
            ValueType::TypeFloat => match &self.elem {
                SchemaElem::Float(_) => Ok(()),
                _ => Err(SchemaValidationError::TypeMismatch(format!(
                    "Expected float type but found {:?} for {}",
                    self.elem.type_name(),
                    path
                ))),
            },
            ValueType::TypeBool => match &self.elem {
                SchemaElem::Bool(_) => Ok(()),
                _ => Err(SchemaValidationError::TypeMismatch(format!(
                    "Expected bool type but found {:?} for {}",
                    self.elem.type_name(),
                    path
                ))),
            },
            ValueType::TypeList => match &self.elem {
                SchemaElem::List(schemas) => {
                    for (i, schema) in schemas.iter().enumerate() {
                        schema.validate_value_type(&mut format!("{}.{}", path, i))?;
                    }
                    Ok(())
                }
                _ => Err(SchemaValidationError::TypeMismatch(format!(
                    "Expected list type but found {:?} for {}",
                    self.elem.type_name(),
                    path
                ))),
            },
            ValueType::TypeObject => match &self.elem {
                SchemaElem::Object(_) => {
                    for (key, schema) in match &self.elem {
                        SchemaElem::Object(map) => map,
                        _ => unreachable!(),
                    } {
                        schema.validate_value_type(&mut format!("{}.{}", path, key))?;
                    }

                    Ok(())
                }
                _ => Err(SchemaValidationError::TypeMismatch(format!(
                    "Expected object type but found {:?}",
                    self.elem.type_name()
                ))),
            },
        }?;

        Ok(())
    }
}

impl fmt::Debug for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Schema")
            .field("value_type", &self.value_type)
            .field("elem", &self.elem)
            .field("schema_version", &self.schema_version)
            .field("min_items", &self.min_items)
            .field("max_items", &self.max_items)
            .field("has_default", &self.default.is_some())
            .field("has_validate_fn", &self.validate_fn.as_ref().map(|_| true))
            .field("has_default_fn", &self.default_fn.as_ref().map(|_| true))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let lambda = |elem: &SchemaElem| {
            // Basic predicate that accepts only string default values longer than 0
            match elem {
                SchemaElem::String(s) => !s.is_empty(),
                _ => true,
            }
        };

        let schema = SchemaBuilder::new()
            .value_type(ValueType::TypeString)
            .schema_version(1)
            .min_items(1)
            .max_items(10)
            .default(SchemaElem::String("instance_type".to_string()))
            .validate_fn(lambda)
            .build();

        // Exercise default_value path as well
        let _ = schema.default_value();

        println!("Schema is {:?}", schema);
    }

    #[test]
    fn test_validate_fn_with_valid_data() {
        // Create a validation function that checks string length > 3
        let validate_fn = |elem: &SchemaElem| match elem {
            SchemaElem::String(s) => s.len() > 3,
            _ => false,
        };

        let schema = SchemaBuilder::new()
            .value_type(ValueType::TypeString)
            .elem(SchemaElem::String("valid_string".to_string()))
            .validate_fn(validate_fn)
            .build();

        // Access the validation function
        if let Some(validator) = &schema.validate_fn {
            assert!(validator(&SchemaElem::String("test".to_string())));
            assert!(validator(&SchemaElem::String("hello".to_string())));
        }
    }

    #[test]
    fn test_validate_fn_with_invalid_data() {
        // Create a validation function that checks string length > 3
        let validate_fn = |elem: &SchemaElem| match elem {
            SchemaElem::String(s) => s.len() > 3,
            _ => false,
        };

        let schema = SchemaBuilder::new()
            .value_type(ValueType::TypeString)
            .elem(SchemaElem::String("ok".to_string()))
            .validate_fn(validate_fn)
            .build();

        // Access the validation function
        if let Some(validator) = &schema.validate_fn {
            assert!(!validator(&SchemaElem::String("ok".to_string())));
            assert!(!validator(&SchemaElem::String("no".to_string())));
            assert!(!validator(&SchemaElem::Int(123)));
        }
    }

    #[test]
    fn test_schema_validation_type_match() {
        // Test string type validation
        let string_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeString)
            .elem(SchemaElem::String("test".to_string()))
            .build();
        assert!(string_schema.validate_schema().is_ok());

        // Test int type validation
        let int_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeInt)
            .elem(SchemaElem::Int(42))
            .build();
        assert!(int_schema.validate_schema().is_ok());

        // Test float type validation
        let float_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeFloat)
            .elem(SchemaElem::Float(3.14))
            .build();
        assert!(float_schema.validate_schema().is_ok());

        // Test bool type validation
        let bool_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeBool)
            .elem(SchemaElem::Bool(true))
            .build();
        assert!(bool_schema.validate_schema().is_ok());
    }

    #[test]
    fn test_schema_validation_type_mismatch() {
        // Test string type with wrong elem
        let string_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeString)
            .elem(SchemaElem::Int(42))
            .build();
        let result = string_schema.validate_schema();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SchemaValidationError::TypeMismatch(_)
        ));

        // Test int type with wrong elem
        let int_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeInt)
            .elem(SchemaElem::String("not an int".to_string()))
            .build();
        let result = int_schema.validate_schema();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SchemaValidationError::TypeMismatch(_)
        ));

        // Test bool type with wrong elem
        let bool_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeBool)
            .elem(SchemaElem::Float(3.14))
            .build();
        let result = bool_schema.validate_schema();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SchemaValidationError::TypeMismatch(_)
        ));
    }

    #[test]
    fn test_validate_fn_with_numeric_range() {
        // Create a validation function for integers in range 0-100
        let validate_int_range = |elem: &SchemaElem| match elem {
            SchemaElem::Int(i) => *i >= 0 && *i <= 100,
            _ => false,
        };

        let schema = SchemaBuilder::new()
            .value_type(ValueType::TypeInt)
            .elem(SchemaElem::Int(50))
            .validate_fn(validate_int_range)
            .build();

        if let Some(validator) = &schema.validate_fn {
            // Valid values
            assert!(validator(&SchemaElem::Int(0)));
            assert!(validator(&SchemaElem::Int(50)));
            assert!(validator(&SchemaElem::Int(100)));

            // Invalid values
            assert!(!validator(&SchemaElem::Int(-1)));
            assert!(!validator(&SchemaElem::Int(101)));
        }
    }

    #[test]
    fn test_validate_fn_with_pattern_matching() {
        // Create a validation function that checks if string starts with "aws_"
        let validate_prefix = |elem: &SchemaElem| match elem {
            SchemaElem::String(s) => s.starts_with("aws_"),
            _ => false,
        };

        let schema = SchemaBuilder::new()
            .value_type(ValueType::TypeString)
            .elem(SchemaElem::String("aws_instance".to_string()))
            .validate_fn(validate_prefix)
            .build();

        if let Some(validator) = &schema.validate_fn {
            // Valid values
            assert!(validator(&SchemaElem::String("aws_instance".to_string())));
            assert!(validator(&SchemaElem::String("aws_bucket".to_string())));
            assert!(validator(&SchemaElem::String("aws_".to_string())));

            // Invalid values
            assert!(!validator(&SchemaElem::String("gcp_instance".to_string())));
            assert!(!validator(&SchemaElem::String("azure_vm".to_string())));
            assert!(!validator(&SchemaElem::String("".to_string())));
        }
    }

    #[test]
    fn test_list_schema_validation() {
        // Create a list schema with nested string schemas
        let item_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeString)
            .elem(SchemaElem::String("item".to_string()))
            .build();

        let list_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeList)
            .elem(SchemaElem::List(vec![item_schema]))
            .build();

        assert!(list_schema.validate_schema().is_ok());
    }

    #[test]
    fn test_object_schema_validation() {
        // Create an object schema with nested schemas
        let mut fields = BTreeMap::new();

        fields.insert(
            "name".to_string(),
            SchemaBuilder::new()
                .value_type(ValueType::TypeString)
                .elem(SchemaElem::String("".to_string()))
                .build(),
        );

        fields.insert(
            "age".to_string(),
            SchemaBuilder::new()
                .value_type(ValueType::TypeInt)
                .elem(SchemaElem::Int(0))
                .build(),
        );

        let object_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeObject)
            .elem(SchemaElem::Object(fields))
            .build();

        assert!(object_schema.validate_schema().is_ok());
    }

    #[test]
    fn test_nested_validation_error() {
        // Create a list schema with an invalid nested schema (type mismatch)
        let invalid_item = SchemaBuilder::new()
            .value_type(ValueType::TypeString)
            .elem(SchemaElem::Int(42)) // Wrong type!
            .build();

        let list_schema = SchemaBuilder::new()
            .value_type(ValueType::TypeList)
            .elem(SchemaElem::List(vec![invalid_item]))
            .build();

        let result = list_schema.validate_schema();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SchemaValidationError::TypeMismatch(_)
        ));
    }
}
