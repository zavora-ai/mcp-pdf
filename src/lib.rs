//! # mcp-pdf
//!
//! The PDF Operating Layer for AI Agents — 57 MCP tools for inspecting, extracting,
//! generating, converting, manipulating, securing, and filling PDFs.
//!
//! ## Overview
//!
//! `mcp-pdf` is a Model Context Protocol (MCP) server that gives AI agents comprehensive
//! PDF capabilities. It runs as a standalone binary communicating over stdio.
//!
//! ## Installation
//!
//! ```bash
//! cargo install mcp-pdf
//! ```
//!
//! ## Configuration
//!
//! Add to your MCP client configuration:
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "pdf": {
//!       "command": "mcp-pdf"
//!     }
//!   }
//! }
//! ```
//!
//! ## Tool Categories
//!
//! | Category | Tools | Description |
//! |----------|-------|-------------|
//! | Inspect | 8 | Structural analysis, classification, health checks |
//! | Extract | 8 | Text, tables, images, metadata, annotations |
//! | Generate | 9 | Invoices, receipts, contracts, certificates, reports |
//! | Convert | 6 | PDF↔Markdown, HTML, JSON, CSV, images |
//! | Manipulate | 10 | Merge, split, rotate, compress, watermark |
//! | Numbering | 2 | Page numbers, Bates numbering |
//! | Security | 9 | Encrypt, redact, sanitize, scan PII |
//! | Forms | 5 | Detect, fill, flatten (interactive + flat) |
//!
//! ## Architecture
//!
//! Pure Rust with zero system dependencies:
//! - [`lopdf`](https://crates.io/crates/lopdf) — PDF read/write/manipulate
//! - [`printpdf`](https://crates.io/crates/printpdf) — PDF generation
//! - [`pdf_oxide`](https://crates.io/crates/pdf_oxide) — High-fidelity extraction
//! - [`comrak`](https://crates.io/crates/comrak) — Markdown parsing
//! - [`qrcode`](https://crates.io/crates/qrcode) — QR code generation

/// MCP server implementation with tool routing.
pub mod server;

/// Tool implementations organized by category.
pub mod tools;
