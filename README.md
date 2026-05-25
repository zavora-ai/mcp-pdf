# PDF MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-pdf.svg)](https://crates.io/crates/mcp-pdf)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)

Complete PDF operations for AI agents — extract text, tables, forms, generate documents, invoices, merge, split, rotate, compress, encrypt. 20 tools, zero configuration, pure Rust.

## Why This Exists

No comprehensive PDF MCP exists. The market has single-purpose tools:

| Feature | kodey-pdf | gen-pdf | ninjadoc | **mcp-pdf** |
|---------|:---------:|:-------:|:--------:|:-----------:|
| Extract text | ❌ | ❌ | ✅ (1 tool) | ✅ |
| Extract tables | ❌ | ❌ | ❌ | ✅ |
| Form detection | ✅ | ❌ | ❌ | ✅ |
| Form filling | ✅ | ❌ | ❌ | ✅ |
| Generate PDF | ❌ | ✅ | ❌ | ✅ |
| Generate invoices | ❌ | ❌ | ❌ | ✅ |
| Merge/split | ❌ | ❌ | ❌ | ✅ |
| Rotate/compress | ❌ | ❌ | ❌ | ✅ |
| Encrypt | ❌ | ❌ | ❌ | ✅ |
| Signatures | ❌ | ❌ | ❌ | ✅ |
| Local (no cloud) | ❌ (R2) | ❌ | ❌ | ✅ |
| **Total tools** | **2** | **1** | **1** | **20** |

## Tools (20)

### Read & Extract (6)

| Tool | Purpose |
|------|---------|
| `extract_text` | Extract all text from PDF |
| `extract_metadata` | Page count, file size |
| `get_page_count` | Number of pages |
| `extract_page_text` | Text from a specific page |
| `extract_tables` | Tables as JSON arrays |
| `extract_images` | Image info from PDF |

### Forms (4)

| Tool | Purpose |
|------|---------|
| `detect_form_fields` | List all form fields |
| `fill_form` | Fill fields by name |
| `get_form_data` | Current field values |
| `flatten_form` | Make fields non-editable |

### Generate (2)

| Tool | Purpose |
|------|---------|
| `create_pdf` | Create PDF from text content |
| `create_invoice` | Generate invoice from structured data |

### Manipulate (4)

| Tool | Purpose |
|------|---------|
| `merge_pdfs` | Merge multiple PDFs |
| `split_pdf` | Extract page ranges |
| `rotate_pages` | Rotate pages (90/180/270°) |
| `compress_pdf` | Reduce file size |

### Security (2)

| Tool | Purpose |
|------|---------|
| `encrypt_pdf` | Password-protect |
| `verify_signature` | Check digital signatures |

### Utility (2)

| Tool | Purpose |
|------|---------|
| `add_watermark` | Text watermark |
| `get_info` | Quick PDF summary |

## Installation

```bash
cargo install mcp-pdf
```

Optional system tools (for merge/split/encrypt):
```bash
# macOS
brew install poppler qpdf

# Linux
apt install poppler-utils qpdf
```

## Configuration

**Zero configuration.** Just run `mcp-pdf`.

```json
{ "mcpServers": { "pdf": { "command": "mcp-pdf" } } }
```

## Usage Examples

### Extract content
```
"What does this contract say?"
→ extract_text(path="/docs/contract.pdf")

"How many pages is the report?"
→ get_page_count(path="/docs/report.pdf")
```

### Generate documents
```
"Create an invoice for Acme Corp"
→ create_invoice(output="invoice.pdf", company="Zavora AI", customer="Acme Corp",
    items=[{description:"Consulting", quantity:10, unit_price_cents:15000}])
```

### Manipulate
```
"Merge these three PDFs"
→ merge_pdfs(files=["part1.pdf","part2.pdf","part3.pdf"], output="combined.pdf")

"Compress this large PDF"
→ compress_pdf(path="large.pdf")
```

## Tested Live

```
✅ create_pdf: Generated test document
✅ create_invoice: Generated invoice ($8,000.00 total)
✅ extract_text: "Hello from mcp-pdf! This is a test document."
✅ get_info: {"pages": 1, "size_kb": 2, "version": "1.3"}
```

## License

Apache-2.0 — Part of [ADK-Rust Enterprise](https://enterprise.adk-rust.com)
