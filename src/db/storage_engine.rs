use std::{collections::HashMap, fs::{File, OpenOptions}, path::Path};
use serde::{Deserialize, Serialize};
use super::schema::{Row, Table};
use std::io::{Read, Write};


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct StorageEngine {
    pub tables: HashMap<String, Table>,
}

impl StorageEngine {
    pub fn new() -> Self {
        StorageEngine {
            tables: HashMap::new(),
        }
    }

    /// Delete rows from a table based on a condition
    pub fn delete_rows<F>(&mut self, table_name: &str, condition: F)
    where
        F: Fn(&Row) -> bool,
    {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.rows.retain(|_, row| !condition(row));
        }
    }

    pub fn update_rows<F>(
        &mut self,
        table_name: &str,
        updates: HashMap<String, String>,
        condition: F,
    ) -> Result<(), String>
    where
        F: Fn(&Row) -> bool,
    {
        if let Some(table) = self.tables.get_mut(table_name) {
            if let Some(pk) = &table.primary_key {
                if let Some(new_pk_value) = updates.get(pk) {
                    if table.rows.values().any(|r| r.data.get(pk) == Some(new_pk_value)) {
                        return Err(format!(
                            "Duplicate value '{}' for primary key '{}'",
                            new_pk_value, pk
                        ));
                    }
                }
            }
    
            for row in table.rows.values_mut() {
                if condition(row) {
                    for (column, value) in &updates {
                        row.data.insert(column.clone(), value.clone());
                    }
                }
            }
            Ok(())
        } else {
            Err(format!("Table '{}' does not exist", table_name))
        }
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<String>, primary_key: Option<&str>) {
        if let Some(pk) = primary_key {
            if !columns.contains(&pk.to_string()) {
                panic!("Primary key '{}' must be one of the table columns", pk);
            }
        }

        self.tables.insert(
            name.to_string(),
            Table {
                columns,
                rows: HashMap::new(),
                primary_key: primary_key.map(String::from),
            },
        );
    }

    pub fn insert_row(&mut self, table_name: &str, row: Row) -> Result<(), String> {
        if let Some(table) = self.tables.get_mut(table_name) {
            // Validate uniqueness for the primary key
            if let Some(pk) = &table.primary_key {
                if let Some(pk_value) = row.data.get(pk) {
                    if table.rows.values().any(|r| r.data.get(pk) == Some(pk_value)) {
                        return Err(format!(
                            "Duplicate value '{}' for primary key '{}'",
                            pk_value, pk
                        ));
                    }
                } else {
                    return Err(format!(
                        "Missing value for primary key '{}'",
                        pk
                    ));
                }
            }

            let row_id = table.rows.len();
            table.rows.insert(row_id, row);
            Ok(())
        } else {
            Err(format!("Table '{}' does not exist", table_name))
        }
    }


    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buffer.clear();
        buffer.extend(bincode::serialize(self).unwrap());
        Ok(())
    }


    pub fn deserialize(buffer: &[u8]) -> Result<Self, std::io::Error> {
        Ok(bincode::deserialize(buffer).unwrap())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileSystem {
    pub storage_engine: StorageEngine,
    file_path: String,
}

impl FileSystem {
    pub fn new(file_path: &str) -> Self {
        let mut storage_engine = StorageEngine::new();
        if Path::new(file_path).exists() {
            storage_engine = FileSystem::load_from_file(file_path).unwrap_or(StorageEngine::new());
        }
        FileSystem {
            storage_engine,
            file_path: file_path.to_string(),
        }
    }

    pub fn create_table(
        &mut self,
        name: &str,
        columns: Vec<String>,
        primary_key: Option<&str>,
    ) {
        self.storage_engine.create_table(name, columns, primary_key);
        self.save_to_file().unwrap();
    }

    pub fn insert_row(&mut self, table_name: &str, row: Row) -> Result<(), String> {
        let result = self.storage_engine.insert_row(table_name, row);
        self.save_to_file().unwrap();
        result
    }

    fn save_to_file(&self) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;

        let mut buffer = Vec::new();
        self.storage_engine.serialize(&mut buffer)?;
        file.write_all(&buffer)?;
        Ok(())
    }

    fn load_from_file(file_path: &str) -> Result<StorageEngine, std::io::Error> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let storage_engine = StorageEngine::deserialize(&buffer)?;
        Ok(storage_engine)
    }

    /// Delete rows and persist the changes
    pub fn delete_rows<F>(&mut self, table_name: &str, condition: F)
    where
        F: Fn(&Row) -> bool,
    {
        self.storage_engine.delete_rows(table_name, condition);
        self.save_to_file().unwrap();
    }

    /// Update rows and persist the changes
    pub fn update_rows<F>(&mut self, table_name: &str, updates: HashMap<String, String>, condition: F) -> Result<(), String>
    where
        F: Fn(&Row) -> bool,
    {
        let result = self.storage_engine.update_rows(table_name, updates, condition);
        self.save_to_file().unwrap();
        result
    }
}