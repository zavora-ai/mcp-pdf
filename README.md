# mcp-pdf

[![Crates.io](https://img.shields.io/crates/v/mcp-pdf.svg)](https://crates.io/crates/mcp-pdf)
[![Docs.rs](https://docs.rs/mcp-pdf/badge.svg)](https://docs.rs/mcp-pdf)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://enterprise.adk-rust.com)

The PDF Operating Layer for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) agents. Provides 57 MCP tools for inspecting, extracting, generating, converting, manipulating, securing, and filling PDFs — **pure Rust, zero system dependencies, 17MB binary**.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-pdf/main/docs/assets/architecture.svg" alt="mcp-pdf Architecture" width="800"/>
</p>

## Key Principles

- **Universal PDF coverage** — inspect, extract, generate, convert, manipulate, secure, and fill in one binary.
- **High fidelity** — pdf_oxide for extraction (100% pass rate on 3,830 PDFs), printpdf for generation.
- **Cross-viewer compatible** — all output works in Safari, Preview, Chrome, and Acrobat.
- **Zero system dependencies** — pure Rust. No LibreOffice, no Tesseract, no pdfium required.
- **Local-first** — no cloud calls, no API keys, no data leaves your machine.
- **Registry-ready** — ships with `mcp-server.toml` for automatic ADK-Rust Enterprise onboarding.

## Tools (57)

| Tool | Purpose | Category |
|------|---------|----------|
| `inspect_pdf` | Full structural profile (pages, fonts, forms, signatures) | Inspect |
| `classify_pdf` | Identify document type (invoice, contract, form, scan) | Inspect |
| `health_check_pdf` | Detect corruption, xref errors, broken objects | Inspect |
| `detect_features` | Report forms, tags, signatures, JavaScript, embedded files | Inspect |
| `profile_complexity` | Rate extraction difficulty (1-5 scale) | Inspect |
| `get_page_count` | Number of pages | Inspect |
| `get_info` | Quick metadata (pages, size, version, title, author) | Inspect |
| `repair_pdf` | Rebuild xref, fix streams, recover corrupted PDFs | Inspect |
| `extract_text` | All text from PDF | Extract |
| `extract_page_text` | Text from a specific page | Extract |
| `extract_metadata` | Title, author, dates, creator, producer, XMP | Extract |
| `extract_tables` | Tables as JSON arrays with headers and rows | Extract |
| `extract_images` | Embedded image info (dimensions, color space) | Extract |
| `extract_bookmarks` | Outline/bookmark tree | Extract |
| `extract_annotations` | Comments, highlights, stamps | Extract |
| `extract_key_values` | Label: value pairs from forms | Extract |
| `create_invoice` | Professional invoice (logo, QR code, line items) | Generate |
| `create_receipt` | Payment receipt (4 stamp styles, custom colors) | Generate |
| `create_quote` | Price quote with validity and notes | Generate |
| `create_statement` | Account statement with transactions and balance | Generate |
| `create_purchase_order` | PO with buyer/vendor, terms | Generate |
| `create_letter` | Business letter with letterhead | Generate |
| `create_certificate` | Certificate (5 styles: classic/modern/elegant/academic/minimal) | Generate |
| `create_report` | Multi-section report with headings | Generate |
| `create_contract` | Agreement with numbered clauses and signature blocks | Generate |
| `pdf_to_markdown` | High-fidelity PDF→Markdown (headings, tables, layout) | Convert |
| `pdf_to_html` | PDF→HTML with structure preservation | Convert |
| `pdf_to_json` | PDF→structured JSON (text + markdown per page) | Convert |
| `pdf_to_csv` | Extract tables as CSV | Convert |
| `markdown_to_pdf` | Markdown→styled PDF (comrak parser + printpdf) | Convert |
| `images_to_pdf` | Images→PDF (one per page, auto-scaled) | Convert |
| `merge_pdfs` | Merge multiple PDFs into one | Manipulate |
| `split_pdf` | Extract page ranges | Manipulate |
| `split_by_bookmarks` | Split into files by bookmark sections | Manipulate |
| `rotate_pages` | Rotate pages (90/180/270°) | Manipulate |
| `crop_pages` | Crop to bounding box | Manipulate |
| `reorder_pages` | Reorder by index array | Manipulate |
| `delete_pages` | Remove specified pages | Manipulate |
| `compress_pdf` | Reduce file size (66% typical reduction) | Manipulate |
| `add_watermark` | Text watermark via Form XObject (all viewers) | Manipulate |
| `add_headers_footers` | Dynamic headers/footers ({page}/{total} placeholders) | Manipulate |
| `add_page_numbers` | Arabic, Roman, or alpha numbering | Numbering |
| `add_bates_numbers` | Legal Bates numbering with prefix | Numbering |
| `hash_pdf` | SHA-256 integrity hash | Security |
| `encrypt_pdf` | AES-128 R4 password protection (pure Rust) | Security |
| `decrypt_pdf` | Remove password from owned document | Security |
| `set_permissions` | Control print/copy/edit permissions | Security |
| `scan_sensitive_data` | Detect PII (emails, phones, SSNs, credit cards) | Security |
| `redact_pdf` | True redaction (content removal, not masking) | Security |
| `sanitize_pdf` | Remove JavaScript, actions, embedded files, metadata | Security |
| `remove_metadata` | Strip author, dates, creator, XMP | Security |
| `detect_active_content` | Find JavaScript, actions, embedded executables | Security |
| `detect_form_fields` | List all form fields (name, type, value) | Forms |
| `fill_form` | Fill interactive form fields by name | Forms |
| `flatten_form` | Make fields non-editable | Forms |
| `fill_flat_form` | Overlay text at x,y positions (flat/scanned forms) | Forms |
| `describe_form_layout` | Page dimensions + detected field lines for positioning | Forms |

## Installation

### From crates.io

```bash
cargo install mcp-pdf
```

### From source

```bash
git clone https://github.com/zavora-ai/mcp-pdf
cd mcp-pdf
cargo build --release
```

Binary: `target/release/mcp-pdf` (17MB)

## Configuration

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "pdf": {
      "command": "mcp-pdf"
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "pdf": {
      "command": "mcp-pdf"
    }
  }
}
```

### Kiro

```bash
kiro mcp add pdf -- mcp-pdf
```

## Generation Features

### Invoice with QR Code + Logo

```json
{
  "name": "create_invoice",
  "arguments": {
    "output": "invoice.pdf",
    "company": "Zavora AI",
    "customer": "Acme Corp",
    "invoice_number": "INV-2026-100",
    "logo": "/path/to/logo.png",
    "qr_data": "https://pay.example.com/inv/100",
    "qr_size": "small",
    "qr_position": "bottom-right",
    "items": [
      {"description": "Platform License", "quantity": 1, "unit_price_cents": 5000000}
    ]
  }
}
```

### Receipt with Stamp

```json
{
  "name": "create_receipt",
  "arguments": {
    "output": "receipt.pdf",
    "company": "Zavora AI",
    "customer": "Acme Corp",
    "items": [{"description": "Payment", "quantity": 1, "unit_price_cents": 5000000}],
    "stamp": "received",
    "stamp_style": "circle",
    "stamp_color": "#006400"
  }
}
```

Stamp styles: `circle` · `rectangle` · `elegant` · `badge`

### Certificate (5 styles)

```json
{
  "name": "create_certificate",
  "arguments": {
    "output": "cert.pdf",
    "recipient": "James Karanja",
    "title": "Excellence in Engineering",
    "issuer": "Zavora AI",
    "style": "elegant"
  }
}
```

Styles: `classic` · `modern` · `elegant` · `academic` · `minimal`

## Security Model

```
Agent requests operation → Local processing → Result returned
                              │
                              ├── No cloud calls
                              ├── No data leaves machine
                              ├── Temp files cleaned
                              └── Hashes for integrity
```

- **Encryption** — AES-128 R4 with proper O/U value computation (PDF spec compliant)
- **Redaction** — True content removal via lopdf `replace_partial_text` (not visual masking)
- **Sanitization** — Removes JavaScript, actions, embedded files, metadata in one call
- **PII scanning** — Regex-based detection of emails, phones, SSNs, credit cards

## Competitive Comparison

| Feature | mcp-pdf | pdf-reader-mcp | mcp-pdf-utils | Nutrient DWS |
|---------|:-------:|:--------------:|:-------------:|:------------:|
| Total tools | **57** | 1 | 11 | 6 |
| Generate documents | ✅ | ❌ | ❌ | ❌ |
| QR codes | ✅ | ❌ | ❌ | ❌ |
| Stamps (4 styles) | ✅ | ❌ | ❌ | ❌ |
| High-fidelity extraction | ✅ | ✅ | basic | ✅ |
| True redaction | ✅ | ❌ | ❌ | ✅ (cloud) |
| Encryption | ✅ | ❌ | ❌ | ✅ (cloud) |
| Form filling | ✅ | ❌ | ❌ | ❌ |
| Local-first | ✅ | ✅ | ✅ | ❌ |
| Pure Rust | ✅ | ❌ (TS) | ❌ (TS) | ❌ (TS) |

## Documentation

| Document | Description |
|----------|-------------|
| [PROPOSAL-V3.md](PROPOSAL-V3.md) | Full platform proposal (131 tools, 10 verticals) |
| [SPEC.md](SPEC.md) | Implementation specification |
| [mcp-server.toml](mcp-server.toml) | ADK-Rust Enterprise registry manifest |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and credentials
- **Structured tracing** — `RUST_LOG` env-filter for observability
