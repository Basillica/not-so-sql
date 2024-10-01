use std::collections::HashMap;

use super::{query::QueryPlan, schema::Row, storage_engine::StorageEngine};



pub struct ExecutionEngine {
    storage_engine: StorageEngine,
}

impl ExecutionEngine {
    pub fn new(storage_engine: StorageEngine) -> Self {
        ExecutionEngine { storage_engine }
    }

    pub fn execute(&self, query_plan: &QueryPlan) -> Result<Vec<Row>, ExecutionError> {
        let table = self.storage_engine.tables.get(&query_plan.table.0).ok_or(ExecutionError::TableNotFound(query_plan.table.0.clone()))?;
        println!("rows {:?} and projection: {:?}", table.rows, &query_plan.projection);
        let mut result = Vec::new();
        for row in table.rows.values() {
            let mut row_data = HashMap::new();
            for column in &query_plan.projection {
                row_data.insert(column.0.clone(), row.data.get(&column.0).cloned().unwrap_or_default());
            }
            result.push(Row { data: row_data });
        }
        Ok(result)
    }
}


#[derive(Debug)]
pub enum ExecutionError {
    TableNotFound(String),
}