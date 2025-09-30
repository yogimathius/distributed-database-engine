use crate::{ServerConfig, Result};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::SystemTime};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::info;

#[derive(Clone)]
pub struct DatabaseServer {
    config: ServerConfig,
    state: Arc<DatabaseState>,
}

#[derive(Debug)]
struct DatabaseState {
    start_time: SystemTime,
    port: u16,
    storage_stats: tokio::sync::RwLock<StorageStats>,
    consensus_stats: tokio::sync::RwLock<ConsensusStats>,
    query_stats: tokio::sync::RwLock<QueryStats>,
}

#[derive(Debug, Clone, Serialize)]
struct StorageStats {
    memtable_size: u64,
    sstable_count: u32,
    cache_hit_rate: f64,
    total_keys: u64,
    total_size_bytes: u64,
    compaction_count: u32,
}

#[derive(Debug, Clone, Serialize)]
struct ConsensusStats {
    node_id: String,
    role: String, // "leader", "follower", "candidate"
    current_term: u64,
    commit_index: u64,
    cluster_size: u32,
    healthy_nodes: u32,
}

#[derive(Debug, Clone, Serialize)]
struct QueryStats {
    total_queries: u64,
    queries_per_second: f64,
    avg_latency_ms: f64,
    p99_latency_ms: f64,
    cache_hit_rate: f64,
}

#[derive(Serialize)]
struct SystemStatus {
    status: String,
    uptime_seconds: u64,
    version: String,
    storage: StorageStats,
    consensus: ConsensusStats,
    query: QueryStats,
}

#[derive(Deserialize)]
struct QueryRequest {
    sql: String,
}

#[derive(Serialize)]
struct QueryResponse {
    success: bool,
    rows_affected: Option<u64>,
    execution_time_ms: f64,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

impl DatabaseServer {
    pub async fn new(port: u16) -> Result<Self> {
        let config = ServerConfig::new(port);
        let state = Arc::new(DatabaseState {
            start_time: SystemTime::now(),
            port,
            storage_stats: tokio::sync::RwLock::new(StorageStats::default()),
            consensus_stats: tokio::sync::RwLock::new(ConsensusStats::default()),
            query_stats: tokio::sync::RwLock::new(QueryStats::default()),
        });

        Ok(Self { config, state })
    }

    pub async fn start(self) -> Result<()> {
        info!("🔥 NextDB Server starting on port {}", self.config.port);

        // Create static file directory if it doesn't exist
        tokio::fs::create_dir_all("web/static").await?;
        
        // Create the main HTML page
        self.create_web_interface().await?;

        let app = Router::new()
            .route("/", get(serve_dashboard))
            .route("/api/status", get(get_status))
            .route("/api/query", post(execute_query))
            .route("/api/storage/stats", get(get_storage_stats))
            .route("/api/consensus/stats", get(get_consensus_stats))
            .route("/api/query/stats", get(get_query_stats))
            .nest_service("/static", ServeDir::new("web/static"))
            .layer(CorsLayer::permissive())
            .with_state(self.state.clone());

        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.config.port)).await?;
        
        info!("🚀 NextDB Server running on http://localhost:{}", self.config.port);
        info!("📊 Dashboard available at http://localhost:{}/", self.config.port);
        info!("📡 API available at http://localhost:{}/api", self.config.port);

        // Start background tasks for simulation
        self.start_simulation_tasks();

        axum::serve(listener, app).await?;

        Ok(())
    }

    async fn create_web_interface(&self) -> Result<()> {
        let html_content = include_str!("../../../web/dashboard.html");
        tokio::fs::write("web/static/index.html", html_content).await?;
        info!("📄 Web dashboard created at web/static/index.html");
        Ok(())
    }

    fn start_simulation_tasks(&self) {
        let state = self.state.clone();
        
        // Simulate storage operations
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                let mut stats = state.storage_stats.write().await;
                stats.total_keys += 150;
                stats.total_size_bytes += 4096 * 150;
                stats.memtable_size = (stats.memtable_size + 2048) % 64000000;
                stats.cache_hit_rate = 0.85 + (rand::random_f64() * 0.1);
                if rand::random_f64() > 0.8 {
                    stats.compaction_count += 1;
                    stats.sstable_count += 1;
                }
            }
        });

        let state = self.state.clone();
        // Simulate consensus operations
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
            loop {
                interval.tick().await;
                let mut stats = state.consensus_stats.write().await;
                stats.current_term += if rand::random_f64() > 0.95 { 1 } else { 0 };
                stats.commit_index += 1;
            }
        });

        let state = self.state.clone();
        // Simulate query operations
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let mut stats = state.query_stats.write().await;
                let new_queries = 50 + (rand::random::<u64>() % 100);
                stats.total_queries += new_queries;
                stats.queries_per_second = new_queries as f64;
                stats.avg_latency_ms = 2.5 + (rand::random_f64() * 2.0);
                stats.p99_latency_ms = 8.0 + (rand::random_f64() * 4.0);
                stats.cache_hit_rate = 0.75 + (rand::random_f64() * 0.2);
            }
        });
    }
}

async fn serve_dashboard() -> Html<&'static str> {
    Html(include_str!("../../../web/dashboard.html"))
}

async fn get_status(State(state): State<Arc<DatabaseState>>) -> Json<SystemStatus> {
    let uptime = state.start_time.elapsed().unwrap_or_default().as_secs();
    let storage = state.storage_stats.read().await.clone();
    let consensus = state.consensus_stats.read().await.clone();
    let query = state.query_stats.read().await.clone();

    Json(SystemStatus {
        status: "operational".to_string(),
        uptime_seconds: uptime,
        version: "0.1.0".to_string(),
        storage,
        consensus,
        query,
    })
}

async fn execute_query(
    State(_state): State<Arc<DatabaseState>>,
    Json(req): Json<QueryRequest>,
) -> std::result::Result<Json<QueryResponse>, StatusCode> {
    info!("Executing SQL query: {}", req.sql);
    
    // Simulate query execution
    let execution_time = 1.5 + (rand::random_f64() * 3.0);
    tokio::time::sleep(std::time::Duration::from_millis(execution_time as u64)).await;

    let response = if req.sql.to_lowercase().contains("select") {
        QueryResponse {
            success: true,
            rows_affected: None,
            execution_time_ms: execution_time,
            result: Some(serde_json::json!([
                {"id": 1, "name": "Alice", "age": 30},
                {"id": 2, "name": "Bob", "age": 25}
            ])),
            error: None,
        }
    } else if req.sql.to_lowercase().contains("insert") {
        QueryResponse {
            success: true,
            rows_affected: Some(1),
            execution_time_ms: execution_time,
            result: None,
            error: None,
        }
    } else {
        QueryResponse {
            success: false,
            rows_affected: None,
            execution_time_ms: execution_time,
            result: None,
            error: Some("Query not supported in simulation mode".to_string()),
        }
    };

    Ok(Json(response))
}

async fn get_storage_stats(State(state): State<Arc<DatabaseState>>) -> Json<StorageStats> {
    Json(state.storage_stats.read().await.clone())
}

async fn get_consensus_stats(State(state): State<Arc<DatabaseState>>) -> Json<ConsensusStats> {
    Json(state.consensus_stats.read().await.clone())
}

async fn get_query_stats(State(state): State<Arc<DatabaseState>>) -> Json<QueryStats> {
    Json(state.query_stats.read().await.clone())
}

impl Default for StorageStats {
    fn default() -> Self {
        Self {
            memtable_size: 1024 * 1024,
            sstable_count: 5,
            cache_hit_rate: 0.85,
            total_keys: 10000,
            total_size_bytes: 100 * 1024 * 1024,
            compaction_count: 0,
        }
    }
}

impl Default for ConsensusStats {
    fn default() -> Self {
        Self {
            node_id: "node-001".to_string(),
            role: "leader".to_string(),
            current_term: 1,
            commit_index: 0,
            cluster_size: 3,
            healthy_nodes: 3,
        }
    }
}

impl Default for QueryStats {
    fn default() -> Self {
        Self {
            total_queries: 0,
            queries_per_second: 0.0,
            avg_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            cache_hit_rate: 0.0,
        }
    }
}

// Simple random number generation for simulation
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn random<T>() -> T
    where
        T: From<u64>,
    {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        T::from(hasher.finish())
    }

    pub fn random_f64() -> f64 {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        (hasher.finish() as f64) / (u64::MAX as f64)
    }
}