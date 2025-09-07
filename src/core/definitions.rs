use serde::{Deserialize, Serialize};

/// BPMN 2.0 Definitions root element
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definitions {
    pub name: Option<String>,
    pub target_namespace: Option<String>,
    pub processes: Vec<Process>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Process {
    pub id: String,
    pub name: Option<String>,
}

