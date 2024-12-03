use db::{executor::ExecutionEngine, lexer::Tokenizer, parser::Parser, query::QueryPlanner, schema::Row, storage_engine::FileSystem};
use std::collections::HashMap;
mod db;

fn main() {
    // let mut filesystem = FileSystem::new("database.db");
    // filesystem.create_table("users", vec!["id".to_string(), "name".to_string(), "email".to_string()]);
    // filesystem.insert_row("users", Row{
    //     data: vec![
    //         ("id".to_string(), "1".to_string()),
    //         ("name".to_string(), "anthony etienne".to_string()),
    //         ("email".to_string(), "anthony.etienne@gmail.com".to_string()),
    //     ].into_iter().collect(),
    // });
    // filesystem.insert_row("users", Row{
    //     data: vec![
    //         ("id".to_string(), "2".to_string()),
    //         ("name".to_string(), "etienne anthony".to_string()),
    //         ("email".to_string(), "etienne.anthony@gmail.com".to_string()),
    //     ].into_iter().collect(),
    // });

    // let input = "SELECT * FROM users";
    // let tokens = Tokenizer::tokenize(input);
    // let parser = Parser::new(tokens);
    // let ast = parser.parse(input).unwrap();
    // let query_planner = QueryPlanner::new();
    // let query_plan = query_planner.plan(&ast);

    // // execution engine
    // let execution_engine = ExecutionEngine::new(filesystem.storage_engine.clone());
    // let result = execution_engine.execute(&query_plan).unwrap();
    // println!("the result: {:?}", result)

    let mut fs = FileSystem::new("database.bin");

    // Create table
    fs.create_table(
        "users",
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
        Some("id"), // Specify 'id' as the primary key
    );

    // Insert rows
    fs.insert_row("users", Row {
        data: HashMap::from([
            ("id".to_string(), "1".to_string()),
            ("name".to_string(), "Alice".to_string()),
            ("age".to_string(), "25".to_string())
        ])
    }).unwrap();
    let result = fs.insert_row("users", Row {
        data: HashMap::from([
            ("id".to_string(), "2".to_string()),
            ("name".to_string(), "Alice".to_string()),
            ("age".to_string(), "35".to_string())
        ])
    });

    // if let Err(err) = result {
    println!("Error: {:?}", result);
    // }

    // Delete rows where age > 30
    fs.delete_rows("users", |row| {
        row.data.get("age").map_or(false, |age| age.parse::<i32>().unwrap_or(0) > 30)
    });

    // Update rows where id = 1
    let mut updates = HashMap::new();
    updates.insert("name".to_string(), "Updated Alice".to_string());
    let res = fs.update_rows("users", updates, |row| {
        row.data.get("id").map_or(false, |id| id == "1")
    });
    println!("res: {:?}", res);

    println!("{:?}", fs.storage_engine.tables);
}
