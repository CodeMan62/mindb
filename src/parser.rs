// parser for mindb
use crate::tokenizer::{Token, TokenType, Tokenizer};

#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Exit,
    Help,
    CreateTable {
        table: String,
        columns: Vec<ColumnDef>,
    },
    Insert {
        table: String,
        values: Vec<Expression>,
    },
    Select {
        columns: Vec<String>, // "*" means all
        table: String,
        where_clause: Option<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(String),
    ColumnRef(String),
    BinaryOp {
        op: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
}

fn skip_whitespace(tokens: &[Token], pos: usize) -> usize {
    let mut i = pos;
    while i < tokens.len() {
        if let TokenType::Unknown = tokens[i].token_type {
            i += 1;
        } else {
            break;
        }
    }
    i
}

fn parse_error(msg: &str, pos: usize) -> String {
    format!("Parse error at token {}: {}", pos, msg)
}

fn expect_keyword(tokens: &[Token], pos: usize, kw: &str) -> Result<usize, String> {
    let pos = skip_whitespace(tokens, pos);
    if pos >= tokens.len() {
        return Err(parse_error(
            &format!("expected keyword '{}', got end of input", kw),
            pos,
        ));
    }
    if let TokenType::Keyword = tokens[pos].token_type {
        if tokens[pos].value.to_uppercase() == kw.to_uppercase() {
            return Ok(pos + 1);
        }
    }
    Err(parse_error(
        &format!("expected keyword '{}', got '{}'", kw, tokens[pos].value),
        pos,
    ))
}

fn expect_symbol(tokens: &[Token], pos: usize, sym: &str) -> Result<usize, String> {
    let pos = skip_whitespace(tokens, pos);
    if pos >= tokens.len() {
        return Err(parse_error(
            &format!("expected '{}', got end of input", sym),
            pos,
        ));
    }
    if let TokenType::Symbol = tokens[pos].token_type {
        if tokens[pos].value == sym {
            return Ok(pos + 1);
        }
    }
    Err(parse_error(
        &format!("expected '{}', got '{}'", sym, tokens[pos].value),
        pos,
    ))
}

fn expect_identifier(tokens: &[Token], pos: usize) -> Result<(String, usize), String> {
    let pos = skip_whitespace(tokens, pos);
    if pos >= tokens.len() {
        return Err(parse_error("expected identifier, got end of input", pos));
    }
    if let TokenType::Identifier = tokens[pos].token_type {
        return Ok((tokens[pos].value.clone(), pos + 1));
    }
    Err(parse_error(
        &format!("expected identifier, got '{}'", tokens[pos].value),
        pos,
    ))
}

fn parse_comparison(tokens: &[Token], pos: usize) -> Result<(Expression, usize), String> {
    let pos = skip_whitespace(tokens, pos);
    if pos >= tokens.len() {
        return Err(parse_error("expected expression", pos));
    }

    let (left, pos) = parse_primary(tokens, pos)?;
    let pos = skip_whitespace(tokens, pos);

    if pos >= tokens.len() {
        return Ok((left, pos));
    }

    let op_val = tokens[pos].value.clone();
    let is_op = matches!(op_val.as_str(), "=" | "!" | "<" | ">");
    if !is_op {
        return Ok((left, pos));
    }

    let (op, pos) = collect_operator(tokens, pos);
    let pos = skip_whitespace(tokens, pos);
    let (right, pos) = parse_primary(tokens, pos)?;

    Ok((
        Expression::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
        pos,
    ))
}

fn collect_operator(tokens: &[Token], pos: usize) -> (String, usize) {
    let first = &tokens[pos].value;
    if pos + 1 < tokens.len() {
        if let TokenType::Symbol = tokens[pos + 1].token_type {
            let combined = format!("{}{}", first, tokens[pos + 1].value);
            if matches!(combined.as_str(), "!=" | "<=" | ">=") {
                return (combined, pos + 2);
            }
        }
    }
    (first.clone(), pos + 1)
}

fn parse_expr(tokens: &[Token], pos: usize) -> Result<(Expression, usize), String> {
    let pos = skip_whitespace(tokens, pos);
    let (mut left, mut pos) = parse_comparison(tokens, pos)?;

    loop {
        let p = skip_whitespace(tokens, pos);
        if p >= tokens.len() {
            break;
        }
        match tokens[p].value.to_uppercase().as_str() {
            "AND" => {
                let (right, next) = parse_comparison(tokens, skip_whitespace(tokens, p + 1))?;
                left = Expression::And(Box::new(left), Box::new(right));
                pos = next;
            }
            "OR" => {
                let (right, next) = parse_comparison(tokens, skip_whitespace(tokens, p + 1))?;
                left = Expression::Or(Box::new(left), Box::new(right));
                pos = next;
            }
            _ => break,
        }
    }

    Ok((left, pos))
}

fn parse_primary(tokens: &[Token], pos: usize) -> Result<(Expression, usize), String> {
    let pos = skip_whitespace(tokens, pos);
    if pos >= tokens.len() {
        return Err(parse_error("expected value, got end of input", pos));
    }
    match tokens[pos].token_type {
        TokenType::Number => Ok((Expression::Literal(tokens[pos].value.clone()), pos + 1)),
        TokenType::Identifier => Ok((Expression::ColumnRef(tokens[pos].value.clone()), pos + 1)),
        TokenType::Unknown => {
            let v = tokens[pos].value.clone();
            Ok((Expression::Literal(v), pos + 1))
        }
        TokenType::Keyword => {
            Ok((Expression::Literal(tokens[pos].value.clone()), pos + 1))
        }
        TokenType::Symbol => Err(parse_error(
            &format!("unexpected symbol '{}'", tokens[pos].value),
            pos,
        )),
    }
}

fn parse_create_table(tokens: &[Token], mut pos: usize) -> Result<Statement, String> {
    pos = expect_keyword(tokens, pos, "TABLE")?;
    let (table, mut pos) = expect_identifier(tokens, pos)?;
    pos = expect_symbol(tokens, pos, "(")?;

    let mut columns = Vec::new();
    loop {
        pos = skip_whitespace(tokens, pos);
        if pos >= tokens.len() {
            return Err(parse_error(
                "unexpected end inside CREATE TABLE column list",
                pos,
            ));
        }
        if let TokenType::Symbol = tokens[pos].token_type {
            if tokens[pos].value == ")" {
                break;
            }
            if tokens[pos].value == "," {
                pos += 1;
                continue;
            }
        }
        let (col_name, next) = expect_identifier(tokens, pos)?;
        pos = next;
        pos = skip_whitespace(tokens, pos);
        if pos >= tokens.len() {
            return Err(parse_error("expected column type", pos));
        }
        let col_type = tokens[pos].value.to_uppercase();
        pos += 1;
        columns.push(ColumnDef {
            name: col_name,
            col_type,
        });
    }

    Ok(Statement::CreateTable { table, columns })
}

fn parse_insert(tokens: &[Token], mut pos: usize) -> Result<Statement, String> {
    pos = expect_keyword(tokens, pos, "INTO")?;
    let (table, mut pos) = expect_identifier(tokens, pos)?;
    pos = expect_keyword(tokens, pos, "VALUES")?;
    pos = expect_symbol(tokens, pos, "(")?;

    let mut values = Vec::new();
    loop {
        pos = skip_whitespace(tokens, pos);
        if pos >= tokens.len() {
            return Err(parse_error("unexpected end inside VALUES list", pos));
        }
        if let TokenType::Symbol = tokens[pos].token_type {
            if tokens[pos].value == ")" {
                break;
            }
            if tokens[pos].value == "," {
                pos += 1;
                continue;
            }
        }
        let (expr, next) = parse_primary(tokens, pos)?;
        pos = next;
        values.push(expr);
    }

    Ok(Statement::Insert { table, values })
}

fn parse_select(tokens: &[Token], mut pos: usize) -> Result<Statement, String> {
    // parse column list: * or col1, col2, ...
    let mut columns = Vec::new();
    pos = skip_whitespace(tokens, pos);

    if pos < tokens.len() {
        if let TokenType::Symbol = tokens[pos].token_type {
            if tokens[pos].value == "*" {
                columns.push("*".to_string());
                pos += 1;
            }
        }
    }

    if columns.is_empty() {
        loop {
            pos = skip_whitespace(tokens, pos);
            if pos >= tokens.len() {
                break;
            }
            if let TokenType::Symbol = tokens[pos].token_type {
                if tokens[pos].value == "," {
                    pos += 1;
                    continue;
                }
            }
            if let TokenType::Keyword = tokens[pos].token_type {
                break;
            }
            let (col, next) = expect_identifier(tokens, pos)?;
            columns.push(col);
            pos = next;
        }
    }

    pos = expect_keyword(tokens, pos, "FROM")?;
    let (table, mut pos) = expect_identifier(tokens, pos)?;

    pos = skip_whitespace(tokens, pos);
    let where_clause = if pos < tokens.len() {
        if let TokenType::Keyword = tokens[pos].token_type {
            if tokens[pos].value.to_uppercase() == "WHERE" {
                pos += 1;
                let (expr, next) = parse_expr(tokens, pos)?;
                pos = next;
                Some(expr)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let _ = pos; // suppress unused warning after optional clause

    Ok(Statement::Select {
        columns,
        table,
        where_clause,
    })
}

pub fn parse(input: &str) -> Result<Statement, String> {
    let trimmed = input.trim();
    match trimmed.to_lowercase().as_str() {
        ".exit" | "exit" => return Ok(Statement::Exit),
        ".help" | "help" => return Ok(Statement::Help),
        _ => {}
    }

    let mut tokenizer = Tokenizer::new(trimmed.to_string());
    let tokens = tokenizer.tokenize();

    let pos = skip_whitespace(&tokens, 0);
    if pos >= tokens.len() {
        return Err(parse_error("empty input", 0));
    }

    let first = &tokens[pos];
    match first.token_type {
        TokenType::Keyword => {
            let kw = first.value.to_uppercase();
            let next_pos = pos + 1;
            match kw.as_str() {
                "SELECT" => parse_select(&tokens, next_pos),
                "INSERT" => parse_insert(&tokens, next_pos),
                "CREATE" => parse_create_table(&tokens, next_pos),
                "EXIT" => Ok(Statement::Exit),
                "HELP" => Ok(Statement::Help),
                _ => Err(parse_error(
                    &format!("unknown keyword '{}'", first.value),
                    pos,
                )),
            }
        }
        _ => Err(parse_error(
            &format!("expected SQL statement, got '{}'", first.value),
            pos,
        )),
    }
}
