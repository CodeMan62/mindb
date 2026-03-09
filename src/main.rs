use std::io::{self, Write};
mod tokenizer;
mod parser;
mod row;
mod storage;

use row::{Row, Schema};
use storage::table::Table;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "users.db";
    let schema = Schema::new(&["name", "email"]);
    let mut table = Table::open(path, schema)?;

    table.insert(&Row::new(1, &["alice", "alice@example.com"]))?;
    table.insert(&Row::new(2, &["bob", "bob@example.com"]))?;
    table.insert(&Row::new(3, &["carol", "carol@example.com"]))?;

    println!("schema cols: {}", table.schema.col_count);
    println!(
        "columns: {}",
        (0..table.schema.col_count)
            .map(|i| table.schema.cols[i].name_str().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("row_count: {}", table.row_count);

    println!();
    for row in table.scan()? {
        let vals: Vec<&str> = (0..table.schema.col_count)
            .map(|i| row.values[i].as_str())
            .collect();
        println!("id={} | {}", row.id, vals.join(" | "));
    }

    Ok(())
}


fn run_repl(){
        print!("▖  ▖▄▖▖ ▖▄ ▄
▛▖▞▌▐ ▛▖▌▌▌▙▘
▌▝ ▌▟▖▌▝▌▙▘▙▘
             \n");
    loop {
        print!("mindb> ");
        let mut input = String::new();
        io::stdout().flush();
        io::stdin().read_line(&mut input).expect("h");
        let trimmed_input = input.trim();
        if trimmed_input == "exit" {
            break;
        }
        if trimmed_input.is_empty(){
            continue;
        }
        println!("{}", trimmed_input);
    }
}
