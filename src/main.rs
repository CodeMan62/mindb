mod executor;
mod parser;
mod row;
mod storage;
mod tokenizer;
mod execute;

use executor::{print_result, Engine};
use parser::parse;

fn run(engine: &mut Engine, sql: &str) {
    println!("mindb> {}", sql);
    match parse(sql) {
        Ok(stmt) => match engine.execute(stmt) {
            Ok(result) => print_result(result),
            Err(e) => println!("Error: {}", e),
        },
        Err(e) => println!("Parse error: {}", e),
    }
    println!();
}

fn main() {
    let _ = std::fs::remove_file("users.db");
    let mut engine = Engine::new();

    run(&mut engine, "CREATE TABLE users (name TEXT, score INT)");

    run(&mut engine, "INSERT INTO users VALUES (1, alice, 980)");
    run(&mut engine, "INSERT INTO users VALUES (2, bob, 870)");
    run(&mut engine, "INSERT INTO users VALUES (3, carol, 990)");

    run(&mut engine, "SELECT * FROM users");

    run(&mut engine, "SELECT name FROM users WHERE id = 2");

    run(&mut engine, "SELECT * FROM users WHERE score > 900");

    // error cases — show proper error messages
    run(&mut engine, "SELECT * FROM orders");
    run(&mut engine, "INSERT INTO users VALUES (4, dave)");
    run(&mut engine, "CREATE TABLE users (name TEXT, score INT)");
}
