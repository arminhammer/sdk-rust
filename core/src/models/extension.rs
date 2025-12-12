use crate::models::map::*;
use crate::models::task::*;
use serde_derive::{Deserialize, Serialize};

/// Represents the definition of a an extension
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtensionDefinition {
    Named(std::collections::HashMap<String, ExtensionFields>),
}

impl Default for ExtensionDefinition {
    fn default() -> Self {
        ExtensionDefinition::Named(std::collections::HashMap::new())
    }
}

/// Represents the fields of an extension definition
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionFields {
    /// Gets/sets the type of task to extend
    #[serde(rename = "extend")]
    pub extend: String,

    /// Gets/sets a runtime expression, if any, used to determine whether or not the extension should apply in the specified context
    #[serde(rename = "when", skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,

    /// Gets/sets a name/definition mapping, if any, of the tasks to execute before the extended task
    #[serde(rename = "before", skip_serializing_if = "Option::is_none")]
    pub before: Option<Map<String, TaskDefinition>>,

    /// Gets/sets a name/definition mapping, if any, of the tasks to execute after the extended task
    #[serde(rename = "after", skip_serializing_if = "Option::is_none")]
    pub after: Option<Map<String, TaskDefinition>>,
}
