//! Unified search functionality for all domains
//!
//! Provides dynamic SQL building for flexible searches with filters

use sqlx::{SqlitePool, Row, Column};
use serde_json::{json, Value};
use crate::error::{AppError, AppResult};

/// Search filter condition
#[derive(Debug, Clone)]
pub struct SearchFilter {
    pub column: String,
    pub operator: SearchOperator,
    pub value: String,
}

/// Supported search operators
#[derive(Debug, Clone)]
pub enum SearchOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    In(Vec<String>),
}

/// Search request parameters
#[derive(Debug, Default)]
pub struct SearchRequest {
    pub domain: String,
    pub filters: Vec<SearchFilter>,
    pub order_by: Option<String>,
    pub order_direction: String, // ASC or DESC
    pub limit: i64,
    pub offset: Option<i64>,
}

/// Execute a search query
pub async fn search_domain(
    pool: &SqlitePool,
    request: &SearchRequest,
) -> AppResult<(Value, String)> {
    let (table, columns) = get_table_info(&request.domain)?;
    
    // Build dynamic SQL
    let mut sql = format!("SELECT {} FROM {}", columns, table);
    let mut binds: Vec<String> = Vec::new();
    
    // Add WHERE clauses
    if !request.filters.is_empty() {
        sql.push_str(" WHERE ");
        let mut conditions = Vec::new();
        
        for (i, filter) in request.filters.iter().enumerate() {
            let param_name = format!("param_{}", i);
            let condition = match filter.operator {
                SearchOperator::Eq => format!("{} = ?", filter.column),
                SearchOperator::Ne => format!("{} != ?", filter.column),
                SearchOperator::Gt => format!("{} > ?", filter.column),
                SearchOperator::Gte => format!("{} >= ?", filter.column),
                SearchOperator::Lt => format!("{} < ?", filter.column),
                SearchOperator::Lte => format!("{} <= ?", filter.column),
                SearchOperator::Like => format!("{} LIKE ?", filter.column),
                SearchOperator::In(ref values) => {
                    let placeholders = (0..values.len())
                        .map(|_| "?")
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{} IN ({})", filter.column, placeholders)
                }
            };
            conditions.push(condition);
            binds.push(filter.value.clone());
        }
        
        sql.push_str(&conditions.join(" AND "));
    }
    
    // Add ORDER BY
    if let Some(ref order) = request.order_by {
        sql.push_str(&format!(" ORDER BY {} {}", order, request.order_direction));
    }
    
    // Add LIMIT
    sql.push_str(&format!(" LIMIT {}", request.limit));
    
    // Add OFFSET
    if let Some(offset) = request.offset {
        sql.push_str(&format!(" OFFSET {}", offset));
    }
    
    // Execute query
    let mut query = sqlx::query(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }
    
    // Handle IN operator binds
    for filter in &request.filters {
        if let SearchOperator::In(ref values) = filter.operator {
            for value in values {
                query = query.bind(value);
            }
        }
    }
    
    let rows = query.fetch_all(pool).await?;
    
    // Convert rows to JSON
    let results: Vec<Value> = rows.iter().map(|row| {
        let mut obj = serde_json::Map::new();
        // Get column names and values
        for (i, column) in row.columns().iter().enumerate() {
            if let Ok(value) = row.try_get::<String, _>(i) {
                obj.insert(column.name().to_string(), json!(value));
            } else if let Ok(value) = row.try_get::<f64, _>(i) {
                obj.insert(column.name().to_string(), json!(value));
            } else if let Ok(value) = row.try_get::<i64, _>(i) {
                obj.insert(column.name().to_string(), json!(value));
            } else if let Ok(value) = row.try_get::<Option<String>, _>(i) {
                obj.insert(column.name().to_string(), json!(value));
            }
        }
        json!(obj)
    }).collect();
    
    Ok((
        json!(results),
        format!("{} result(s) found", results.len())
    ))
}

/// Get table name and columns for a domain
fn get_table_info(domain: &str) -> AppResult<(&'static str, &'static str)> {
    match domain {
        "transaction" | "transactions" => Ok(("transactions", "id, amount, kind, description, timestamp, category_id, place_id")),
        "activity" | "activities" => Ok(("activities", "id, start_time, stop_time, description, category_id, place_id")),
        "todo" | "todos" => Ok(("todos", "id, description, status, priority, due_date, created_at, completed_at, category_id, place_id")),
        "journal" | "journals" => Ok(("journal_entries", "id, content, date, category_id, place_id, created_at")),
        "category" | "categories" => Ok(("categories", "id, name")),
        "place" | "places" => Ok(("places", "id, name")),
        "tag" | "tags" => Ok(("tags", "id, name")),
        "person" | "persons" => Ok(("persons", "id, name")),
        _ => Err(AppError::ValidationError(format!("Unknown domain: {}", domain))),
    }
}

/// Parse search command arguments into SearchRequest
pub fn parse_search_args(domain: &str, args: &[&str]) -> AppResult<SearchRequest> {
    let mut request = SearchRequest {
        domain: domain.to_string(),
        limit: 100, // Default limit
        order_direction: "ASC".to_string(),
        ..Default::default()
    };
    
    let mut i = 0;
    while i < args.len() {
        if !args[i].starts_with("--") {
            i += 1;
            continue;
        }
        
        let key = args[i].trim_start_matches("--");
        let value = if i + 1 < args.len() && !args[i + 1].starts_with("--") {
            i += 1;
            args[i]
        } else {
            "true"
        };
        
        match key {
            // Filter shortcuts (e.g., --kind shopping, --status pending)
            "where" => {
                // Parse complex WHERE clause: "kind=shopping AND amount>50"
                request.filters.extend(parse_where_clause(value)?);
            }
            "order-by" | "order_by" => {
                request.order_by = Some(value.to_string());
            }
            "order" => {
                request.order_direction = if value.to_uppercase() == "DESC" {
                    "DESC".to_string()
                } else {
                    "ASC".to_string()
                };
            }
            "limit" => {
                request.limit = value.parse().unwrap_or(100);
            }
            "offset" => {
                request.offset = Some(value.parse().unwrap_or(0));
            }
            // Dynamic filter parsing
            _ => {
                // Convert key to column name (e.g., min-amount -> amount, kind -> kind)
                let column = normalize_column_name(key);
                if is_valid_column(&request.domain, &column) {
                    request.filters.push(SearchFilter {
                        column,
                        operator: SearchOperator::Eq,
                        value: value.to_string(),
                    });
                }
            }
        }
        i += 1;
    }
    
    Ok(request)
}

/// Parse WHERE clause string into filters
fn parse_where_clause(clause: &str) -> AppResult<Vec<SearchFilter>> {
    let mut filters = Vec::new();
    
    // Split by AND
    let conditions = clause.split(" AND ");
    
    for condition in conditions {
        let condition = condition.trim();
        
        // Try different operators
        let (column, operator, value) = if let Some(idx) = condition.find("!=") {
            (&condition[..idx], SearchOperator::Ne, &condition[idx+2..])
        } else if let Some(idx) = condition.find(">=") {
            (&condition[..idx], SearchOperator::Gte, &condition[idx+2..])
        } else if let Some(idx) = condition.find("<=") {
            (&condition[..idx], SearchOperator::Lte, &condition[idx+2..])
        } else if let Some(idx) = condition.find(">") {
            (&condition[..idx], SearchOperator::Gt, &condition[idx+1..])
        } else if let Some(idx) = condition.find("<") {
            (&condition[..idx], SearchOperator::Lt, &condition[idx+1..])
        } else if let Some(idx) = condition.find("=") {
            (&condition[..idx], SearchOperator::Eq, &condition[idx+1..])
        } else {
            return Err(AppError::ValidationError(
                format!("Invalid condition: {}", condition)
            ));
        };
        
        filters.push(SearchFilter {
            column: column.trim().to_string(),
            operator,
            value: value.trim().trim_matches('\'').to_string(),
        });
    }
    
    Ok(filters)
}

/// Normalize column name from CLI argument
fn normalize_column_name(key: &str) -> String {
    key.replace("-", "_")
        .replace("min_", "")
        .replace("max_", "")
        .replace("start_", "")
        .replace("end_", "")
}

/// Check if column is valid for the domain
fn is_valid_column(domain: &str, column: &str) -> bool {
    let valid_columns: &[&str] = match domain {
        "transaction" | "transactions" => &["kind", "amount", "description", "category", "place", "category_id", "place_id"],
        "activity" | "activities" => &["description", "category", "place", "start_time", "stop_time", "category_id", "place_id"],
        "todo" | "todos" => &["status", "priority", "description", "category", "place", "due_date", "category_id", "place_id"],
        "journal" | "journals" => &["content", "date", "category", "place", "category_id", "place_id"],
        "category" | "categories" => &["name"],
        "place" | "places" => &["name"],
        "tag" | "tags" => &["name"],
        "person" | "persons" => &["name"],
        _ => return false,
    };
    
    valid_columns.contains(&column)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_where_clause_simple() {
        let filters = parse_where_clause("kind=shopping").unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].column, "kind");
        assert_eq!(filters[0].value, "shopping");
    }
    
    #[test]
    fn test_parse_where_clause_multiple() {
        let filters = parse_where_clause("kind=shopping AND amount>50").unwrap();
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].column, "kind");
        assert_eq!(filters[1].column, "amount");
        assert_eq!(matches!(filters[1].operator, SearchOperator::Gt), true);
    }
    
    #[test]
    fn test_normalize_column_name() {
        assert_eq!(normalize_column_name("min-amount"), "amount");
        assert_eq!(normalize_column_name("start-time"), "time");
        assert_eq!(normalize_column_name("kind"), "kind");
    }
}
