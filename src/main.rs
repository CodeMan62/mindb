mod parser;
mod storage;
mod tokenizer;
mod schema;
mod row;
mod execution;
use execution::execution::Engine;
use parser::parse;

use crate::execution::execution::ExecuteResult;

pub fn print_result(result: ExecuteResult) {
    match result {
        ExecuteResult::Created(name) => println!("table \"{}\" created.", name),
        ExecuteResult::Inserted => println!("1 row inserted."),
        ExecuteResult::Message(msg) => println!("{}", msg),
        ExecuteResult::Rows { headers, rows } => {
            if rows.is_empty() {
                println!("(0 rows)");
                return;
            }
            let widths: Vec<usize> = (0..headers.len())
                .map(|i| {
                    rows.iter()
                        .map(|r| r[i].len())
                        .max()
                        .unwrap_or(0)
                        .max(headers[i].len())
                })
                .collect();

            print_row_line(&headers, &widths);
            println!(
                "{}",
                widths
                    .iter()
                    .map(|&w| "-".repeat(w + 2))
                    .collect::<Vec<_>>()
                    .join("+")
            );
            for row in &rows {
                print_row_line(row, &widths);
            }
            println!(
                "({} row{})",
                rows.len(),
                if rows.len() == 1 { "" } else { "s" }
            );
        }
    }
}

fn print_row_line(cols: &[String], widths: &[usize]) {
    let parts: Vec<String> = cols
        .iter()
        .zip(widths)
        .map(|(v, &w)| format!(" {:<w$} ", v))
        .collect();
    println!("{}", parts.join("|"));
}
fn run(engine: &mut Engine, sql: &str) {
    println!("mindb> {}", sql);
    match parse(sql) {
        Ok(stmt) => match engine.execute(stmt) {
            Ok(result) => print_result(result),
            Err(e) => println!("Error: {}", e),
        },
        Err(e) => println!("Parse error: {}", e),
    }
}

fn main() {
    let mut engine = Engine::new();

    run(&mut engine, "CREATE TABLE users (name TEXT, score INT)");

    //run(&mut engine, "INSERT INTO users VALUES (1, alice, 980)");
    //run(&mut engine, "INSERT INTO users VALUES (2, bob, 870)");
    //run(&mut engine, "INSERT INTO users VALUES (3, carol, 990)");

    //run(&mut engine, "SELECT * FROM users");

    //run(&mut engine, "SELECT name FROM users WHERE id = 2");

    //run(&mut engine, "SELECT * FROM users WHERE score > 900");

    //run(&mut engine, "SELECT * FROM orders");
    //run(&mut engine, "INSERT INTO users VALUES (4, dave)");
    //run(&mut engine, "CREATE TABLE users (name TEXT, score INT)");
}
