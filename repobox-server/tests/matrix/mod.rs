//! Test matrix framework for repo.box policy testing
//! 
//! This module implements the server-first test matrix architecture described in
//! docs/spec/server-first-test-matrix.md. The framework is designed to:
//! 
//! - Ensure server integration tests are the canonical source of truth for security behavior
//! - Provide systematic coverage of policy dimensions through matrix-driven testing
//! - Make test coverage auditable and extensible when adding new policy dimensions
//! 
//! Key components:
//! - `dimensions`: Registry of available test dimensions
//! - `scenario`: Test scenario declarations and builders
//! - `coverage`: Coverage contracts and auditing
//! - `fixtures`: Test data builders for creating test repositories
//! - `runners`: Test execution engines for server and shim testing

pub mod dimensions;
pub mod scenario;
pub mod coverage;
pub mod fixtures;
pub mod runners;

pub use dimensions::*;
pub use scenario::*;
pub use coverage::*;
pub use fixtures::*;
pub use runners::*;