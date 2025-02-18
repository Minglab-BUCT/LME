use crate::{layer::Layer, sparse_molecule::SparseMolecule};
use redb::{Database, ReadableTableMetadata, TableDefinition};
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    path::PathBuf,
};

const LAYER_TABLE: TableDefinition<u64, Layer> = TableDefinition::new("layer_table");

use serde::{Deserialize, Serialize};

pub type Window = BTreeMap<String, Vec<u64>>;

#[derive(Deserialize, Serialize)]
pub struct WorkflowData {
    pub base: SparseMolecule,
    pub layers: LayerStorage,
    pub current_window: Window,
}

#[derive(Deserialize, Serialize)]
#[serde(try_from = "LayerStorageConfig")]
pub struct LayerStorage {
    db_path: PathBuf,
    #[serde(skip)]
    db: Database,
}

impl LayerStorage {
    pub fn new(db_path: PathBuf) -> Self {
        let db = Database::create(&db_path)
            .or(Database::open(&db_path))
            .unwrap();
        Self { db_path, db }
    }

    pub fn retain(&self, retains: &BTreeSet<u64>) {
        let writer = self.db.begin_write().unwrap();
        {
            let mut table = writer.open_table(LAYER_TABLE).unwrap();
            table.retain(|k, _| retains.contains(&k)).unwrap();
        }
        writer.commit().unwrap();
    }
}

#[derive(Deserialize, Serialize)]
pub struct LayerStorageConfig {
    db_path: PathBuf,
}

impl TryFrom<LayerStorageConfig> for LayerStorage {
    type Error = anyhow::Error;
    fn try_from(value: LayerStorageConfig) -> Result<Self, Self::Error> {
        let db = Database::create(&value.db_path).or(Database::open(&value.db_path))?;
        Ok(Self {
            db_path: value.db_path,
            db,
        })
    }
}

impl LayerStorage {
    fn next_layer_id(&self) -> u64 {
        let read_txn = self.db.begin_read().unwrap();
        if let Ok(table) = read_txn.open_table(LAYER_TABLE) {
            table.len().unwrap()
        } else {
            0
        }
    }

    pub fn create_layers(&self, layers: &[Layer]) -> Range<u64> {
        let start_id = self.next_layer_id();
        let write_txn = self.db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(LAYER_TABLE).unwrap();
            for (idx, layer) in layers.into_iter().enumerate() {
                table.insert(start_id + idx as u64, layer.clone()).unwrap();
            }
        }
        write_txn.commit().unwrap();
        start_id..self.next_layer_id()
    }

    pub fn read_layer(&self, layer_id: u64) -> Option<Layer> {
        self.db
            .begin_read()
            .unwrap()
            .open_table(LAYER_TABLE)
            .unwrap()
            .get(layer_id)
            .unwrap()
            .map(|acc| acc.value())
    }
}
