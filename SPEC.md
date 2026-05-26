# mcp-pdf v3.0 — Implementation Specification

> Complete engineering spec for building the PDF Operating Layer. Every tool, every module, every struct, every dependency — nothing omitted.

---

## 1. Project Structure

```
mcp-pdf/
├── Cargo.toml                    # Workspace root with feature flags
├── src/
│   ├── main.rs                   # Entry point: stdio transport, feature detection
│   ├── server.rs                 # MCP server: all tool registrations via #[tool_router]
│   ├── config.rs                 # Runtime config: branding, AI provider, paths
│   ├── error.rs                  # Unified error types + graceful degradation responses
│   │
│   ├── core/                     # ALWAYS COMPILED — pure Rust, zero external deps
│   │   ├── mod.rs
│   │   ├── inspect.rs            # Pillar 1: inspect_pdf, classify_pdf, health_check_pdf, etc.
│   │   ├── extract.rs            # Pillar 2: extract_text, extract_tables, etc.
│   │   ├── manipulate.rs         # Pillar 8: merge, split, rotate, crop, compress, etc.
│   │   ├── numbering.rs          # Pillar 9: page numbers, Bates, headers/footers, TOC
│   │   ├── forms.rs              # Pillar 10 (partial): detect_form_fields, fill_form, etc.
│   │   ├── security.rs           # Pillar 11: encrypt, redact, sanitize, hash, etc.
│   │   ├── accessibility.rs      # Pillar 12: audit, tag, alt_text, validate PDF/UA, etc.
│   │   └── batch.rs              # Pillar 13: batch_process, workflows, deduplicate
│   │
│   ├── generate/                 # Document generation engine
│   │   ├── mod.rs
│   │   ├── engine.rs             # Layout engine: pages, sections, tables, images
│   │   ├── style.rs              # Style system: 8 styles, color palettes, typography
│   │   ├── branding.rs           # Logo loading, color config, font embedding
│   │   ├── templates.rs          # Template registry + rendering
│   │   ├── documents.rs          # Pillar 7: create_invoice, create_report, etc.
│   │   └── multi_page.rs         # Auto page breaks, continuation headers
│   │
│   ├── verticals/                # Industry template packs
│   │   ├── mod.rs                # Vertical registry, schema validation
│   │   ├── healthcare.rs         # 8 templates
│   │   ├── legal.rs              # 10 templates
│   │   ├── finance.rs            # 10 templates
│   │   ├── government.rs         # 10 templates
│   │   ├── education.rs          # 10 templates
│   │   ├── real_estate.rs        # 8 templates
│   │   ├── hr.rs                 # 10 templates
│   │   ├── logistics.rs          # 8 templates
│   │   ├── insurance.rs          # 8 templates
│   │   └── construction.rs       # 8 templates
│   │
│   ├── ocr/                      # Feature: ocr
│   │   ├── mod.rs
│   │   └── tesseract.rs          # Pillar 3: ocr_pdf, ocr_page, ocr_region, etc.
│   │
│   ├── ai/                       # Feature: ai
│   │   ├── mod.rs
│   │   ├── provider.rs           # LLM provider abstraction (Anthropic/OpenAI/Ollama)
│   │   ├── rag.rs                # RAG pipeline: chunk, embed, search, cite
│   │   ├── intelligence.rs       # Pillar 4: summarize, answer, search, compare, etc.
│   │   └── idp.rs                # Pillar 5: parse_invoice, parse_contract, etc.
│   │
│   ├── conversion/               # Feature: conversion
│   │   ├── mod.rs
│   │   └── bridge.rs             # Pillar 6: LibreOffice/Pandoc bridge
│   │
│   └── signatures/               # Feature: signatures
│       ├── mod.rs
│       └── pkcs.rs               # Pillar 10 (partial): sign_pdf, validate_signatures
│
├── templates/                    # Static template definitions (JSON schemas)
│   ├── healthcare/
│   ├── legal/
│   ├── finance/
│   └── ...
│
├── fonts/                        # Bundled fonts (Inter, etc.) for generation
│
├── tests/
│   ├── integration/              # End-to-end tool tests
│   ├── fixtures/                 # Sample PDFs for testing
│   └── styles/                   # Visual regression tests
│
├── docs/
│   ├── assets/
│   │   └── architecture.svg
│   ├── api-reference.md
│   └── verticals.md
│
├── PROPOSAL-V3.md
├── SPEC.md                       # This file
├── README.md
└── mcp-server.toml
```

---

## 2. Cargo.toml

```toml
[package]
name = "mcp-pdf"
version = "3.0.0"
edition = "2021"
description = "The PDF Operating Layer for AI Agents — 131 tools, 88 templates, 10 verticals"
license = "Apache-2.0"
repository = "https://github.com/zavora-ai/mcp-pdf"
keywords = ["mcp", "pdf", "ai", "document", "enterprise"]
categories = ["command-line-utilities", "text-processing"]

[dependencies]
# MCP framework
rmcp = { version = "1.7", features = ["server", "transport-io", "macros"] }
schemars = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "2"

# Core PDF (always present)
lopdf = "0.40"
printpdf = { version = "0.7", features = ["embedded_images"] }
pdf-extract = "0.10"
image = "0.24"
sha2 = "0.10"
chrono = { version = "0.4", features = ["serde"] }
base64 = "0.22"
regex = "1"
glob = "0.3"
tempfile = "3"
walkdir = "2"

# OCR (optional)
tesseract = { version = "0.13", optional = true }

# AI (optional)
reqwest = { version = "0.12", features = ["json"], optional = true }

# Conversion (optional — detects system LibreOffice/Pandoc)
which = { version = "7", optional = true }

# Signatures (optional)
openssl = { version = "0.10", optional = true }

# Search (optional)
tantivy = { version = "0.22", optional = true }

# QR/Barcode
qrcode = "0.14"
bardecoder = { version = "0.5", optional = true }

[features]
default = ["core"]
core = []
ocr = ["dep:tesseract"]
ai = ["dep:reqwest"]
conversion = ["dep:which"]
signatures = ["dep:openssl"]
search = ["dep:tantivy"]
barcodes = ["dep:bardecoder"]
full = ["ocr", "ai", "conversion", "signatures", "search", "barcodes"]
```

---

## 3. Entry Point (main.rs)

```rust
use rmcp::{ServiceExt, transport::stdio};

mod server;
mod config;
mod error;
mod core;
mod generate;
mod verticals;

#[cfg(feature = "ocr")]
mod ocr;
#[cfg(feature = "ai")]
mod ai;
#[cfg(feature = "conversion")]
mod conversion;
#[cfg(feature = "signatures")]
mod signatures;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = server::PdfServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
```

---

## 4. Error Handling (error.rs)

```rust
use rmcp::model::{CallToolResult, Content};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Invalid PDF: {0}")]
    InvalidPdf(String),
    #[error("Corrupted PDF: {0}")]
    CorruptedPdf(String),
    #[error("Feature not available: {feature}. {guidance}")]
    FeatureNotAvailable { feature: String, guidance: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("PDF parse error: {0}")]
    Parse(String),
    #[error("Encryption: document is password-protected")]
    Encrypted,
    #[error("{0}")]
    Other(String),
}

impl PdfError {
    pub fn feature_missing(feature: &str, install: &str, fallback: &str) -> Self {
        Self::FeatureNotAvailable {
            feature: feature.to_string(),
            guidance: format!("Install: {}. Fallback: {}", install, fallback),
        }
    }

    pub fn to_tool_result(&self) -> CallToolResult {
        CallToolResult::error(vec![Content::text(self.to_string())])
    }
}
```

---

## 5. Configuration (config.rs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfConfig {
    pub branding: Option<Branding>,
    pub ai: Option<AiConfig>,
    pub ocr: Option<OcrConfig>,
    pub temp_dir: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Branding {
    pub logo: Option<String>,          // Path to logo PNG/JPEG
    pub primary_color: Option<String>, // Hex: "#0A1A33"
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    pub font_family: Option<String>,   // "Inter", "Helvetica", etc.
    pub font_path: Option<String>,     // Path to TTF/OTF
    pub company_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,              // "anthropic", "openai", "ollama", "bedrock"
    pub model: String,
    pub api_key: Option<String>,       // From env var, never stored
    pub base_url: Option<String>,      // For ollama/custom
    pub chunk_size: usize,             // Default: 8000
    pub chunk_overlap: usize,          // Default: 200
    pub max_context_pages: usize,      // Default: 50
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OcrConfig {
    pub language: String,              // Default: "eng"
    pub dpi: u32,                      // Default: 300
    pub psm: u8,                       // Page segmentation mode
}
```

---

## 6. Style System (generate/style.rs)

```rust
use printpdf::*;

#[derive(Clone, Debug)]
pub struct Style {
    pub name: &'static str,
    pub colors: ColorPalette,
    pub typography: Typography,
    pub table: TableStyle,
    pub layout: LayoutConfig,
}

#[derive(Clone, Debug)]
pub struct ColorPalette {
    pub primary: Rgb,        // Header backgrounds, totals
    pub secondary: Rgb,      // Accent elements
    pub text: Rgb,           // Body text
    pub muted: Rgb,          // Labels, captions
    pub background: Rgb,     // Page background
    pub table_header_bg: Rgb,
    pub table_alt_row: Rgb,
    pub border: Rgb,
}

#[derive(Clone, Debug)]
pub struct Typography {
    pub title_size: f32,     // pt
    pub heading_size: f32,
    pub body_size: f32,
    pub caption_size: f32,
    pub label_size: f32,
    pub line_height: f32,    // multiplier
}

#[derive(Clone, Debug)]
pub struct TableStyle {
    pub header_bg: bool,
    pub alt_rows: bool,
    pub borders: BorderStyle,
    pub cell_padding: f32,   // mm
}

#[derive(Clone, Debug)]
pub enum BorderStyle {
    None,
    Hairline,      // 0.25pt
    Thin,          // 0.5pt
    HeaderOnly,    // Only under header row
    Full,          // All cells
}

#[derive(Clone, Debug)]
pub struct LayoutConfig {
    pub margin_top: f32,     // mm
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub header_height: f32,  // mm (0 = no header bar)
    pub footer_height: f32,
    pub column_gap: f32,     // For multi-column
    pub columns: u8,         // 1 or 2
}

/// All 8 built-in styles
pub fn get_style(name: &str) -> Style {
    match name {
        "minimal" => style_minimal(),
        "modern" => style_modern(),
        "corporate" => style_corporate(),
        "stripe" => style_stripe(),
        "legal" => style_legal(),
        "academic" => style_academic(),
        "government" => style_government(),
        "finance" => style_finance(),
        _ => style_minimal(), // fallback
    }
}

fn style_minimal() -> Style {
    Style {
        name: "minimal",
        colors: ColorPalette {
            primary: Rgb::new(0.2, 0.2, 0.2, None),
            secondary: Rgb::new(0.4, 0.4, 0.4, None),
            text: Rgb::new(0.15, 0.15, 0.15, None),
            muted: Rgb::new(0.6, 0.6, 0.6, None),
            background: Rgb::new(1.0, 1.0, 1.0, None),
            table_header_bg: Rgb::new(1.0, 1.0, 1.0, None),
            table_alt_row: Rgb::new(1.0, 1.0, 1.0, None),
            border: Rgb::new(0.85, 0.85, 0.85, None),
        },
        typography: Typography {
            title_size: 24.0, heading_size: 14.0, body_size: 10.0,
            caption_size: 8.0, label_size: 8.0, line_height: 1.4,
        },
        table: TableStyle {
            header_bg: false, alt_rows: false,
            borders: BorderStyle::Hairline, cell_padding: 2.0,
        },
        layout: LayoutConfig {
            margin_top: 25.0, margin_bottom: 20.0,
            margin_left: 25.0, margin_right: 25.0,
            header_height: 0.0, footer_height: 10.0,
            column_gap: 0.0, columns: 1,
        },
    }
}

// ... style_modern(), style_corporate(), style_stripe(),
//     style_legal(), style_academic(), style_government(), style_finance()
// Each follows same pattern with different values
```

---

## 7. Generation Engine (generate/engine.rs)

```rust
use printpdf::*;
use std::io::BufWriter;
use std::fs::File;

pub struct PdfEngine {
    doc: PdfDocumentReference,
    style: Style,
    branding: Option<Branding>,
    current_page: usize,
    current_y: f32,          // Current Y position (mm from bottom)
    page_height: f32,        // 297.0 for A4
    page_width: f32,         // 210.0 for A4
    content_top: f32,        // After header
    content_bottom: f32,     // Before footer
}

impl PdfEngine {
    pub fn new(title: &str, style: Style, branding: Option<Branding>) -> Self { ... }

    // === Page management ===
    pub fn new_page(&mut self) -> PdfLayerReference { ... }
    pub fn check_page_break(&mut self, needed_height: f32) { ... }
    pub fn current_layer(&self) -> PdfLayerReference { ... }

    // === Header/Footer ===
    pub fn draw_header(&self, layer: &PdfLayerReference) { ... }
    pub fn draw_footer(&self, layer: &PdfLayerReference, page_num: usize) { ... }
    pub fn draw_logo(&self, layer: &PdfLayerReference) { ... }

    // === Text primitives ===
    pub fn draw_title(&mut self, text: &str) { ... }
    pub fn draw_heading(&mut self, text: &str) { ... }
    pub fn draw_body(&mut self, text: &str) { ... }
    pub fn draw_label(&mut self, text: &str) { ... }
    pub fn draw_text(&mut self, text: &str, size: f32, x: f32, bold: bool) { ... }

    // === Layout primitives ===
    pub fn draw_rect(&self, layer: &PdfLayerReference, x: f32, y: f32, w: f32, h: f32, color: Rgb) { ... }
    pub fn draw_line(&self, layer: &PdfLayerReference, x1: f32, y1: f32, x2: f32, y2: f32) { ... }
    pub fn draw_separator(&mut self) { ... }
    pub fn advance_y(&mut self, mm: f32) { ... }

    // === Table ===
    pub fn draw_table(&mut self, headers: &[&str], rows: &[Vec<String>], col_widths: &[f32]) { ... }
    pub fn draw_table_header(&mut self, headers: &[&str], col_widths: &[f32]) { ... }
    pub fn draw_table_row(&mut self, cells: &[String], col_widths: &[f32], alt: bool) { ... }

    // === Image ===
    pub fn draw_image(&mut self, path: &str, x: f32, y: f32, max_width: f32) { ... }
    pub fn draw_qr_code(&mut self, data: &str, x: f32, y: f32, size: f32) { ... }

    // === Signature blocks ===
    pub fn draw_signature_block(&mut self, name: &str, title: &str) { ... }
    pub fn draw_signature_line(&mut self, label: &str) { ... }

    // === Save ===
    pub fn save(&self, path: &str) -> Result<(), PdfError> { ... }
}
```

---

## 8. Branding & Logo (generate/branding.rs)

```rust
use printpdf::*;
use image::DynamicImage;

pub struct LoadedBranding {
    pub logo_image: Option<(Vec<u8>, u32, u32)>,  // RGB data, width, height
    pub primary: Rgb,
    pub secondary: Rgb,
    pub accent: Rgb,
    pub company_name: Option<String>,
    pub font: Option<Vec<u8>>,  // TTF bytes
}

impl LoadedBranding {
    /// Load branding from config. Converts logo to RGB (no alpha for Preview compat).
    pub fn from_config(branding: &Branding) -> Self { ... }

    /// Parse hex color "#RRGGBB" to Rgb
    fn parse_hex(hex: &str) -> Rgb { ... }

    /// Load and convert image to RGB (strip alpha channel)
    fn load_logo(path: &str) -> Option<(Vec<u8>, u32, u32)> {
        let bytes = std::fs::read(path).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
        let rgb = img.to_rgb8();
        let (w, h) = rgb.dimensions();
        Some((rgb.into_raw(), w, h))
    }

    /// Add logo to layer at position with auto-scaling
    pub fn add_logo_to_layer(&self, layer: &PdfLayerReference, x: f32, y: f32, max_width: f32) {
        if let Some((ref data, w, h)) = self.logo_image {
            let scale = (max_width / (*w as f32 * 0.353)).min(1.0); // 1pt = 0.353mm
            let image = Image::from(ImageXObject {
                width: Px(*w as usize),
                height: Px(*h as usize),
                color_space: ColorSpace::Rgb,
                bits_per_component: ColorBits::Bit8,
                interpolate: true,
                image_data: data.clone(),
                image_filter: None,
                clipping_bbox: None,
                smask: None,
            });
            image.add_to_layer(layer.clone(), ImageTransform {
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(y)),
                scale_x: Some(scale),
                scale_y: Some(scale),
                ..Default::default()
            });
        }
    }
}
```

---

## 9. Tool Specifications — Pillar 1: Inspect & Understand

### 9.1 inspect_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct InspectPdfInput {
    pub pdf_path: String,
}

// Output JSON:
{
    "path": "/path/to/file.pdf",
    "file_size_bytes": 245000,
    "pdf_version": "1.7",
    "page_count": 12,
    "pages": [{ "width": 612, "height": 792, "rotation": 0 }],
    "fonts": ["Helvetica", "Times-Roman"],
    "has_images": true,
    "image_count": 3,
    "has_forms": false,
    "has_signatures": true,
    "has_encryption": false,
    "has_javascript": false,
    "has_embedded_files": false,
    "has_bookmarks": true,
    "has_annotations": true,
    "metadata": { "title": "...", "author": "...", "creator": "...", "created": "..." }
}
```

**Implementation:** Use `lopdf::Document::load()`, iterate objects, check for `/AcroForm`, `/Sig`, `/JavaScript`, `/EmbeddedFiles`, `/Outlines`, `/Annots`.

### 9.2 classify_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ClassifyPdfInput {
    pub pdf_path: String,
}

// Output: document type classification
{
    "classification": "invoice",  // invoice|contract|form|scan|report|letter|certificate|presentation|spreadsheet|unknown
    "confidence": 0.85,
    "signals": ["has_line_items", "contains_total", "has_vendor_info"],
    "is_scanned": false,
    "has_selectable_text": true,
    "primary_language": "en"
}
```

**Implementation:** Extract first 2 pages of text. Use keyword heuristics:
- Invoice: "invoice", "bill to", "total", "qty", "amount"
- Contract: "agreement", "parties", "whereas", "hereby", "clause"
- Form: has AcroForm fields
- Scan: pages are single large images with no text layer
- Report: "executive summary", "table of contents", section headings
- Letter: "dear", "sincerely", "regards", short document
- Certificate: "certify", "awarded", "completion", single page

### 9.3 health_check_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct HealthCheckInput {
    pub pdf_path: String,
}

// Output:
{
    "status": "healthy",  // healthy|warnings|corrupted
    "issues": [],
    "xref_valid": true,
    "all_objects_readable": true,
    "stream_integrity": true,
    "page_tree_valid": true,
    "repairable": true
}
```

**Implementation:** `lopdf::Document::load()` with error catching. Check xref table, iterate all objects, verify stream lengths, validate page tree.

### 9.4 detect_features

```rust
// Output:
{
    "has_forms": false,
    "form_field_count": 0,
    "has_tags": true,
    "has_signatures": false,
    "signature_count": 0,
    "has_javascript": false,
    "has_embedded_files": false,
    "embedded_file_count": 0,
    "has_annotations": true,
    "annotation_count": 5,
    "has_bookmarks": true,
    "has_layers": false,
    "has_transparency": true,
    "has_3d_content": false,
    "color_spaces": ["DeviceRGB", "DeviceCMYK"]
}
```

### 9.5 profile_complexity

```rust
// Output:
{
    "complexity_score": 3,  // 1-5 scale
    "level": "moderate",    // simple|moderate|complex|very_complex|extreme
    "factors": {
        "is_scanned": false,
        "has_multi_column": true,
        "has_tables": true,
        "has_images_over_text": false,
        "has_watermarks": false,
        "page_count": 12,
        "estimated_word_count": 4500,
        "font_count": 3,
        "has_non_latin": false
    },
    "recommended_approach": "extract_text_structured for best results"
}
```

### 9.6 get_page_count

```rust
// Simple: returns integer
"12"
```

**Implementation:** `doc.get_pages().len()`

### 9.7 get_info

```rust
// Output:
{
    "page_count": 12,
    "file_size": "245 KB",
    "pdf_version": "1.7",
    "encrypted": false,
    "title": "Q2 Report",
    "author": "James Karanja",
    "created": "2026-05-20T10:30:00Z",
    "modified": "2026-05-25T14:22:00Z"
}
```

### 9.8 repair_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct RepairInput {
    pub pdf_path: String,
    pub output: String,
    pub mode: Option<String>,  // "safe"|"aggressive" (default: "safe")
}

// Output:
{
    "status": "repaired",
    "output": "/path/to/repaired.pdf",
    "fixes_applied": ["rebuilt_xref", "fixed_stream_lengths"],
    "pages_recovered": 12,
    "pages_lost": 0
}
```

**Implementation:** Load with `lopdf::Document::load()` in permissive mode. Rebuild xref. Re-serialize. For aggressive: attempt object-by-object recovery.

---

## 10. Tool Specifications — Pillar 2: Extract Content

### 10.1 extract_text

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ExtractTextInput {
    pub pdf_path: String,
    pub pages: Option<String>,  // "1-5,10" or null for all
}
// Returns: plain text string
```

**Implementation:** `pdf_extract::extract_text()` or page-by-page with `pdf_extract::extract_text_from_page()`.

### 10.2 extract_text_structured

```rust
// Output:
{
    "pages": [
        {
            "page_number": 1,
            "blocks": [
                {
                    "type": "heading",  // heading|paragraph|list_item|table_cell|caption
                    "text": "Executive Summary",
                    "bbox": [72, 700, 300, 720],
                    "font_size": 16.0,
                    "bold": true
                }
            ]
        }
    ],
    "total_words": 4500,
    "total_pages": 12
}
```

**Implementation:** Use `lopdf` to iterate content streams, parse text operators (Tj, TJ, Tm), track font size changes to detect headings vs body.

### 10.3 extract_page_text

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ExtractPageTextInput {
    pub pdf_path: String,
    pub page_number: u32,
}
// Returns: text from that page
```

### 10.4 extract_layout

```rust
// Output:
{
    "pages": [
        {
            "page_number": 1,
            "elements": [
                { "type": "heading", "level": 1, "text": "Title", "y": 750 },
                { "type": "paragraph", "text": "...", "y": 700 },
                { "type": "table", "rows": 5, "cols": 4, "y": 500 },
                { "type": "image", "width": 200, "height": 150, "y": 300 }
            ],
            "columns": 1,
            "reading_order": [0, 1, 2, 3]
        }
    ]
}
```

### 10.5 extract_tables

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ExtractTablesInput {
    pub pdf_path: String,
    pub pages: Option<String>,
    pub format: Option<String>,  // "json"|"csv" (default: "json")
}

// Output:
{
    "tables": [
        {
            "page": 1,
            "headers": ["Description", "Qty", "Price", "Total"],
            "rows": [
                ["Widget A", "10", "$5.00", "$50.00"],
                ["Widget B", "5", "$12.00", "$60.00"]
            ],
            "bbox": [72, 400, 540, 550]
        }
    ],
    "table_count": 2
}
```

**Implementation:** Detect tables by finding aligned text blocks with consistent column positions. Use horizontal/vertical line detection from content streams.

### 10.6 extract_key_values

```rust
// Output:
{
    "pairs": [
        { "key": "Invoice Number", "value": "INV-2026-042", "page": 1 },
        { "key": "Date", "value": "May 26, 2026", "page": 1 },
        { "key": "Total", "value": "$57,000.00", "page": 1 }
    ]
}
```

**Implementation:** Find patterns like "Label: Value", "Label    Value" (colon-separated or tab-aligned).

### 10.7 extract_images

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ExtractImagesInput {
    pub pdf_path: String,
    pub output_dir: Option<String>,  // If set, export images to this dir
}

// Output:
{
    "images": [
        { "page": 1, "index": 0, "width": 200, "height": 62, "color_space": "RGB", "bits": 8, "exported": "/tmp/img_0.png" }
    ],
    "count": 3
}
```

**Implementation:** Iterate objects, find `/Subtype /Image`, extract raw data, decode based on `/Filter`.

### 10.8 extract_figures

```rust
// Output: figures with captions (images that have nearby text labels)
{
    "figures": [
        { "page": 3, "caption": "Figure 1: Revenue Growth", "bbox": [72, 300, 400, 500], "type": "chart" }
    ]
}
```

### 10.9 extract_annotations

```rust
// Output:
{
    "annotations": [
        { "page": 1, "type": "highlight", "color": "yellow", "text": "important clause", "author": "James" },
        { "page": 2, "type": "comment", "content": "Review this section", "author": "Sarah", "date": "2026-05-20" },
        { "page": 3, "type": "stamp", "name": "APPROVED" }
    ],
    "count": 5
}
```

**Implementation:** Iterate `/Annots` array on each page, parse `/Subtype`, extract `/Contents`, `/T` (author), `/M` (date).

### 10.10 extract_bookmarks

```rust
// Output:
{
    "bookmarks": [
        { "title": "Chapter 1", "page": 1, "children": [
            { "title": "Section 1.1", "page": 3, "children": [] }
        ]},
        { "title": "Chapter 2", "page": 15, "children": [] }
    ]
}
```

**Implementation:** Parse `/Outlines` dictionary, follow `/First`/`/Next`/`/Last` linked list.

### 10.11 extract_attachments

```rust
// Output:
{
    "attachments": [
        { "name": "data.xlsx", "size": 45000, "mime_type": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" }
    ],
    "count": 1
}
```

**Implementation:** Check `/Names` → `/EmbeddedFiles` in document catalog.

### 10.12 extract_barcodes

```rust
// Output:
{
    "barcodes": [
        { "page": 1, "type": "QR", "value": "https://zavora.ai/invoice/123", "bbox": [450, 700, 550, 800] }
    ]
}
```

**Implementation:** Render page to image, use `bardecoder` crate to detect QR/barcodes.

### 10.13 extract_metadata

```rust
// Output:
{
    "title": "Q2 Report",
    "author": "James Karanja",
    "subject": "Quarterly Performance",
    "keywords": "finance, report, Q2",
    "creator": "mcp-pdf v3.0",
    "producer": "printpdf 0.7",
    "created": "2026-05-20T10:30:00Z",
    "modified": "2026-05-25T14:22:00Z",
    "xmp": { ... }  // Raw XMP if present
}
```

**Implementation:** Read `/Info` dictionary + parse XMP metadata stream if present.

### 10.14 extract_reading_order

```rust
// Output:
{
    "pages": [
        {
            "page_number": 1,
            "reading_order": [
                { "index": 0, "type": "heading", "text": "Title..." },
                { "index": 1, "type": "paragraph", "text": "First paragraph..." },
                { "index": 2, "type": "table", "summary": "4x5 table" }
            ]
        }
    ]
}
```

**Implementation:** If tagged PDF: follow structure tree. Otherwise: sort text blocks by Y (top-to-bottom), then X (left-to-right) within same Y band.

---

## 11. Tool Specifications — Pillar 3: OCR & Scan Processing

*All tools gated behind `#[cfg(feature = "ocr")]`. Return `FeatureNotAvailable` if missing.*

### 11.1 detect_ocr_needed

```rust
#[derive(Deserialize, JsonSchema)]
pub struct DetectOcrInput { pub pdf_path: String }

// Output:
{
    "ocr_needed": true,
    "scanned_pages": [1, 2, 5],
    "text_pages": [3, 4],
    "total_pages": 5,
    "recommendation": "ocr_pdf with language 'eng'"
}
```

**Implementation:** For each page, check if it has text operators in content stream. If page only has image XObjects and no text, it's scanned.

### 11.2 ocr_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct OcrPdfInput {
    pub pdf_path: String,
    pub output: String,
    pub language: Option<String>,      // Default: "eng". Multi: "eng+fra"
    pub pages: Option<String>,         // "1-5" or all
    pub dpi: Option<u32>,              // Default: 300
}

// Output:
{
    "output": "/path/to/searchable.pdf",
    "pages_processed": 5,
    "languages_detected": ["en"],
    "confidence": 0.92,
    "word_count": 3200
}
```

**Implementation:** Extract page images → run Tesseract → create new PDF with text layer overlaid on original images.

### 11.3 ocr_page

```rust
#[derive(Deserialize, JsonSchema)]
pub struct OcrPageInput {
    pub pdf_path: String,
    pub page_number: u32,
    pub language: Option<String>,
}
// Returns: extracted text from OCR
```

### 11.4 ocr_region

```rust
#[derive(Deserialize, JsonSchema)]
pub struct OcrRegionInput {
    pub pdf_path: String,
    pub page_number: u32,
    pub bbox: [f32; 4],  // [x1, y1, x2, y2] in points
    pub language: Option<String>,
}
// Returns: text from that region
```

### 11.5 ocr_table

```rust
// Output: structured table from OCR
{
    "headers": ["Date", "Description", "Amount"],
    "rows": [
        ["2026-05-01", "Payment received", "$5,000.00"],
        ["2026-05-15", "Service fee", "-$250.00"]
    ],
    "confidence": 0.88
}
```

**Implementation:** OCR the region, then apply table detection heuristics (aligned columns, consistent spacing).

### 11.6 clean_scan

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CleanScanInput {
    pub pdf_path: String,
    pub output: String,
    pub deskew: Option<bool>,      // Default: true
    pub denoise: Option<bool>,     // Default: true
    pub contrast: Option<bool>,    // Default: false
}
// Output: cleaned PDF path + operations applied
```

**Implementation:** Extract images, apply image processing (rotation correction, noise removal, contrast enhancement), re-embed.

---

## 12. Tool Specifications — Pillar 4: AI Intelligence

*All tools gated behind `#[cfg(feature = "ai")]`.*

### AI Provider Abstraction (ai/provider.rs)

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, prompt: &str, system: &str) -> Result<String>;
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

pub struct AnthropicProvider { api_key: String, model: String }
pub struct OpenAiProvider { api_key: String, model: String }
pub struct OllamaProvider { base_url: String, model: String }

pub fn create_provider(config: &AiConfig) -> Box<dyn LlmProvider> { ... }
```

### RAG Pipeline (ai/rag.rs)

```rust
pub struct RagPipeline {
    provider: Box<dyn LlmProvider>,
    chunk_size: usize,
    chunk_overlap: usize,
}

impl RagPipeline {
    pub fn chunk_text(text: &str, page_map: &[(usize, usize)]) -> Vec<Chunk> { ... }
    pub async fn embed_chunks(&self, chunks: &[Chunk]) -> Vec<EmbeddedChunk> { ... }
    pub fn search(&self, query_embedding: &[f32], chunks: &[EmbeddedChunk], top_k: usize) -> Vec<SearchResult> { ... }
    pub async fn answer(&self, question: &str, context: &[SearchResult]) -> AnswerResult { ... }
}

pub struct Chunk { pub text: String, pub page: usize, pub start_char: usize }
pub struct EmbeddedChunk { pub chunk: Chunk, pub embedding: Vec<f32> }
pub struct SearchResult { pub chunk: Chunk, pub score: f32 }
pub struct AnswerResult { pub answer: String, pub citations: Vec<Citation> }
pub struct Citation { pub page: usize, pub text: String, pub score: f32 }
```

### 12.1 summarize_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SummarizeInput {
    pub pdf_path: String,
    pub summary_type: Option<String>,  // "executive"|"bullets"|"section" (default: "executive")
    pub max_length: Option<usize>,     // Max words (default: 300)
    pub pages: Option<String>,
}

// Output:
{
    "summary": "...",
    "summary_type": "executive",
    "source_pages": [1, 2, 3, 4, 5],
    "word_count": 250
}
```

### 12.2 answer_question

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AnswerQuestionInput {
    pub pdf_path: String,
    pub question: String,
}

// Output:
{
    "answer": "Payment is due within 30 days of invoice date.",
    "confidence": 0.93,
    "citations": [
        { "page": 3, "text": "Payment shall be due within thirty (30) days...", "bbox": [72, 420, 510, 440] }
    ]
}
```

### 12.3 search_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SearchPdfInput {
    pub pdf_path: String,
    pub query: String,
    pub case_sensitive: Option<bool>,
}

// Output:
{
    "matches": [
        { "page": 3, "text": "...payment terms...", "position": 1245, "context": "The payment terms require..." }
    ],
    "total_matches": 4
}
```

**Implementation:** Extract text, regex/substring search with context window.

### 12.4 semantic_search

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SemanticSearchInput {
    pub pdf_path: String,
    pub query: String,
    pub top_k: Option<usize>,  // Default: 5
}

// Output:
{
    "results": [
        { "page": 5, "text": "The contractor shall deliver...", "score": 0.89 }
    ]
}
```

### 12.5 build_index

```rust
#[derive(Deserialize, JsonSchema)]
pub struct BuildIndexInput {
    pub path: String,          // File or directory
    pub index_name: String,
}
// Output: { "indexed_files": 42, "total_chunks": 1200, "index_path": "..." }
```

### 12.6–12.10 extract_entities, extract_claims, extract_timeline, extract_obligations, extract_risks

All follow same pattern:
```rust
#[derive(Deserialize, JsonSchema)]
pub struct ExtractEntitiesInput {
    pub pdf_path: String,
    pub entity_types: Option<Vec<String>>,  // ["person", "org", "date", "amount", "location"]
}

// Output varies by tool:
// entities: [{ "type": "person", "value": "James Karanja", "page": 1, "confidence": 0.95 }]
// claims: [{ "claim": "Revenue grew 42%", "page": 2, "source_text": "..." }]
// timeline: [{ "date": "2026-01-15", "event": "Contract signed", "page": 1 }]
// obligations: [{ "party": "Contractor", "obligation": "Deliver by June 1", "deadline": "2026-06-01", "page": 4 }]
// risks: [{ "category": "financial", "description": "Unlimited liability clause", "severity": "high", "page": 7 }]
```

**Implementation:** Extract text → LLM with structured JSON schema prompt → validate output.

### 12.11 compare_documents

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CompareInput {
    pub pdf_a: String,
    pub pdf_b: String,
    pub focus: Option<String>,  // "text"|"semantic"|"both" (default: "both")
}

// Output:
{
    "text_diff": { "added_lines": 15, "removed_lines": 8, "changed_lines": 3 },
    "semantic_changes": [
        { "category": "payment_terms", "change": "Net 30 → Net 60", "page_a": 4, "page_b": 4, "significance": "high" }
    ],
    "summary": "Key change: payment terms extended from 30 to 60 days."
}
```

### 12.12 translate_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct TranslateInput {
    pub pdf_path: String,
    pub target_language: String,  // "es", "fr", "sw", "zh", etc.
    pub output: String,
    pub preserve_layout: Option<bool>,  // Default: true
}
// Output: translated PDF path + word count
```

### 12.13 assess_reading_level

```rust
// Output:
{
    "flesch_kincaid_grade": 12.3,
    "flesch_reading_ease": 45.2,
    "gunning_fog": 14.1,
    "level": "college",
    "avg_sentence_length": 22.5,
    "avg_syllables_per_word": 1.8,
    "complex_word_pct": 18.5
}
```

**Implementation:** Pure text analysis — no AI needed. Count sentences, words, syllables.

### 12.14 generate_brief

```rust
#[derive(Deserialize, JsonSchema)]
pub struct GenerateBriefInput {
    pub pdf_path: String,
    pub brief_type: Option<String>,  // "executive"|"legal"|"research" (default: "executive")
    pub output: String,
}
// Output: generates a new PDF brief from the source document
```

---

## 13. Tool Specifications — Pillar 5: Document Intelligence (IDP)

*All tools gated behind `#[cfg(feature = "ai")]`.*

### Common IDP Output Structure

```rust
#[derive(Serialize)]
pub struct IdpResult<T> {
    pub document_type: String,
    pub confidence: f32,
    pub data: T,
    pub warnings: Vec<String>,
}

#[derive(Serialize)]
pub struct FieldWithConfidence<T> {
    pub value: T,
    pub confidence: f32,
}
```

### 13.1 parse_invoice

```rust
// Output:
{
    "document_type": "invoice",
    "confidence": 0.97,
    "data": {
        "vendor": { "value": { "name": "Zavora AI", "address": "Nairobi, Kenya" }, "confidence": 0.99 },
        "customer": { "value": { "name": "Acme Corp", "address": "SF, CA" }, "confidence": 0.95 },
        "invoice_number": { "value": "INV-2026-042", "confidence": 0.99 },
        "date": { "value": "2026-05-26", "confidence": 0.98 },
        "due_date": { "value": "2026-06-25", "confidence": 0.90 },
        "currency": { "value": "USD", "confidence": 0.95 },
        "line_items": [
            { "description": "Enterprise License", "quantity": 1, "unit_price": 25000.00, "total": 25000.00, "confidence": 0.95 }
        ],
        "subtotal": { "value": 52000.00, "confidence": 0.97 },
        "tax": { "value": 5000.00, "confidence": 0.85 },
        "total": { "value": 57000.00, "confidence": 0.98 }
    }
}
```

### 13.2–13.7 parse_receipt, parse_id_document, parse_contract, parse_bank_statement, parse_form, parse_resume

Each follows same pattern: extract text → LLM with schema → validate → return typed JSON with confidence.

**parse_receipt schema:** merchant, date, items[], total, payment_method, transaction_id
**parse_id_document schema:** name, dob, id_number, expiry, nationality, document_type, issuing_authority
**parse_contract schema:** parties[], effective_date, term, governing_law, clauses[], obligations[], termination_conditions
**parse_bank_statement schema:** bank, account_number, account_holder, period, opening_balance, transactions[], closing_balance
**parse_form schema:** form_title, fields[] (each: label, value, field_type)
**parse_resume schema:** name, email, phone, location, summary, experience[], education[], skills[], certifications[]

### 13.8 extract_line_items

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ExtractLineItemsInput {
    pub pdf_path: String,
    pub pages: Option<String>,
}

// Output:
{
    "line_items": [
        { "description": "Widget A", "quantity": 10, "unit_price": 5.00, "total": 50.00 }
    ],
    "source_pages": [1, 2]
}
```

### 13.9 extract_named_entities

```rust
// Output:
{
    "entities": {
        "persons": ["James Karanja", "Sarah Kimani"],
        "organizations": ["Zavora AI", "Acme Corporation"],
        "locations": ["Nairobi", "San Francisco"],
        "dates": ["May 26, 2026", "June 1, 2026"],
        "amounts": ["$57,000.00", "KES 1,920,000"]
    }
}
```

---

## 14. Tool Specifications — Pillar 6: Format Conversion

*All tools gated behind `#[cfg(feature = "conversion")]`.*

### Conversion Bridge (conversion/bridge.rs)

```rust
pub struct ConversionBridge;

impl ConversionBridge {
    /// Check if LibreOffice is available
    pub fn libreoffice_available() -> bool {
        which::which("soffice").is_ok() || which::which("libreoffice").is_ok()
    }

    /// Check if Pandoc is available
    pub fn pandoc_available() -> bool {
        which::which("pandoc").is_ok()
    }

    /// Convert using LibreOffice headless
    pub async fn libreoffice_convert(input: &str, output_format: &str, output_dir: &str) -> Result<String> {
        // soffice --headless --convert-to {format} --outdir {dir} {input}
    }

    /// Convert using Pandoc
    pub async fn pandoc_convert(input: &str, output: &str, from: &str, to: &str) -> Result<String> {
        // pandoc -f {from} -t {to} -o {output} {input}
    }
}
```

### 14.1 pdf_to_markdown

```rust
#[derive(Deserialize, JsonSchema)]
pub struct PdfToMarkdownInput {
    pub pdf_path: String,
    pub output: Option<String>,
    pub preserve_tables: Option<bool>,  // Default: true
    pub preserve_images: Option<bool>,  // Default: false
}
// Returns: markdown string (or writes to output file)
```

**Implementation:** Extract structured text → convert headings to `#`, tables to `|` format, lists to `-`.

### 14.2 pdf_to_html

```rust
// Uses pandoc or custom conversion
```

### 14.3 pdf_to_docx

```rust
// Uses LibreOffice: soffice --headless --convert-to docx
```

### 14.4 pdf_to_images

```rust
#[derive(Deserialize, JsonSchema)]
pub struct PdfToImagesInput {
    pub pdf_path: String,
    pub output_dir: String,
    pub format: Option<String>,  // "png"|"jpeg"|"webp" (default: "png")
    pub dpi: Option<u32>,        // Default: 150
    pub pages: Option<String>,
}
// Output: list of generated image paths
```

### 14.5 pdf_to_json

```rust
// Combines extract_text_structured + extract_tables + extract_metadata into one JSON
```

### 14.6 pdf_to_csv

```rust
// Extracts tables and writes each as CSV
```

### 14.7 markdown_to_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct MarkdownToPdfInput {
    pub markdown: String,       // Markdown content or file path
    pub output: String,
    pub style: Option<String>,  // Default: "minimal"
    pub title: Option<String>,
}
```

**Implementation:** Parse markdown (headings, paragraphs, lists, tables, code blocks) → render with PdfEngine using selected style.

### 14.8 html_to_pdf

```rust
// Uses headless Chrome/Chromium if available, otherwise pandoc
```

### 14.9 images_to_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ImagesToPdfInput {
    pub image_paths: Vec<String>,
    pub output: String,
    pub page_size: Option<String>,  // "a4"|"letter"|"fit" (default: "a4")
}
```

**Implementation:** Create PDF with one image per page using printpdf.

### 14.10 office_to_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct OfficeToPdfInput {
    pub input_path: String,   // .docx, .xlsx, .pptx
    pub output: String,
}
// Uses LibreOffice headless
```

---

## 15. Tool Specifications — Pillar 7: Document Generation

### Common Generation Input Pattern

All generation tools accept:
```rust
pub struct CommonGenInput {
    pub output: String,
    pub style: Option<String>,         // "minimal"|"modern"|"corporate"|"stripe"|"legal"|"academic"|"government"|"finance"
    pub vertical: Option<String>,      // "healthcare"|"legal"|"finance"|... (applies vertical template)
    pub template: Option<String>,      // Specific template name within vertical
    pub branding: Option<BrandingInput>,
    pub compliance: Option<Vec<String>>, // ["hipaa", "pdf_ua", "pdf_a"]
}

pub struct BrandingInput {
    pub logo: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub company_name: Option<String>,
    pub font_path: Option<String>,
}
```

### 15.1 create_document

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateDocumentInput {
    pub output: String,
    pub title: String,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
    pub sections: Vec<Section>,
    pub header: Option<String>,
    pub footer: Option<String>,
}

pub struct Section {
    pub heading: Option<String>,
    pub body: Option<String>,
    pub table: Option<TableData>,
    pub image: Option<String>,       // Path to image
    pub list: Option<Vec<String>>,
    pub page_break: Option<bool>,
}

pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}
```

### 15.2 create_from_template

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateFromTemplateInput {
    pub output: String,
    pub vertical: String,
    pub template: String,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
    pub data: serde_json::Value,  // Template-specific data
    pub compliance: Option<Vec<String>>,
}
```

**Implementation:** Look up template schema from `verticals/` module → validate data against schema → render using PdfEngine with vertical-specific layout rules.

### 15.3 create_template

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateTemplateInput {
    pub name: String,
    pub vertical: Option<String>,
    pub fields: Vec<TemplateField>,
    pub sections: Vec<TemplateSectionDef>,
    pub style_base: Option<String>,
}

pub struct TemplateField {
    pub name: String,
    pub field_type: String,  // "string"|"number"|"date"|"array"|"object"
    pub required: bool,
    pub label: Option<String>,
}
```

### 15.4 create_invoice

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateInvoiceInput {
    pub output: String,
    pub company: String,
    pub customer: String,
    pub invoice_number: Option<String>,
    pub date: Option<String>,
    pub due_date: Option<String>,
    pub items: Vec<InvoiceItem>,
    pub tax_rate: Option<f64>,
    pub notes: Option<String>,
    pub payment_terms: Option<String>,
    pub currency: Option<String>,      // Default: "USD"
    pub logo: Option<String>,
    pub style: Option<String>,         // Default: "minimal"
    pub branding: Option<BrandingInput>,
}

pub struct InvoiceItem {
    pub description: String,
    pub quantity: u32,
    pub unit_price_cents: i64,
}
```

### 15.5 create_receipt

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateReceiptInput {
    pub output: String,
    pub company: String,
    pub customer: String,
    pub receipt_number: Option<String>,
    pub date: Option<String>,
    pub items: Vec<ReceiptItem>,
    pub payment_method: Option<String>,
    pub transaction_id: Option<String>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}
```

### 15.6 create_quote

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateQuoteInput {
    pub output: String,
    pub company: String,
    pub customer: String,
    pub quote_number: Option<String>,
    pub valid_until: Option<String>,
    pub items: Vec<InvoiceItem>,
    pub notes: Option<String>,
    pub terms: Option<String>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}
```

### 15.7 create_statement

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateStatementInput {
    pub output: String,
    pub company: String,
    pub account_holder: String,
    pub account_number: String,
    pub period: String,
    pub opening_balance_cents: i64,
    pub transactions: Vec<Transaction>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}

pub struct Transaction {
    pub date: String,
    pub description: String,
    pub amount_cents: i64,  // Positive = credit, negative = debit
    pub balance_cents: i64,
}
```

### 15.8 create_purchase_order

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreatePurchaseOrderInput {
    pub output: String,
    pub buyer: String,
    pub vendor: String,
    pub po_number: Option<String>,
    pub items: Vec<InvoiceItem>,
    pub delivery_terms: Option<String>,
    pub payment_terms: Option<String>,
    pub ship_to: Option<String>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}
```

### 15.9 create_contract

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateContractInput {
    pub output: String,
    pub title: String,
    pub parties: Vec<Party>,
    pub effective_date: String,
    pub clauses: Vec<Clause>,
    pub governing_law: Option<String>,
    pub signatures: Vec<SignatureBlock>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}

pub struct Party { pub name: String, pub role: Option<String>, pub address: Option<String> }
pub struct Clause { pub title: String, pub body: String, pub sub_clauses: Option<Vec<String>> }
pub struct SignatureBlock { pub name: String, pub title: Option<String>, pub company: Option<String> }
```

### 15.10 create_report

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateReportInput {
    pub output: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub sections: Vec<ReportSection>,
    pub include_toc: Option<bool>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}

pub struct ReportSection {
    pub heading: String,
    pub body: Option<String>,
    pub table: Option<TableData>,
    pub subsections: Option<Vec<ReportSection>>,
}
```

### 15.11 create_proposal

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateProposalInput {
    pub output: String,
    pub company: String,
    pub client: String,
    pub title: String,
    pub sections: Vec<Section>,
    pub pricing: Option<Vec<InvoiceItem>>,
    pub timeline: Option<Vec<Milestone>>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}

pub struct Milestone { pub phase: String, pub description: String, pub duration: String }
```

### 15.12 create_letter

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateLetterInput {
    pub output: String,
    pub from: LetterParty,
    pub to: LetterParty,
    pub subject: Option<String>,
    pub date: Option<String>,
    pub body: String,
    pub signature: Option<String>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}

pub struct LetterParty { pub name: String, pub company: Option<String>, pub address: Option<String> }
```

### 15.13 create_certificate

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateCertificateInput {
    pub output: String,
    pub recipient: String,
    pub title: String,
    pub description: Option<String>,
    pub issuer: String,
    pub date: Option<String>,
    pub certificate_id: Option<String>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}
```

### 15.14 create_pdf_packet

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreatePacketInput {
    pub output: String,
    pub title: String,
    pub cover_page: Option<CoverPage>,
    pub include_toc: Option<bool>,
    pub documents: Vec<PacketDocument>,
    pub bates_prefix: Option<String>,
    pub page_numbers: Option<bool>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}

pub struct CoverPage { pub title: String, pub subtitle: Option<String>, pub prepared_by: Option<String>, pub date: Option<String> }
pub struct PacketDocument { pub path: String, pub label: Option<String> }
```

---

## 16. Tool Specifications — Pillar 8: Page & File Operations

### 16.1 merge_pdfs

```rust
#[derive(Deserialize, JsonSchema)]
pub struct MergePdfsInput {
    pub pdf_paths: Vec<String>,
    pub output: String,
    pub add_bookmarks: Option<bool>,  // Bookmark per source file (default: true)
}
```

**Implementation:** `lopdf::Document::load()` each, merge page trees, update xref.

### 16.2 split_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SplitPdfInput {
    pub pdf_path: String,
    pub ranges: String,        // "1-5,10-15" or "each" for one file per page
    pub output_dir: String,
}
// Output: list of generated file paths
```

### 16.3 split_by_bookmarks

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SplitByBookmarksInput {
    pub pdf_path: String,
    pub output_dir: String,
    pub level: Option<u32>,  // Bookmark depth to split at (default: 1)
}
```

### 16.4 rotate_pages

```rust
#[derive(Deserialize, JsonSchema)]
pub struct RotatePagesInput {
    pub pdf_path: String,
    pub output: String,
    pub pages: Option<String>,   // "1,3,5" or "all" (default: "all")
    pub degrees: u32,            // 90, 180, 270
}
```

**Implementation:** Modify `/Rotate` entry in page dictionary.

### 16.5 crop_pages

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CropPagesInput {
    pub pdf_path: String,
    pub output: String,
    pub pages: Option<String>,
    pub crop_box: [f32; 4],  // [left, bottom, right, top] in points
}
```

**Implementation:** Set `/CropBox` on page dictionaries.

### 16.6 resize_pages

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ResizePagesInput {
    pub pdf_path: String,
    pub output: String,
    pub page_size: String,  // "a4"|"letter"|"legal"|"a3"|custom "WxH"
}
```

### 16.7 reorder_pages

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ReorderPagesInput {
    pub pdf_path: String,
    pub output: String,
    pub order: Vec<u32>,  // [3, 1, 2, 4] — new page order
}
```

### 16.8 delete_pages

```rust
#[derive(Deserialize, JsonSchema)]
pub struct DeletePagesInput {
    pub pdf_path: String,
    pub output: String,
    pub pages: Vec<u32>,  // Pages to remove
}
```

### 16.9 insert_pages

```rust
#[derive(Deserialize, JsonSchema)]
pub struct InsertPagesInput {
    pub pdf_path: String,
    pub source_pdf: String,
    pub output: String,
    pub position: u32,           // Insert after this page (0 = beginning)
    pub source_pages: Option<String>,  // Pages from source to insert
}
```

### 16.10 compress_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CompressPdfInput {
    pub pdf_path: String,
    pub output: String,
    pub quality: Option<String>,  // "low"|"medium"|"high" (default: "medium")
}

// Output:
{
    "output": "/path/to/compressed.pdf",
    "original_size": 2450000,
    "compressed_size": 890000,
    "reduction_pct": 63.7
}
```

**Implementation:** Recompress streams with FlateDecode, downsample images, remove duplicate objects.

### 16.11 linearize_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct LinearizeInput {
    pub pdf_path: String,
    pub output: String,
}
// Optimizes for web streaming (first page loads fast)
```

### 16.12 add_watermark

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AddWatermarkInput {
    pub pdf_path: String,
    pub output: String,
    pub text: Option<String>,       // Text watermark (e.g., "CONFIDENTIAL")
    pub image: Option<String>,      // Image watermark path
    pub opacity: Option<f32>,       // 0.0-1.0 (default: 0.3)
    pub rotation: Option<f32>,      // Degrees (default: 45)
    pub pages: Option<String>,      // Default: "all"
}
```

---

## 17. Tool Specifications — Pillar 9: Headers, Footers & Numbering

### 17.1 add_page_numbers

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AddPageNumbersInput {
    pub pdf_path: String,
    pub output: String,
    pub format: Option<String>,     // "arabic"|"roman"|"alpha" (default: "arabic")
    pub position: Option<String>,   // "bottom-center"|"bottom-right"|"top-right" (default: "bottom-center")
    pub start_page: Option<u32>,    // Skip first N pages (default: 1)
    pub start_number: Option<u32>,  // Start counting from (default: 1)
    pub prefix: Option<String>,     // e.g., "Page "
    pub suffix: Option<String>,     // e.g., " of {total}"
}
```

### 17.2 add_bates_numbers

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AddBatesInput {
    pub pdf_path: String,
    pub output: String,
    pub prefix: String,             // e.g., "ZAVORA"
    pub start: u32,                 // Starting number
    pub digits: Option<u32>,        // Zero-pad to N digits (default: 6)
    pub suffix: Option<String>,
    pub position: Option<String>,   // Default: "bottom-right"
    pub font_size: Option<f32>,     // Default: 8.0
}
// Output: { "output": "...", "range": "ZAVORA000001 - ZAVORA000012" }
```

### 17.3 add_headers_footers

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AddHeadersFootersInput {
    pub pdf_path: String,
    pub output: String,
    pub header_left: Option<String>,
    pub header_center: Option<String>,
    pub header_right: Option<String>,
    pub footer_left: Option<String>,
    pub footer_center: Option<String>,
    pub footer_right: Option<String>,
    pub font_size: Option<f32>,
    pub skip_first: Option<bool>,   // Skip first page (default: false)
}
// Dynamic fields: {page}, {total}, {date}, {title}, {author}
```

### 17.4 add_table_of_contents

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AddTocInput {
    pub pdf_path: String,
    pub output: String,
    pub title: Option<String>,      // Default: "Table of Contents"
    pub max_depth: Option<u32>,     // Heading levels to include (default: 3)
    pub position: Option<String>,   // "beginning"|"after_cover" (default: "beginning")
}
```

**Implementation:** Extract heading structure from bookmarks or text analysis → generate TOC page(s) with page references → insert at position.

---

## 18. Tool Specifications — Pillar 10: Forms & Signatures

### 18.1 detect_form_fields

```rust
#[derive(Deserialize, JsonSchema)]
pub struct DetectFormFieldsInput { pub pdf_path: String }

// Output:
{
    "fields": [
        { "name": "first_name", "type": "text", "value": "", "required": true, "page": 1, "bbox": [72, 600, 250, 620] },
        { "name": "agree", "type": "checkbox", "value": "Off", "required": true, "page": 2, "bbox": [72, 400, 90, 418] }
    ],
    "field_count": 12,
    "filled_count": 3
}
```

**Implementation:** Parse `/AcroForm` → `/Fields` array. Each field: `/T` (name), `/FT` (type: Tx/Btn/Ch/Sig), `/V` (value), `/Ff` (flags for required).

### 18.2 describe_form

```rust
// Output: human-readable description
"This is a 2-page employment application form with 12 fields:
- Page 1: Personal info (name, email, phone, address)
- Page 2: Employment history (3 entries), education, signature
Required fields: name, email, signature"
```

### 18.3 form_to_schema

```rust
// Output: JSON Schema for the form
{
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "properties": {
        "first_name": { "type": "string" },
        "last_name": { "type": "string" },
        "email": { "type": "string", "format": "email" }
    },
    "required": ["first_name", "last_name", "email"]
}
```

### 18.4 get_form_data

```rust
// Output: current field values
{ "first_name": "James", "last_name": "", "email": "james@zavora.ai", "agree": "Off" }
```

### 18.5 fill_form

```rust
#[derive(Deserialize, JsonSchema)]
pub struct FillFormInput {
    pub pdf_path: String,
    pub output: String,
    pub field_values: serde_json::Value,  // { "first_name": "James", "agree": "Yes" }
}
```

**Implementation:** Load doc, find each field by `/T` name, set `/V` value, update `/AP` (appearance).

### 18.6 fill_form_from_text (feature: ai)

```rust
#[derive(Deserialize, JsonSchema)]
pub struct FillFormFromTextInput {
    pub pdf_path: String,
    pub output: String,
    pub user_text: String,  // "My name is James Karanja, email james@zavora.ai, I agree to terms"
}
// AI maps unstructured text to form fields
```

### 18.7 validate_form

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ValidateFormInput {
    pub pdf_path: String,
    pub rules: Option<serde_json::Value>,  // Custom validation rules
}

// Output:
{
    "valid": false,
    "missing_required": ["signature", "date"],
    "invalid_format": [{ "field": "email", "value": "not-an-email", "expected": "email format" }],
    "completion_pct": 75
}
```

### 18.8 flatten_form

```rust
#[derive(Deserialize, JsonSchema)]
pub struct FlattenFormInput {
    pub pdf_path: String,
    pub output: String,
}
// Burns field values into page content, removes interactivity
```

**Implementation:** For each field, render value into page content stream at field position, then remove `/AcroForm`.

### 18.9 create_fillable_form

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateFillableFormInput {
    pub output: String,
    pub title: String,
    pub fields: Vec<FormFieldDef>,
    pub style: Option<String>,
    pub branding: Option<BrandingInput>,
}

pub struct FormFieldDef {
    pub name: String,
    pub label: String,
    pub field_type: String,  // "text"|"checkbox"|"radio"|"dropdown"|"date"|"signature"
    pub required: bool,
    pub options: Option<Vec<String>>,  // For dropdown/radio
    pub placeholder: Option<String>,
}
```

### 18.10 add_signature_field

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AddSignatureFieldInput {
    pub pdf_path: String,
    pub output: String,
    pub page: u32,
    pub bbox: [f32; 4],         // [x1, y1, x2, y2]
    pub signer_label: String,   // "Signer 1", "Client", etc.
}
```

### 18.11 sign_pdf (feature: signatures)

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SignPdfInput {
    pub pdf_path: String,
    pub output: String,
    pub certificate_path: String,  // PEM or PKCS#12
    pub private_key_path: Option<String>,
    pub password: Option<String>,
    pub reason: Option<String>,
    pub location: Option<String>,
    pub visible: Option<bool>,     // Show signature appearance (default: true)
    pub page: Option<u32>,
    pub bbox: Option<[f32; 4]>,
}
```

**Implementation:** Use `openssl` to create PKCS#7 detached signature, embed in PDF `/Sig` dictionary.

### 18.12 validate_signatures

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ValidateSignaturesInput { pub pdf_path: String }

// Output:
{
    "signatures": [
        {
            "signer": "James Karanja",
            "date": "2026-05-26T10:30:00Z",
            "valid": true,
            "covers_whole_document": true,
            "certificate": { "issuer": "DigiCert", "expires": "2027-01-15" },
            "tampered": false
        }
    ],
    "document_integrity": "intact"
}
```

---

## 19. Tool Specifications — Pillar 11: Security & Compliance

### 19.1 scan_sensitive_data

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ScanSensitiveInput {
    pub pdf_path: String,
    pub categories: Option<Vec<String>>,  // ["email", "phone", "ssn", "credit_card", "name", "address", "id_number"]
}

// Output:
{
    "findings": [
        { "category": "email", "value": "j***@zavora.ai", "page": 1, "count": 2 },
        { "category": "phone", "value": "+254***789", "page": 1, "count": 1 },
        { "category": "national_id", "value": "***4567", "page": 2, "count": 1 }
    ],
    "total_findings": 4,
    "risk_level": "high"
}
```

**Implementation:** Extract text, apply regex patterns for each category:
- Email: `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}`
- Phone: `\+?[\d\s\-\(\)]{7,15}`
- SSN: `\d{3}-\d{2}-\d{4}`
- Credit card: `\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}`
- National ID (Kenya): `\d{7,8}`

### 19.2 preview_redactions

```rust
#[derive(Deserialize, JsonSchema)]
pub struct PreviewRedactionsInput {
    pub pdf_path: String,
    pub rules: RedactionRules,
}

pub struct RedactionRules {
    pub categories: Option<Vec<String>>,  // Auto-detect categories
    pub patterns: Option<Vec<String>>,    // Custom regex patterns
    pub terms: Option<Vec<String>>,       // Exact terms to redact
}

// Output: what WOULD be redacted (non-destructive)
{
    "preview": [
        { "page": 1, "text": "james@zavora.ai", "category": "email", "bbox": [100, 500, 250, 515] }
    ],
    "total_redactions": 8
}
```

### 19.3 redact_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct RedactPdfInput {
    pub pdf_path: String,
    pub output: String,
    pub rules: RedactionRules,
    pub replacement: Option<String>,  // Default: "█████" (black box)
    pub remove_metadata: Option<bool>,  // Default: true
}

// Output:
{
    "output": "/path/to/redacted.pdf",
    "redactions_applied": 8,
    "categories_removed": ["email", "phone"],
    "metadata_stripped": true,
    "sha256": "abc123..."
}
```

**Implementation:** TRUE redaction — not just visual overlay:
1. Find text positions matching rules
2. Remove text operators from content stream
3. Place black rectangle over area
4. Remove from any text extraction cache
5. Strip metadata if requested
6. Verify by re-extracting text

### 19.4 verify_redaction

```rust
#[derive(Deserialize, JsonSchema)]
pub struct VerifyRedactionInput {
    pub pdf_path: String,
    pub expected_removed: Vec<String>,  // Terms that should NOT appear
}

// Output:
{
    "verified": true,
    "text_search_passed": true,
    "object_scan_passed": true,
    "metadata_scan_passed": true,
    "hidden_content_scan_passed": true
}
```

### 19.5 sanitize_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SanitizePdfInput {
    pub pdf_path: String,
    pub output: String,
    pub policy: Option<String>,  // "standard"|"strict"|"paranoid" (default: "standard")
}
// standard: remove JS, actions, embedded files
// strict: + remove comments, hidden layers, form data
// paranoid: + remove all metadata, flatten everything
```

### 19.6 remove_metadata

```rust
#[derive(Deserialize, JsonSchema)]
pub struct RemoveMetadataInput {
    pub pdf_path: String,
    pub output: String,
    pub fields: Option<Vec<String>>,  // Specific fields, or all if omitted
}
```

### 19.7 detect_active_content

```rust
// Output:
{
    "javascript": [{ "page": 3, "trigger": "page_open", "code_length": 245 }],
    "actions": [{ "page": 1, "type": "URI", "target": "https://..." }],
    "embedded_files": [{ "name": "payload.exe", "size": 45000 }],
    "risk_level": "high"
}
```

### 19.8 remove_active_content

```rust
#[derive(Deserialize, JsonSchema)]
pub struct RemoveActiveContentInput {
    pub pdf_path: String,
    pub output: String,
}
// Strips all JS, actions, embedded executables
```

### 19.9 encrypt_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct EncryptPdfInput {
    pub pdf_path: String,
    pub output: String,
    pub user_password: Option<String>,   // Password to open
    pub owner_password: String,          // Password for full access
    pub permissions: Option<Permissions>,
}

pub struct Permissions {
    pub print: bool,
    pub copy: bool,
    pub modify: bool,
    pub annotate: bool,
    pub fill_forms: bool,
}
```

### 19.10 decrypt_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct DecryptPdfInput {
    pub pdf_path: String,
    pub output: String,
    pub password: String,
}
```

### 19.11 set_permissions

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SetPermissionsInput {
    pub pdf_path: String,
    pub output: String,
    pub owner_password: String,
    pub permissions: Permissions,
}
```

### 19.12 hash_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct HashPdfInput {
    pub pdf_path: String,
    pub algorithm: Option<String>,  // "sha256"|"sha512"|"md5" (default: "sha256")
}
// Output: { "algorithm": "sha256", "hash": "abc123...", "file_size": 245000 }
```

**Implementation:** `sha2::Sha256::digest(std::fs::read(path))`.

---

## 20. Tool Specifications — Pillar 12: Accessibility & Standards

### 20.1 audit_accessibility

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AuditAccessibilityInput { pub pdf_path: String }

// Output:
{
    "score": 45,  // 0-100
    "level": "poor",  // poor|fair|good|excellent
    "issues": [
        { "severity": "critical", "code": "NO_TAGS", "description": "Document has no structure tags", "page": null },
        { "severity": "major", "code": "MISSING_ALT_TEXT", "description": "3 images missing alt text", "pages": [1, 4, 7] },
        { "severity": "minor", "code": "NO_LANG", "description": "Document language not set", "page": null }
    ],
    "summary": { "critical": 1, "major": 2, "minor": 3 },
    "pdf_ua_compliant": false
}
```

**Implementation:** Check for: `/MarkInfo`, `/StructTreeRoot`, `/Lang`, alt text on images, reading order, heading hierarchy, table headers.

### 20.2 generate_accessibility_report

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AccessibilityReportInput {
    pub pdf_path: String,
    pub output: String,  // Generates a PDF report
}
// Creates a human-readable remediation guide as PDF
```

### 20.3 detect_reading_order_issues

```rust
// Output:
{
    "issues": [
        { "page": 2, "description": "Multi-column content may read incorrectly", "elements_affected": 5 }
    ]
}
```

### 20.4 set_reading_order

```rust
#[derive(Deserialize, JsonSchema)]
pub struct SetReadingOrderInput {
    pub pdf_path: String,
    pub output: String,
    pub order_map: Vec<ReadingOrderEntry>,
}

pub struct ReadingOrderEntry { pub page: u32, pub element_indices: Vec<u32> }
```

### 20.5 detect_missing_alt_text

```rust
// Output:
{
    "images_without_alt": [
        { "page": 1, "index": 0, "bbox": [72, 600, 300, 750] },
        { "page": 4, "index": 2, "bbox": [100, 200, 400, 400] }
    ],
    "total_images": 8,
    "missing_count": 3
}
```

### 20.6 add_alt_text

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AddAltTextInput {
    pub pdf_path: String,
    pub output: String,
    pub alt_texts: Vec<AltTextEntry>,
}

pub struct AltTextEntry { pub page: u32, pub image_index: u32, pub alt_text: String }
```

### 20.7 auto_tag_pdf

```rust
#[derive(Deserialize, JsonSchema)]
pub struct AutoTagInput {
    pub pdf_path: String,
    pub output: String,
}

// Output:
{
    "output": "/path/to/tagged.pdf",
    "tags_added": { "headings": 5, "paragraphs": 23, "tables": 2, "lists": 3, "figures": 4 },
    "document_language_set": "en"
}
```

**Implementation:** Analyze text sizes/weights to detect headings. Detect tables by alignment. Wrap in structure tree with `/H1`, `/P`, `/Table`, `/L`, `/Figure` tags.

### 20.8 validate_pdf_a

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ValidatePdfAInput {
    pub pdf_path: String,
    pub level: Option<String>,  // "1b"|"2b"|"3b" (default: "2b")
}

// Output:
{
    "compliant": false,
    "level_tested": "PDF/A-2b",
    "violations": [
        { "code": "FONT_NOT_EMBEDDED", "description": "Helvetica not embedded", "severity": "error" },
        { "code": "NO_OUTPUT_INTENT", "description": "Missing output intent", "severity": "error" }
    ],
    "warnings": []
}
```

### 20.9 convert_to_pdf_a

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ConvertToPdfAInput {
    pub pdf_path: String,
    pub output: String,
    pub level: Option<String>,  // "1b"|"2b"|"3b" (default: "2b")
}
// Embeds fonts, adds output intent, sets metadata, removes JS
```

### 20.10 validate_pdf_ua

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ValidatePdfUaInput { pub pdf_path: String }

// Output: similar to audit_accessibility but specifically against PDF/UA-1 standard
{
    "compliant": false,
    "standard": "PDF/UA-1",
    "violations": [...],
    "matterhorn_protocol_checks": { "passed": 28, "failed": 3, "total": 31 }
}
```

---

## 21. Tool Specifications — Pillar 13: Batch & Workflows

### 21.1 batch_process

```rust
#[derive(Deserialize, JsonSchema)]
pub struct BatchProcessInput {
    pub folder_path: String,
    pub operation: String,          // Any single-document tool name
    pub operation_args: Option<serde_json::Value>,  // Additional args for the tool
    pub output_dir: Option<String>,
    pub recursive: Option<bool>,    // Default: false
    pub filter: Option<String>,     // Glob pattern, e.g., "*.pdf"
    pub parallel: Option<u32>,      // Max concurrent (default: 4)
}

// Output:
{
    "processed": 42,
    "succeeded": 40,
    "failed": 2,
    "failures": [
        { "file": "corrupted.pdf", "error": "Invalid PDF structure" }
    ],
    "output_dir": "/path/to/output/"
}
```

### 21.2 create_workflow

```rust
#[derive(Deserialize, JsonSchema)]
pub struct CreateWorkflowInput {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
}

pub struct WorkflowStep {
    pub tool: String,                    // Tool name
    pub args: Option<serde_json::Value>, // Static args
    pub condition: Option<String>,       // e.g., "classification.is_scanned == true"
    pub output_var: Option<String>,      // Store result as variable
}

// Output: { "workflow_id": "invoice_pipeline_v1", "steps": 4, "saved": true }
```

**Implementation:** Serialize workflow to JSON, store in `~/.mcp-pdf/workflows/`.

### 21.3 run_workflow

```rust
#[derive(Deserialize, JsonSchema)]
pub struct RunWorkflowInput {
    pub workflow_name: String,
    pub input_paths: Vec<String>,  // Files or directories
    pub output_dir: Option<String>,
}

// Output:
{
    "workflow": "invoice_pipeline_v1",
    "files_processed": 25,
    "results": [
        { "file": "inv_001.pdf", "steps_completed": 4, "status": "success", "output": "..." }
    ],
    "duration_ms": 12500
}
```

### 21.4 dry_run_workflow

```rust
#[derive(Deserialize, JsonSchema)]
pub struct DryRunWorkflowInput {
    pub workflow_name: String,
    pub input_paths: Vec<String>,
}

// Output: preview of what would happen (no files modified)
{
    "workflow": "invoice_pipeline_v1",
    "files_matched": 25,
    "planned_steps": [
        { "file": "inv_001.pdf", "steps": ["classify_pdf", "ocr_pdf", "parse_invoice"] }
    ]
}
```

### 21.5 deduplicate_pdfs

```rust
#[derive(Deserialize, JsonSchema)]
pub struct DeduplicateInput {
    pub folder_path: String,
    pub mode: Option<String>,  // "hash"|"content"|"both" (default: "hash")
    pub action: Option<String>,  // "report"|"move"|"delete" (default: "report")
}

// Output:
{
    "duplicates": [
        { "original": "report_v1.pdf", "duplicates": ["report_copy.pdf", "report (1).pdf"], "hash": "abc..." }
    ],
    "total_duplicates": 5,
    "space_recoverable": "12.5 MB"
}
```

**Implementation:** SHA-256 hash each file. For "content" mode, extract text and compare.

### 21.6 generate_report

```rust
#[derive(Deserialize, JsonSchema)]
pub struct GenerateReportInput {
    pub data: serde_json::Value,  // Results from batch_process or run_workflow
    pub output: String,
    pub format: Option<String>,   // "json"|"csv"|"pdf" (default: "json")
}
```

---

## 22. Vertical Template System (verticals/mod.rs)

### Template Registry

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerticalTemplate {
    pub vertical: String,
    pub name: String,
    pub description: String,
    pub style: String,                    // Base style to use
    pub schema: TemplateSchema,           // Field validation
    pub layout: TemplateLayout,           // How to render
    pub compliance: Vec<String>,          // Auto-applied rules
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplateSchema {
    pub fields: Vec<FieldDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub label: String,                    // Display label
    pub validation: Option<String>,       // Regex or format
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Number,
    Date,
    Currency,
    Array(Box<FieldType>),
    Object(Vec<FieldDef>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TemplateLayout {
    pub header_fields: Vec<String>,       // Fields shown in header area
    pub body_sections: Vec<LayoutSection>,
    pub footer_fields: Vec<String>,
    pub signature_blocks: bool,
    pub page_numbers: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayoutSection {
    pub section_type: SectionType,
    pub title: Option<String>,
    pub fields: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SectionType {
    KeyValue,       // Label: Value pairs
    Table,          // Array field rendered as table
    Paragraph,      // Long text field
    SignatureBlock,  // Signature lines
    Separator,      // Visual divider
}

/// Get all templates for a vertical
pub fn get_vertical_templates(vertical: &str) -> Vec<VerticalTemplate> { ... }

/// Get a specific template
pub fn get_template(vertical: &str, template: &str) -> Option<VerticalTemplate> { ... }

/// Validate data against template schema
pub fn validate_data(template: &VerticalTemplate, data: &Value) -> Result<(), Vec<String>> { ... }

/// List all verticals
pub fn list_verticals() -> Vec<VerticalInfo> { ... }

/// List all templates across all verticals
pub fn list_all_templates() -> Vec<(String, String, String)> { ... } // (vertical, name, description)
```

### Example: Healthcare Lab Report Template

```rust
// verticals/healthcare.rs
pub fn lab_report_template() -> VerticalTemplate {
    VerticalTemplate {
        vertical: "healthcare".into(),
        name: "lab_report".into(),
        description: "Laboratory test results report".into(),
        style: "medical".into(),
        schema: TemplateSchema {
            fields: vec![
                FieldDef { name: "patient".into(), field_type: FieldType::Object(vec![
                    FieldDef { name: "name".into(), field_type: FieldType::String, required: true, label: "Patient Name".into(), validation: None },
                    FieldDef { name: "id".into(), field_type: FieldType::String, required: true, label: "MRN".into(), validation: None },
                    FieldDef { name: "dob".into(), field_type: FieldType::Date, required: true, label: "Date of Birth".into(), validation: None },
                ]), required: true, label: "Patient".into(), validation: None },
                FieldDef { name: "ordering_physician".into(), field_type: FieldType::String, required: true, label: "Ordering Physician".into(), validation: None },
                FieldDef { name: "tests".into(), field_type: FieldType::Array(Box::new(FieldType::Object(vec![
                    FieldDef { name: "name".into(), field_type: FieldType::String, required: true, label: "Test".into(), validation: None },
                    FieldDef { name: "result".into(), field_type: FieldType::String, required: true, label: "Result".into(), validation: None },
                    FieldDef { name: "reference".into(), field_type: FieldType::String, required: false, label: "Reference Range".into(), validation: None },
                    FieldDef { name: "flag".into(), field_type: FieldType::String, required: false, label: "Flag".into(), validation: None },
                ]))), required: true, label: "Tests".into(), validation: None },
            ],
        },
        layout: TemplateLayout {
            header_fields: vec!["patient.name".into(), "patient.id".into(), "ordering_physician".into()],
            body_sections: vec![
                LayoutSection { section_type: SectionType::Table, title: Some("Test Results".into()), fields: vec!["tests".into()] },
            ],
            footer_fields: vec![],
            signature_blocks: false,
            page_numbers: true,
        },
        compliance: vec!["hipaa".into(), "pdf_a".into()],
    }
}
```

### Compliance Auto-Application

```rust
pub fn apply_compliance(doc: &mut PdfEngine, rules: &[String]) {
    for rule in rules {
        match rule.as_str() {
            "hipaa" => {
                // Mark PII fields as redactable
                // Add confidentiality notice in footer
            }
            "pdf_ua" => {
                // Ensure structure tags
                // Set document language
                // Add alt text placeholders
            }
            "pdf_a" => {
                // Embed all fonts
                // Add output intent
                // Remove JavaScript
            }
            "ferpa" => {
                // Mark student data as redactable
                // Add FERPA notice
            }
            _ => {}
        }
    }
}
```

---

## 23. Server Registration (server.rs)

```rust
use rmcp::{tool, tool_router, schemars};
use rmcp::handler::server::wrapper::Parameters;

#[derive(Clone)]
pub struct PdfServer {
    config: PdfConfig,
}

#[tool_router(server_handler)]
impl PdfServer {
    // === PILLAR 1: Inspect (8 tools) ===
    #[tool(description = "Full structural profile of a PDF")]
    async fn inspect_pdf(&self, Parameters(input): Parameters<InspectPdfInput>) -> String { ... }

    #[tool(description = "Classify document type: invoice, contract, form, scan, report, letter")]
    async fn classify_pdf(&self, Parameters(input): Parameters<ClassifyPdfInput>) -> String { ... }

    #[tool(description = "Check PDF health: corruption, xref errors, broken objects")]
    async fn health_check_pdf(&self, Parameters(input): Parameters<HealthCheckInput>) -> String { ... }

    #[tool(description = "Detect features: forms, tags, signatures, JavaScript, embedded files")]
    async fn detect_features(&self, Parameters(input): Parameters<DetectFeaturesInput>) -> String { ... }

    #[tool(description = "Rate extraction difficulty: simple to extreme")]
    async fn profile_complexity(&self, Parameters(input): Parameters<ProfileComplexityInput>) -> String { ... }

    #[tool(description = "Get page count")]
    async fn get_page_count(&self, Parameters(input): Parameters<GetPageCountInput>) -> String { ... }

    #[tool(description = "Quick metadata: pages, size, version, encryption, title, author")]
    async fn get_info(&self, Parameters(input): Parameters<GetInfoInput>) -> String { ... }

    #[tool(description = "Repair corrupted PDF: rebuild xref, fix streams")]
    async fn repair_pdf(&self, Parameters(input): Parameters<RepairInput>) -> String { ... }

    // === PILLAR 2: Extract (14 tools) ===
    #[tool(description = "Extract all text from PDF")]
    async fn extract_text(&self, Parameters(input): Parameters<ExtractTextInput>) -> String { ... }

    #[tool(description = "Extract text with page numbers, bounding boxes, block types")]
    async fn extract_text_structured(&self, Parameters(input): Parameters<ExtractTextStructuredInput>) -> String { ... }

    #[tool(description = "Extract text from a specific page")]
    async fn extract_page_text(&self, Parameters(input): Parameters<ExtractPageTextInput>) -> String { ... }

    #[tool(description = "Extract document layout: headings, paragraphs, columns")]
    async fn extract_layout(&self, Parameters(input): Parameters<ExtractLayoutInput>) -> String { ... }

    #[tool(description = "Extract tables as JSON arrays with headers and rows")]
    async fn extract_tables(&self, Parameters(input): Parameters<ExtractTablesInput>) -> String { ... }

    #[tool(description = "Extract key-value pairs (label → value)")]
    async fn extract_key_values(&self, Parameters(input): Parameters<ExtractKeyValuesInput>) -> String { ... }

    #[tool(description = "Extract embedded images")]
    async fn extract_images(&self, Parameters(input): Parameters<ExtractImagesInput>) -> String { ... }

    #[tool(description = "Extract figures and diagrams with captions")]
    async fn extract_figures(&self, Parameters(input): Parameters<ExtractFiguresInput>) -> String { ... }

    #[tool(description = "Extract annotations: comments, highlights, stamps")]
    async fn extract_annotations(&self, Parameters(input): Parameters<ExtractAnnotationsInput>) -> String { ... }

    #[tool(description = "Extract bookmark/outline tree")]
    async fn extract_bookmarks(&self, Parameters(input): Parameters<ExtractBookmarksInput>) -> String { ... }

    #[tool(description = "Extract embedded file attachments")]
    async fn extract_attachments(&self, Parameters(input): Parameters<ExtractAttachmentsInput>) -> String { ... }

    #[tool(description = "Detect and decode QR codes and barcodes")]
    async fn extract_barcodes(&self, Parameters(input): Parameters<ExtractBarcodesInput>) -> String { ... }

    #[tool(description = "Extract document metadata: title, author, dates, XMP")]
    async fn extract_metadata(&self, Parameters(input): Parameters<ExtractMetadataInput>) -> String { ... }

    #[tool(description = "Determine human reading order of page elements")]
    async fn extract_reading_order(&self, Parameters(input): Parameters<ExtractReadingOrderInput>) -> String { ... }

    // === PILLAR 7: Generate (14 tools) ===
    #[tool(description = "Create a general PDF document from sections")]
    async fn create_document(&self, Parameters(input): Parameters<CreateDocumentInput>) -> String { ... }

    #[tool(description = "Render a vertical/custom template with data")]
    async fn create_from_template(&self, Parameters(input): Parameters<CreateFromTemplateInput>) -> String { ... }

    #[tool(description = "Define a reusable document template")]
    async fn create_template(&self, Parameters(input): Parameters<CreateTemplateInput>) -> String { ... }

    #[tool(description = "Generate professional invoice with line items, tax, logo")]
    async fn create_invoice(&self, Parameters(input): Parameters<CreateInvoiceInput>) -> String { ... }

    #[tool(description = "Generate payment receipt")]
    async fn create_receipt(&self, Parameters(input): Parameters<CreateReceiptInput>) -> String { ... }

    #[tool(description = "Generate price quote/estimate")]
    async fn create_quote(&self, Parameters(input): Parameters<CreateQuoteInput>) -> String { ... }

    #[tool(description = "Generate financial/account statement")]
    async fn create_statement(&self, Parameters(input): Parameters<CreateStatementInput>) -> String { ... }

    #[tool(description = "Generate purchase order")]
    async fn create_purchase_order(&self, Parameters(input): Parameters<CreatePurchaseOrderInput>) -> String { ... }

    #[tool(description = "Generate contract with clauses and signature blocks")]
    async fn create_contract(&self, Parameters(input): Parameters<CreateContractInput>) -> String { ... }

    #[tool(description = "Generate multi-section report")]
    async fn create_report(&self, Parameters(input): Parameters<CreateReportInput>) -> String { ... }

    #[tool(description = "Generate client proposal with pricing and timeline")]
    async fn create_proposal(&self, Parameters(input): Parameters<CreateProposalInput>) -> String { ... }

    #[tool(description = "Generate business letter with letterhead")]
    async fn create_letter(&self, Parameters(input): Parameters<CreateLetterInput>) -> String { ... }

    #[tool(description = "Generate certificate of achievement/completion")]
    async fn create_certificate(&self, Parameters(input): Parameters<CreateCertificateInput>) -> String { ... }

    #[tool(description = "Create PDF packet: cover + TOC + merged documents + Bates numbers")]
    async fn create_pdf_packet(&self, Parameters(input): Parameters<CreatePacketInput>) -> String { ... }

    // === PILLAR 8: Manipulate (12 tools) ===
    #[tool(description = "Merge multiple PDFs into one")]
    async fn merge_pdfs(&self, Parameters(input): Parameters<MergePdfsInput>) -> String { ... }

    #[tool(description = "Split PDF by page ranges")]
    async fn split_pdf(&self, Parameters(input): Parameters<SplitPdfInput>) -> String { ... }

    #[tool(description = "Split PDF into files by bookmark sections")]
    async fn split_by_bookmarks(&self, Parameters(input): Parameters<SplitByBookmarksInput>) -> String { ... }

    #[tool(description = "Rotate pages by 90/180/270 degrees")]
    async fn rotate_pages(&self, Parameters(input): Parameters<RotatePagesInput>) -> String { ... }

    #[tool(description = "Crop pages to bounding box")]
    async fn crop_pages(&self, Parameters(input): Parameters<CropPagesInput>) -> String { ... }

    #[tool(description = "Resize pages to standard or custom dimensions")]
    async fn resize_pages(&self, Parameters(input): Parameters<ResizePagesInput>) -> String { ... }

    #[tool(description = "Reorder pages by index array")]
    async fn reorder_pages(&self, Parameters(input): Parameters<ReorderPagesInput>) -> String { ... }

    #[tool(description = "Delete specified pages")]
    async fn delete_pages(&self, Parameters(input): Parameters<DeletePagesInput>) -> String { ... }

    #[tool(description = "Insert pages from another PDF at position")]
    async fn insert_pages(&self, Parameters(input): Parameters<InsertPagesInput>) -> String { ... }

    #[tool(description = "Compress PDF: reduce file size")]
    async fn compress_pdf(&self, Parameters(input): Parameters<CompressPdfInput>) -> String { ... }

    #[tool(description = "Linearize PDF for fast web streaming")]
    async fn linearize_pdf(&self, Parameters(input): Parameters<LinearizeInput>) -> String { ... }

    #[tool(description = "Add text or image watermark")]
    async fn add_watermark(&self, Parameters(input): Parameters<AddWatermarkInput>) -> String { ... }

    // === PILLAR 9: Numbering (4 tools) ===
    #[tool(description = "Add page numbers (Arabic, Roman, or alpha)")]
    async fn add_page_numbers(&self, Parameters(input): Parameters<AddPageNumbersInput>) -> String { ... }

    #[tool(description = "Add Bates numbering for legal documents")]
    async fn add_bates_numbers(&self, Parameters(input): Parameters<AddBatesInput>) -> String { ... }

    #[tool(description = "Add headers and footers with dynamic fields")]
    async fn add_headers_footers(&self, Parameters(input): Parameters<AddHeadersFootersInput>) -> String { ... }

    #[tool(description = "Generate and insert table of contents")]
    async fn add_table_of_contents(&self, Parameters(input): Parameters<AddTocInput>) -> String { ... }

    // === PILLAR 10: Forms (9 core + 3 signatures) ===
    // ... (all 12 tools registered here)

    // === PILLAR 11: Security (12 tools) ===
    // ... (all 12 tools registered here)

    // === PILLAR 12: Accessibility (10 tools) ===
    // ... (all 10 tools registered here)

    // === PILLAR 13: Batch (6 tools) ===
    // ... (all 6 tools registered here)

    // === CONDITIONAL: OCR (6 tools) — #[cfg(feature = "ocr")] ===
    // ... (registered only when feature enabled)

    // === CONDITIONAL: AI (14 + 9 = 23 tools) — #[cfg(feature = "ai")] ===
    // ... (registered only when feature enabled)

    // === CONDITIONAL: Conversion (10 tools) — #[cfg(feature = "conversion")] ===
    // ... (registered only when feature enabled)
}
```

---

## 24. Testing Strategy

### Unit Tests (per module)

```
tests/
├── core/
│   ├── test_inspect.rs        # Test with fixture PDFs (normal, corrupted, encrypted, scanned)
│   ├── test_extract.rs        # Test text/table/image extraction accuracy
│   ├── test_manipulate.rs     # Test merge/split/rotate produce valid PDFs
│   ├── test_numbering.rs      # Test page numbers, Bates, headers
│   ├── test_forms.rs          # Test field detection, fill, flatten
│   ├── test_security.rs       # Test encrypt/decrypt, redaction verification
│   └── test_accessibility.rs  # Test tag detection, audit scoring
├── generate/
│   ├── test_engine.rs         # Test multi-page, page breaks, tables
│   ├── test_styles.rs         # Test all 8 styles produce valid PDFs
│   ├── test_documents.rs      # Test each of 14 generation tools
│   └── test_branding.rs       # Test logo loading, color parsing
├── verticals/
│   ├── test_healthcare.rs     # Test all 8 healthcare templates
│   ├── test_legal.rs          # Test all 10 legal templates
│   ├── test_finance.rs        # Test all 10 finance templates
│   └── ...                    # One test file per vertical
├── integration/
│   ├── test_mcp_protocol.rs   # Test JSON-RPC tool calls end-to-end
│   ├── test_workflows.rs      # Test batch + workflow execution
│   └── test_feature_flags.rs  # Test graceful degradation
└── fixtures/
    ├── sample_invoice.pdf
    ├── sample_contract.pdf
    ├── scanned_document.pdf
    ├── corrupted.pdf
    ├── encrypted.pdf
    ├── form_fillable.pdf
    ├── tagged_accessible.pdf
    └── multi_column.pdf
```

### Test Matrix

| Style × Document Type | minimal | modern | corporate | stripe | legal | academic | government | finance |
|----------------------|---------|--------|-----------|--------|-------|----------|------------|---------|
| Invoice | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Receipt | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Contract | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Report | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Letter | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Certificate | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

**Total: 8 styles × 14 document types × 10 verticals = 1,120 combinations**
(Tested with representative subset: 8 × 14 = 112 core + 88 vertical templates = 200 test PDFs)

---

## 25. Phased Delivery Schedule

### Phase 1: Core Foundation (v3.0.0) — Weeks 1–3

| Week | Deliverable |
|------|-------------|
| 1 | Project restructure, Cargo.toml, error handling, config system. Pillar 1 (8 inspect tools). |
| 2 | Pillar 2 (14 extract tools). Pillar 8 (12 manipulate tools). |
| 3 | Pillar 9 (4 numbering tools). Style system (4 styles). `create_invoice` + `create_receipt`. Integration tests. |

**Exit criteria:** `cargo build` with default features. 40 tools passing. All produce valid PDFs openable in Preview/Chrome/Acrobat.

### Phase 2: Generation & Security (v3.1.0) — Weeks 4–6

| Week | Deliverable |
|------|-------------|
| 4 | Pillar 7 remaining 12 generation tools. PdfEngine multi-page support. |
| 5 | Pillar 11 (12 security tools). Redaction with verification. |
| 6 | Pillar 10 forms (9 tools). 4 additional styles. Batch_process + create_workflow + run_workflow. |

**Exit criteria:** 78 tools total. All 8 styles working. Redaction verified. Forms fillable.

### Phase 3: AI Intelligence (v3.2.0) — Weeks 7–10

| Week | Deliverable |
|------|-------------|
| 7 | AI provider abstraction. RAG pipeline (chunk, embed, search). |
| 8 | Pillar 4 (14 AI tools): summarize, answer, search, compare, translate. |
| 9 | Pillar 5 (9 IDP tools): parse_invoice, parse_contract, etc. |
| 10 | Testing with real PDFs. Confidence scoring. JSON schema validation. |

**Exit criteria:** 101 tools. AI tools work with Anthropic + OpenAI + Ollama.

### Phase 4: OCR & Conversion (v3.3.0) — Weeks 11–14

| Week | Deliverable |
|------|-------------|
| 11 | Tesseract integration. Pillar 3 (6 OCR tools). |
| 12 | LibreOffice bridge. Pillar 6 first 5 conversion tools. |
| 13 | Pillar 6 remaining 5 conversion tools. markdown_to_pdf with style system. |
| 14 | Integration testing. Feature flag graceful degradation. |

**Exit criteria:** 117 tools. OCR works on scanned PDFs. Office conversion works.

### Phase 5: Accessibility, Signatures, Verticals & Polish (v3.4.0) — Weeks 15–18

| Week | Deliverable |
|------|-------------|
| 15 | Pillar 12 (10 accessibility tools). PDF/UA tagging. |
| 16 | Signatures (3 tools). Remaining batch tools (3). |
| 17 | All 10 verticals (88 templates). Template validation. |
| 18 | Full test suite. README. Documentation. Publish v3.0.0 to crates.io. |

**Exit criteria:** 131 tools. 88 templates. All tests passing. Published.

---

## 26. File Size & Performance Targets

| Metric | Target |
|--------|--------|
| Binary size (core only) | < 15 MB |
| Binary size (full features) | < 50 MB |
| Startup time | < 100ms |
| extract_text (10-page PDF) | < 200ms |
| create_invoice | < 50ms |
| merge_pdfs (10 files) | < 500ms |
| compress_pdf | < 2s |
| OCR (1 page) | < 3s |
| AI summarize (10 pages) | < 5s (network-bound) |

---

## 27. Publishing & Distribution

```toml
# mcp-server.toml
[server]
name = "mcp-pdf"
version = "3.0.0"
description = "The PDF Operating Layer for AI Agents"
author = "Zavora AI"
license = "Apache-2.0"
repository = "https://github.com/zavora-ai/mcp-pdf"

[capabilities]
tools = true
resources = false
prompts = false

[tools]
count = 131
categories = ["inspect", "extract", "ocr", "ai", "idp", "convert", "generate", "manipulate", "numbering", "forms", "security", "accessibility", "batch"]

[features]
core = { tools = 92, deps = "none" }
ocr = { tools = 6, deps = "tesseract 5" }
ai = { tools = 23, deps = "LLM API key" }
conversion = { tools = 10, deps = "LibreOffice" }
signatures = { tools = 3, deps = "OpenSSL" }

[verticals]
count = 10
templates = 88
packs = ["healthcare", "legal", "finance", "government", "enterprise"]
```

---

*End of specification. Total: 131 tools, 88 vertical templates, 8 styles, 10 industry verticals, 5 phases over 18 weeks.*
