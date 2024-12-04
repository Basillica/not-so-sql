use db::{
    executor::QueryExecutor, lexer::Tokenizer, parser::Parser, schema::Row,
    storage_engine::FileSystem,
};
use std::collections::HashMap;
mod db;

fn main() {
    let mut filesystem = FileSystem::new("database.db");
    filesystem.create_table(
        "users",
        vec![
            "id".to_string(),
            "name".to_string(),
            "email".to_string(),
            "age".to_string(),
        ],
        Some("id"), // Specify 'id' as the primary key
    );
    filesystem
        .insert_row(
            "users",
            Row {
                data: HashMap::from([
                    ("id".to_string(), "1".to_string()),
                    ("name".to_string(), "anthony etienne".to_string()),
                    ("name".to_string(), "etienne anthony".to_string()),
                    ("email".to_string(), "anthony.etienne@gmail.com".to_string()),
                ]),
            },
        )
        .unwrap();
    filesystem
        .insert_row(
            "users",
            Row {
                data: HashMap::from([
                    ("id".to_string(), "2".to_string()),
                    ("name".to_string(), "anthony etienne".to_string()),
                    ("name".to_string(), "etienne anthony".to_string()),
                    ("email".to_string(), "anthony.etienne@gmail.com".to_string()),
                ]),
            },
        )
        .unwrap();

    let input = "SELECT id, email FROM users WHERE id = '2'";
    let tokens = Tokenizer::tokenize(input);
    let _parser = Parser::new(tokens);
    // let ast = parser.parse(input).unwrap();

    let ast = Parser::parse(input).unwrap();

    // execution engine
    let execution_engine = QueryExecutor::new(&mut filesystem);
    let result = execution_engine.execute(ast).unwrap();
    println!("the result: {:?}", result);

    // Delete rows where age > 30
    filesystem.delete_rows("users", |row| {
        row.data
            .get("age")
            .map_or(false, |age| age.parse::<i32>().unwrap_or(0) > 30)
    });

    // Update rows where id = 1
    let mut updates = HashMap::new();
    updates.insert("name".to_string(), "Updated Alice".to_string());
    let res = filesystem.update_rows("users", updates, |row| {
        row.data.get("id").map_or(false, |id| id == "1")
    });
    println!("res: {:?}", res);

    println!("{:?}", filesystem.storage_engine.tables);
}
