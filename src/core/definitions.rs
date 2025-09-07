use serde::{Deserialize, Serialize};
use crate::types::non_empty::NonEmptyVec;

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
    /// Start events (at least one required - Make Invalid States Unrepresentable)
    pub start_events: NonEmptyVec<StartEvent>,
    pub tasks: Vec<Task>,
    pub gateways: Vec<Gateway>,
    pub end_events: Vec<EndEvent>,
    pub sequence_flows: Vec<SequenceFlow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartEvent {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gateway {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndEvent {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceFlow {
    pub id: String,
    pub name: Option<String>,
    pub source_ref: String,
    pub target_ref: String,
}

