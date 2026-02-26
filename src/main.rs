use std::io::{self, Write};
mod tokenizer;
use tokenizer::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("Rust simple REPL. Type 'exit' to quit.");
    //run_repl();
    let input = "SELECT * FROM users".to_string();
    let mut tokenizer = Tokenizer::new(input);
    let tokens = tokenizer.tokenize();
    for token in tokens {
        println!("{:?}", token);
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
