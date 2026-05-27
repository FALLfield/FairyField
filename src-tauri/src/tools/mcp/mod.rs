//! MCP (Model Context Protocol) module
//!
//! Implements the Model Context Protocol transport layer,
//! input validation, and rate limiting for tool execution
//! exposed via MCP to external clients (Claude Desktop, Cursor, etc.).

pub mod composio;
pub mod oauth;
pub mod rate_limit;
pub mod transport;
pub mod validation;
