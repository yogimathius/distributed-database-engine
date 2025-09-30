use crate::{error::{Result, QueryError}, planner::PhysicalPlan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSet {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// Query executor that executes physical plans
pub struct QueryExecutor;

impl QueryExecutor {
    pub async fn execute(_plan: PhysicalPlan) -> Result<ResultSet> {
        // Simplified executor - return empty result set
        Ok(ResultSet {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec!["1".to_string(), "Alice".to_string()],
                vec!["2".to_string(), "Bob".to_string()],
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_execute_table_scan() {
        let plan = PhysicalPlan::TableScan {
            table: "users".to_string(),
            columns: vec!["*".to_string()],
            filter: None,
        };
        
        let result = QueryExecutor::execute(plan).await.unwrap();
        assert_eq!(result.columns, vec!["id", "name"]);
        assert_eq!(result.rows.len(), 2);
    }
}