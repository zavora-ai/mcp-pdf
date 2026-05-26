//! PDF tool implementations organized by category.
//!
//! Each module contains standalone functions that implement the MCP tool logic.
//! The server module routes MCP calls to these functions.

/// Inspect & understand PDFs: structure, classification, health, features.
pub mod inspect;

/// Extract content: text, tables, images, metadata, annotations, bookmarks.
pub mod extract;

/// Manipulate existing PDFs: merge, split, rotate, compress, watermark.
pub mod manipulate;

/// Page numbering, Bates numbers, headers/footers, split by bookmarks.
pub mod numbering;

/// Generate professional documents: invoices, receipts, contracts, certificates.
pub mod generate;

/// Security: encrypt, decrypt, redact, sanitize, scan PII, permissions.
pub mod security;

/// Form handling: detect fields, fill interactive + flat forms, flatten.
pub mod forms;

/// Format conversion: PDF↔Markdown, HTML, JSON, CSV, images.
pub mod convert;
