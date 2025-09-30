use crate::error::{Result, QueryError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SqlStatement {
    Select {
        columns: Vec<String>,
        table: String,
        where_clause: Option<String>,
    },
    Insert {
        table: String,
        columns: Vec<String>,
        values: Vec<Vec<String>>,
    },
    Update {
        table: String,
        set_clause: Vec<(String, String)>,
        where_clause: Option<String>,
    },
    Delete {
        table: String,
        where_clause: Option<String>,
    },
}

/// Simplified SQL parser
pub struct SqlParser;

impl SqlParser {
    pub fn parse(sql: &str) -> Result<SqlStatement> {
        let sql = sql.trim().to_lowercase();
        
        if sql.starts_with("select") {
            Self::parse_select(&sql)
        } else if sql.starts_with("insert") {
            Self::parse_insert(&sql)
        } else if sql.starts_with("update") {
            Self::parse_update(&sql)
        } else if sql.starts_with("delete") {
            Self::parse_delete(&sql)
        } else {
            Err(QueryError::Parse(format!("Unsupported statement: {}", sql)))
        }
    }
    
    fn parse_select(sql: &str) -> Result<SqlStatement> {
        // Very simplified SELECT parsing
        // Real implementation would use a proper SQL parser
        
        if let Some(from_pos) = sql.find(" from ") {
            let select_part = &sql[6..from_pos].trim(); // Skip "select"
            let after_from = &sql[from_pos + 6..].trim();
            
            let columns: Vec<String> = if select_part == &"*" {
                vec!["*".to_string()]
            } else {
                select_part.split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            };
            
            let (table, where_clause) = if let Some(where_pos) = after_from.find(" where ") {
                let table = after_from[..where_pos].trim().to_string();
                let where_part = after_from[where_pos + 7..].trim().to_string();
                (table, Some(where_part))
            } else {
                (after_from.to_string(), None)
            };
            
            Ok(SqlStatement::Select {
                columns,
                table,
                where_clause,
            })
        } else {
            Err(QueryError::Parse("Invalid SELECT statement".to_string()))
        }
    }
    
    fn parse_insert(_sql: &str) -> Result<SqlStatement> {
        // Simplified INSERT parsing
        Err(QueryError::Parse("INSERT not implemented yet".to_string()))
    }
    
    fn parse_update(_sql: &str) -> Result<SqlStatement> {
        // Simplified UPDATE parsing
        Err(QueryError::Parse("UPDATE not implemented yet".to_string()))
    }
    
    fn parse_delete(_sql: &str) -> Result<SqlStatement> {
        // Simplified DELETE parsing
        Err(QueryError::Parse("DELETE not implemented yet".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_select() {
        let sql = "SELECT * FROM users";
        let result = SqlParser::parse(sql).unwrap();
        
        match result {
            SqlStatement::Select { columns, table, where_clause } => {
                assert_eq!(columns, vec!["*".to_string()]);
                assert_eq!(table, "users");
                assert_eq!(where_clause, None);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }
    
    #[test]
    fn test_parse_select_with_columns() {
        let sql = "SELECT id, name FROM users";
        let result = SqlParser::parse(sql).unwrap();
        
        match result {
            SqlStatement::Select { columns, table, where_clause } => {
                assert_eq!(columns, vec!["id".to_string(), "name".to_string()]);
                assert_eq!(table, "users");
                assert_eq!(where_clause, None);
            }
            _ => panic!("Expected SELECT statement"),
        }
    }
    
    #[test]
    fn test_parse_select_with_where() {
        let sql = "SELECT * FROM users WHERE id = 1";
        let result = SqlParser::parse(sql).unwrap();
        
        match result {
            SqlStatement::Select { columns, table, where_clause } => {
                assert_eq!(columns, vec!["*".to_string()]);
                assert_eq!(table, "users");
                assert_eq!(where_clause, Some("id = 1".to_string()));
            }
            _ => panic!("Expected SELECT statement"),
        }
    }
}