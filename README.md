# NextDB - Distributed Database Engine

Next-generation distributed database engine with LSM-tree storage and Raft consensus

## Purpose
- Next-generation distributed database engine with LSM-tree storage and Raft consensus
- Last structured review: `2026-02-08`

## Current Implementation
- Detected major components: `src/`, `crates/`, `web/`
- No clear API/controller routing signals were detected at this scope
- Cargo metadata is present for Rust components

## Interfaces
- No explicit HTTP endpoint definitions were detected at the project root scope

## Testing and Verification
- `cargo test` appears applicable for Rust components
- Tests are listed here as available commands; rerun before release to confirm current behavior.

## Current Status
- Estimated operational coverage: **41%**
- Confidence level: **medium**

## Next Steps
- Document and stabilize the external interface (CLI, API, or protocol) with explicit examples
- Run the detected tests in CI and track flakiness, duration, and coverage
- Validate runtime claims in this README against current behavior and deployment configuration

## Source of Truth
- This README is intended to be the canonical project summary for portfolio alignment.
- If portfolio copy diverges from this file, update the portfolio entry to match current implementation reality.
