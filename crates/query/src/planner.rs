use crate::{error::{Result, QueryError}, parser::SqlStatement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicalPlan {
    TableScan {
        table: String,
        columns: Vec<String>,
        filter: Option<String>,
    },
    IndexScan {
        table: String,
        index: String,
        columns: Vec<String>,
        filter: Option<String>,
    },
}

/// Query planner that converts SQL statements to execution plans
pub struct QueryPlanner;

impl QueryPlanner {
    pub fn plan(statement: SqlStatement) -> Result<PhysicalPlan> {
        match statement {
            SqlStatement::Select { columns, table, where_clause } => {
                // Simplified planning - just use table scan
                Ok(PhysicalPlan::TableScan {
                    table,
                    columns,
                    filter: where_clause,
                })
            }
            _ => Err(QueryError::Plan("Only SELECT supported".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plan_simple_select() {
        let statement = SqlStatement::Select {
            columns: vec!["*".to_string()],
            table: "users".to_string(),
            where_clause: None,
        };
        
        let plan = QueryPlanner::plan(statement).unwrap();
        
        match plan {
            PhysicalPlan::TableScan { table, columns, filter } => {
                assert_eq!(table, "users");
                assert_eq!(columns, vec!["*".to_string()]);
                assert_eq!(filter, None);
            }
            _ => panic!("Expected TableScan plan"),
        }
    }
}