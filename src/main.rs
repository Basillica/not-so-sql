use db::{executor::ExecutionEngine, parser::Parser, query::QueryPlanner, schema::Row, storage_engine::FileSystem};



mod db;


fn main() {
    let mut filesystem = FileSystem::new("database.db");
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

    let input = "SELECT * FROM users";
    let ast = Parser::parse(input).unwrap();
    let query_planner = QueryPlanner::new();
    let query_plan = query_planner.plan(&ast);

    // execution engine
    let execution_engine = ExecutionEngine::new(filesystem.storage_engine.clone());
    let result = execution_engine.execute(&query_plan).unwrap();
    println!("the result: {:?}", result)
}
