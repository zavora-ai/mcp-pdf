# mcp-pdf

The PDF Operating Layer for AI Agents — 57 tools for inspecting, extracting, generating, converting, manipulating, securing, and filling PDFs.

[![Crates.io](https://img.shields.io/crates/v/mcp-pdf.svg)](https://crates.io/crates/mcp-pdf)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## Why mcp-pdf?

No other MCP server combines read + write + intelligence + security + generation + forms in a single binary. Pure Rust, zero system dependencies, cross-platform.

| Feature | mcp-pdf | Others |
|---------|:-------:|:------:|
| Tools | **57** | 1–11 |
| Generate styled documents | ✅ | ❌ |
| 5 certificate styles | ✅ | ❌ |
| 4 stamp styles + custom colors | ✅ | ❌ |
| QR codes on invoices | ✅ | ❌ |
| High-fidelity PDF→Markdown | ✅ | ❌ |
| True redaction | ✅ | ❌ |
| AES-128 encryption | ✅ | ❌ |
| Form detection + filling | ✅ | partial |
| Flat form filling (coordinates) | ✅ | ❌ |
| Local-first, zero cloud | ✅ | varies |

## Install

```bash
cargo install mcp-pdf
```

## Configure (Claude Desktop / Cursor / VS Code)

```json
{
  "mcpServers": {
    "pdf": {
      "command": "mcp-pdf"
    }
  }
}
```

## Tools (57)

### Inspect & Understand (8)
`inspect_pdf` · `classify_pdf` · `health_check_pdf` · `detect_features` · `profile_complexity` · `get_page_count` · `get_info` · `repair_pdf`

### Extract Content (8)
`extract_text` · `extract_page_text` · `extract_metadata` · `extract_tables` · `extract_images` · `extract_bookmarks` · `extract_annotations` · `extract_key_values`

### Generate Documents (9)
`create_invoice` · `create_receipt` · `create_quote` · `create_statement` · `create_purchase_order` · `create_letter` · `create_certificate` · `create_report` · `create_contract`

### Convert (6)
`pdf_to_markdown` · `pdf_to_html` · `pdf_to_json` · `pdf_to_csv` · `markdown_to_pdf` · `images_to_pdf`

### Manipulate (10)
`merge_pdfs` · `split_pdf` · `split_by_bookmarks` · `rotate_pages` · `crop_pages` · `reorder_pages` · `delete_pages` · `compress_pdf` · `add_watermark` · `add_headers_footers`

### Numbering (2)
`add_page_numbers` · `add_bates_numbers`

### Security (9)
`hash_pdf` · `encrypt_pdf` · `decrypt_pdf` · `set_permissions` · `scan_sensitive_data` · `redact_pdf` · `sanitize_pdf` · `remove_metadata` · `detect_active_content`

### Forms (5)
`detect_form_fields` · `fill_form` · `flatten_form` · `fill_flat_form` · `describe_form_layout`

## Features

- **Logo support** — PNG/JPEG, auto-scaled, RGB-converted for Safari/Preview compatibility
- **QR codes** — Any data, 4 sizes (tiny/small/medium/large), 5 positions
- **Stamps** — 4 styles (circle/rectangle/elegant/badge), custom colors, curved text
- **Certificates** — 5 styles (classic/modern/elegant/academic/minimal)
- **Redaction** — True content removal using lopdf's replace_partial_text
- **Encryption** — AES-128 R4, pure Rust (md5 + aes + cbc crates)
- **High-fidelity conversion** — pdf_oxide for layout-aware markdown/HTML extraction
- **Form filling** — Both interactive (AcroForm) and flat (coordinate-based) forms
- **Watermark** — Form XObject overlay, works in all PDF viewers

## Architecture

```
src/
├── main.rs              # Entry point (9 lines)
├── server.rs            # MCP tool router
└── tools/
    ├── inspect.rs       # 8 inspect functions
    ├── extract.rs       # 8 extract functions
    ├── generate.rs      # 9 document generators
    ├── convert.rs       # 6 converters (pdf_oxide + comrak)
    ├── manipulate.rs    # 10 manipulation functions
    ├── numbering.rs     # Page numbers, Bates, headers/footers, split
    ├── security.rs      # 9 security functions
    └── forms.rs         # 5 form functions
```

## License

Apache-2.0
