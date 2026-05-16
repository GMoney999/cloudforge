// src/lib.rs

//! # linear-mcp-client
//!
//! A trait-based, async Rust client for Linear's hosted MCP server.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use linear_mcp_client::builder::LinearClientBuilder;
//! use linear_mcp_client::linear::LinearClient;
//! use linear_mcp_client::types::CreateIssueInput;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = LinearClientBuilder::new()
//!         .bearer_token_from_env("LINEAR_API_KEY")?
//!         .build()?;
//!
//!     let teams = client.list_teams().await?;
//!     println!("Teams: {teams:#?}");
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod builder;
pub mod client;
pub mod error;
pub mod linear;
pub mod telemetry;
pub mod transport;
pub mod types;

// Flatten the most-used surface to the crate root for ergonomics.
pub use builder::LinearClientBuilder;
pub use error::LinearMcpError;
pub use linear::{LinearClient, TypedLinearClient};
pub use types::*;
