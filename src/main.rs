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
    repl();
}
fn repl() {
    println!("welcome to mindb!");
    let mut engine = Engine::new();
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        run(&mut engine, &input);
    }
}
