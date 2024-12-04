use std::collections::HashMap;

use super::{parser::ASTNode, query::Identifier, schema::Row, storage_engine::FileSystem};

pub struct QueryExecutor<'a> {
    filesystem: &'a mut FileSystem,
}

impl<'a> QueryExecutor<'a> {
    pub fn new(filesystem: &'a mut FileSystem) -> Self {
        QueryExecutor { filesystem }
    }

    pub fn execute(&self, query: ASTNode) -> Result<Vec<Row>, ExecutionError> {
        match query {
            ASTNode::SelectStatement { projection, table } => {
                Ok(self.execute_select(projection, table)?)
            }
            ASTNode::DeleteStatement { table, condition } => {
                todo!()
            }
            ASTNode::InsertStatement {
                table,
                columns,
                values,
            } => {
                todo!()
            }
            ASTNode::UpdateStatement {
                table,
                assignments,
                condition,
            } => {
                // Ok(self.execute_update(table, updates, condition)?)
                todo!()
            }
            ASTNode::Identifier(_) => {
                todo!()
            }
        }
    }

    fn execute_select(
        &self,
        projection: Vec<Identifier>,
        table: Identifier,
    ) -> Result<Vec<Row>, ExecutionError> {
        let table = self
            .filesystem
            .storage_engine
            .tables
            .get(&table.0)
            .ok_or(ExecutionError::TableNotFound)?;
        // println!("rows {:?} and projection: {:?}", table.rows, &projection);
        let mut result = Vec::new();

        for row in table.rows.values() {
            let mut row_data = HashMap::new();
            for column in &projection {
                row_data.insert(
                    column.0.clone(),
                    row.data.get(&column.0).cloned().unwrap_or_default(),
                );
            }
            result.push(Row { data: row_data });
        }
        Ok(result)
    }

    fn execute_update<F>(
        &mut self,
        table: Identifier,
        updates: HashMap<String, String>,
        condition: F,
    ) -> Result<(), String>
    where
        F: Fn(&Row) -> bool,
    {
        self.filesystem.update_rows(&table.0, updates, condition)?;
        Ok(())
    }

    fn execute_insert() {}

    fn execute_delete(&mut self, table: Identifier) {
        self.filesystem.delete_rows(&table.0, |row| {
            row.data
                .get("age")
                .map_or(false, |age| age.parse::<i32>().unwrap_or(0) > 30)
        });
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    TableNotFound,
}

// pub struct QueryExecutor<'a> {
//     filesystem: &'a mut FileSystem,
// }

// impl<'a> QueryExecutor<'a> {
//     pub fn new(filesystem: &'a mut FileSystem) -> Self {
//         QueryExecutor { filesystem }
//     }

//     pub fn execute(&mut self, query: ASTNode) -> Result<(), String> {
//         match query {
//             ASTNode::SelectStatement { projection, table } => {
//                 self.execute_select(projection, table)
//             }
//             ASTNode::DeleteStatement { table, condition } => self.execute_delete(table, condition),
//             ASTNode::UpdateStatement {
//                 table,
//                 updates,
//                 condition,
//             } => self.execute_update(table, updates, condition),
//             ASTNode::InsertStatement { table, values } => self.execute_insert(table, values),
//         }
//     }

//     fn execute_select(&self, projection: Vec<Identifier>, table: Identifier) -> Result<(), String> {
//         // Currently, FileSystem does not support SELECT operations.
//         Err("SELECT queries are not yet implemented".to_string())
//     }

//     fn execute_delete<F>(&mut self, table: Identifier, condition: F) -> Result<(), String>
//     where
//         F: Fn(&Row) -> bool,
//     {
//         self.filesystem.delete_rows(&table.0, condition);
//         Ok(())
//     }

//     fn execute_update<F>(
//         &mut self,
//         table: Identifier,
//         updates: HashMap<String, String>,
//         condition: F,
//     ) -> Result<(), String>
//     where
//         F: Fn(&Row) -> bool,
//     {
//         self.filesystem.update_rows(&table.0, updates, condition)?;
//         Ok(())
//     }

//     fn execute_insert(&mut self, table: Identifier, values: HashMap<String, String>) -> Result<(), String> {
//         self.filesystem.insert_row(&table.0, Row { data: values })?;
//         Ok(())
//     }
// }
