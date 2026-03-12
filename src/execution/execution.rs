use std::collections::HashMap;

use crate::parser::{Expression, Statement};
use crate::row::Row;
use crate::storage::table::Table;
use crate::schema::{Schema, MAX_COL};

pub enum ExecuteResult {
    Created(String),
    Inserted,
    Rows {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Message(String),
}

// All open tables live here for the lifetime of the session.
pub struct Engine {
    tables: HashMap<String, Table>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn execute(&mut self, stmt: Statement) -> Result<ExecuteResult, String> {
        match stmt {
            Statement::CreateTable { table, columns } => self.exec_create(table, columns),
            Statement::Insert { table, values } => self.exec_insert(table, values),
            Statement::Select {
                columns,
                table,
                where_clause,
            } => self.exec_select(table, columns, where_clause),
            Statement::Exit => Ok(ExecuteResult::Message("bye.".into())),
            Statement::Help => Ok(ExecuteResult::Message(
                "commands: CREATE TABLE, INSERT INTO, SELECT, .exit".into(),
            )),
        }
    }

    fn exec_create(
        &mut self,
        table_name: String,
        columns: Vec<crate::parser::ColumnDef>,
    ) -> Result<ExecuteResult, String> {
        if self.tables.contains_key(&table_name) {
            return Err(format!("table \"{}\" already exists", table_name));
        }
        if columns.is_empty() {
            return Err("CREATE TABLE requires at least one column".into());
        }
        // id is implicit; user columns map to the MAX_COL slots
        let user_cols: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
        if user_cols.len() > MAX_COL {
            return Err(format!("too many columns (max {})", MAX_COL));
        }
        let schema = Schema::new(&user_cols);
        let path = format!("{}.db", table_name);
        let t = Table::open(&path, schema).map_err(|e| e.to_string())?;
        self.tables.insert(table_name.clone(), t);
        Ok(ExecuteResult::Created(table_name))
    }

    fn exec_insert(
        &mut self,
        table_name: String,
        values: Vec<Expression>,
    ) -> Result<ExecuteResult, String> {
        let table = self
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| format!("table \"{}\" does not exist", table_name))?;

        // values must be: id + one per schema column
        let expected = 1 + table.schema.num_cols;
        if values.len() != expected {
            return Err(format!(
                "expected {} value(s) (id + {} column(s)), got {}",
                expected,
                table.schema.num_cols,
                values.len()
            ));
        }

        let id: i64 = literal_str(&values[0])
            .parse()
            .map_err(|_| format!("id must be an integer, got \"{}\"", literal_str(&values[0])))?;

        let col_vals: Vec<&str> = values[1..].iter().map(|e| literal_str(e)).collect();
        let row = Row::new(id, &col_vals);
        table.insert(&row).map_err(|e| e.to_string())?;
        Ok(ExecuteResult::Inserted)
    }

    fn exec_select(
        &mut self,
        table_name: String,
        col_names: Vec<String>,
        where_clause: Option<Expression>,
    ) -> Result<ExecuteResult, String> {
        let table = self
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| format!("table \"{}\" does not exist", table_name))?;

        let all_rows = table.scan().map_err(|e| e.to_string())?;
        let schema = table.schema.clone();

        // Resolve which column indices to project
        let project: Vec<usize> = if col_names == ["*"] {
            (0..schema.num_cols).collect()
        } else {
            col_names
                .iter()
                .map(|name| {
                    (0..schema.num_cols)
                        .find(|&i| schema.cols[i].name_str() == name.as_str())
                        .ok_or_else(|| format!("unknown column \"{}\"", name))
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        let headers: Vec<String> = std::iter::once("id".to_string())
            .chain(
                project
                    .iter()
                    .map(|&i| schema.cols[i].name_str().to_string()),
            )
            .collect();

        let mut result_rows: Vec<Vec<String>> = Vec::new();
        for row in all_rows {
            if let Some(expr) = &where_clause {
                if !eval_where(expr, &row, &schema)? {
                    continue;
                }
            }
            let mut record = vec![row.id.to_string()];
            record.extend(project.iter().map(|&i| row.values[i].clone()));
            result_rows.push(record);
        }

        Ok(ExecuteResult::Rows {
            headers,
            rows: result_rows,
        })
    }
}

// ── WHERE evaluation ────────────────────────────────────────────────────────

fn eval_where(expr: &Expression, row: &Row, schema: &Schema) -> Result<bool, String> {
    match expr {
        Expression::BinaryOp { op, left, right } => {
            let lv = resolve_val(left, row, schema)?;
            let rv = resolve_val(right, row, schema)?;
            Ok(match op.as_str() {
                "=" => lv == rv,
                "!=" => lv != rv,
                "<" => cmp_vals(&lv, &rv) == std::cmp::Ordering::Less,
                ">" => cmp_vals(&lv, &rv) == std::cmp::Ordering::Greater,
                "<=" => cmp_vals(&lv, &rv) != std::cmp::Ordering::Greater,
                ">=" => cmp_vals(&lv, &rv) != std::cmp::Ordering::Less,
                _ => return Err(format!("unknown operator \"{}\"", op)),
            })
        }
        Expression::And(l, r) => Ok(eval_where(l, row, schema)? && eval_where(r, row, schema)?),
        Expression::Or(l, r) => Ok(eval_where(l, row, schema)? || eval_where(r, row, schema)?),
        _ => Err("WHERE clause must be a comparison".into()),
    }
}

fn resolve_val(expr: &Expression, row: &Row, schema: &Schema) -> Result<String, String> {
    match expr {
        Expression::Literal(s) => Ok(s.clone()),
        Expression::ColumnRef(name) => {
            if name == "id" {
                return Ok(row.id.to_string());
            }
            let idx = (0..schema.num_cols)
                .find(|&i| schema.cols[i].name_str() == name.as_str())
                .ok_or_else(|| format!("unknown column \"{}\"", name))?;
            Ok(row.values[idx].clone())
        }
        _ => Err("unsupported expression in WHERE".into()),
    }
}

// Numeric-aware comparison: try i64 first, fall back to string order.
fn cmp_vals(a: &str, b: &str) -> std::cmp::Ordering {
    match (a.parse::<i64>(), b.parse::<i64>()) {
        (Ok(x), Ok(y)) => x.cmp(&y),
        _ => a.cmp(b),
    }
}

fn literal_str(expr: &Expression) -> &str {
    match expr {
        Expression::Literal(s) => s.as_str(),
        Expression::ColumnRef(s) => s.as_str(),
        _ => "",
    }
}