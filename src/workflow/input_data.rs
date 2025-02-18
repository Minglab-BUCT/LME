use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::sparse_molecule::SparseMolecule;
use serde::{Deserialize, Serialize};

use super::step::Steps;
use super::workflow_data::{LayerStorageConfig, Window};

#[derive(Deserialize, Default, Debug)]
pub struct WorkflowInput {
    #[serde(default)]
    pub binaries: Vec<PathBuf>,
    #[serde(default)]
    pub base: SparseMolecule,
    pub steps: Steps,
}

#[derive(Deserialize, Serialize)]
pub struct WorkflowCheckPoint {
    pub skip: usize,
    pub base: SparseMolecule,
    pub layers: LayerStorageConfig,
    pub windows: BTreeMap<String, Window>,
    pub current_window: Window,
}
