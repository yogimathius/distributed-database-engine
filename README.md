# NextDB - Distributed Database Engine

NextDB is a next-generation distributed database engine with LSM-tree storage, Raft consensus, and multi-model support.

## Features

- **LSM-Tree Storage Engine**: High-performance storage with WAL and compaction
- **Raft Consensus**: Distributed coordination for high availability  
- **Multi-Model Support**: Relational, document, graph, and time-series data
- **SQL Query Engine**: PostgreSQL-compatible SQL with distributed execution
- **ACID Transactions**: Full ACID compliance with MVCC isolation
- **High Performance**: Target 100K+ reads/sec with P99 < 5ms latency

## Architecture

NextDB is built as a Rust workspace with the following components:

- `nextdb-storage`: LSM-tree storage engine with compression and caching
- `nextdb-consensus`: Raft consensus implementation for distributed coordination
- `nextdb-query`: SQL parser, planner, and execution engine
- `nextdb-transaction`: MVCC transaction management system
- `nextdb-server`: Database server coordinating all components
- `nextdb-client`: Client library with connection pooling

## Installation

```bash
git clone https://github.com/nextdb/nextdb
cd nextdb
cargo build --release
```

## Usage

### Start Database Server

```bash
# Start server on default port 8080
./target/release/nextdb server

# Start server on custom port
./target/release/nextdb server 5432
```

### Interactive Client

```bash
# Connect to local server
./target/release/nextdb client

# Connect to remote server
./target/release/nextdb client hostname:port
```

### Performance Benchmark

```bash
# Run performance benchmark suite
RUST_LOG=info ./target/release/nextdb benchmark
```

## Development Status

**Current Status: 10% Complete** - Comprehensive architecture with basic CLI

### Completed Features
- ✅ Project structure and workspace setup
- ✅ Basic CLI with server/client/benchmark modes
- ✅ Core architecture with all major components
- ✅ Build system and dependency management
- ✅ Testing framework setup

### In Development (90% remaining)
- ⚠️ LSM-tree storage implementation
- ⚠️ Raft consensus algorithm
- ⚠️ SQL query engine
- ⚠️ Transaction processing
- ⚠️ Network protocol and client/server communication
- ⚠️ Production monitoring and operations

## Performance Targets

- **Throughput**: 100,000+ reads/second per node
- **Latency**: P99 < 5ms for point queries  
- **Scalability**: Linear scaling to 100+ nodes
- **Availability**: 99.99% uptime with automated failover
- **Consistency**: Strong consistency with configurable options

## License

MIT License - see LICENSE file for details

## Contributing

NextDB is in early development. Contributions welcome for:
- Storage engine optimization
- Consensus algorithm implementation  
- Query planner and optimizer
- Client driver development
- Performance testing and benchmarks

For detailed development roadmap, see [MVP_REQUIREMENTS.md](MVP_REQUIREMENTS.md)