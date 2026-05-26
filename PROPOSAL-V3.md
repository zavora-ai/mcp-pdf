# mcp-pdf v3.0 — The PDF Operating Layer for AI Agents

> The world runs on PDFs — 400 billion opened annually, 1.7 billion lifetime users, a $2.41B market growing to $7.13B by 2035. This is not a document generator. This is the complete PDF operating layer for AI agents.

---

## Positioning

> The local-first PDF operating layer for AI agents: understand, extract, cite, repair, secure, validate, generate, compare, and automate PDFs at personal, professional, and enterprise scale.

---

## Design Principles

1. **Understand first, act second.** Before any operation, the agent can inspect, classify, and health-check a PDF. No blind processing.

2. **Read-first, generate-second.** Most PDF interactions in the world are with *existing* documents. Prioritize extraction, transformation, and intelligence over generation.

3. **AI-native, not AI-adjacent.** Every extraction and generation tool has an intelligent variant. The MCP makes agents smarter about documents, not just processes bytes.

4. **Local-first, privacy-default.** No silent uploads. No external calls unless explicitly enabled. Temporary files deleted after use. Hashes generated for sensitive workflows.

5. **Modular by capability class.** Core (pure Rust), OCR (Tesseract), Conversion (LibreOffice), AI (LLM APIs), Signatures (OpenSSL). Each is optional and independently configurable via feature flags.

6. **Batch-first design.** Every single-document tool also works on a directory. Enterprises don't process one file at a time.

7. **Graceful degradation.** If a capability module is not present, the tool returns a structured error with installation guidance — never panics.

8. **Universal coverage.** Legal in Nairobi, healthcare in Mumbai, government in São Paulo, education in Berlin — the tool serves them all.

---

## Architecture

### Module System (Feature Flags)

```toml
[dependencies]
# Core — always present (pure Rust, zero external deps)
printpdf = { version = "0.7", features = ["embedded_images"] }
lopdf = "0.40"
pdf-extract = "0.10"
image = "0.24"
sha2 = "0.10"
chrono = "0.4"

[features]
default = ["core"]
core = []

# OCR — binds to system Tesseract 5
ocr = ["dep:tesseract"]

# Conversion — bridges to LibreOffice headless + Pandoc
conversion = ["dep:which"]

# AI Intelligence — calls LLM API (Anthropic/OpenAI/Ollama/Bedrock)
ai = ["dep:reqwest"]

# Signatures — PKI/X.509 digital signatures
signatures = ["dep:openssl"]

# Search — local semantic/keyword indexing
search = ["dep:tantivy"]

# Full — everything
full = ["ocr", "conversion", "ai", "signatures", "search"]
```

### Runtime Capability Detection

```json
{
  "error": "ocr_not_available",
  "message": "OCR requires Tesseract 5. Install: brew install tesseract",
  "fallback": "extract_text can handle digitally-created PDFs without OCR"
}
```

---

## Tool Inventory: 106 Tools Across 13 Pillars

---

### PILLAR 1 — Inspect & Understand (8 tools)

*Before any operation, understand what you're working with.*

| Tool | Description |
|------|-------------|
| `inspect_pdf` | Full structural profile: pages, fonts, images, forms, signatures, encryption, version |
| `classify_pdf` | Identify document type: invoice, contract, form, scan, report, letter, certificate |
| `health_check_pdf` | Detect corruption, xref errors, broken objects, malformed streams |
| `detect_features` | Report: has forms, has tags, has signatures, has JavaScript, has embedded files |
| `profile_complexity` | Extraction/OCR difficulty score (simple text → complex multi-column scan) |
| `get_page_count` | Number of pages |
| `get_info` | Size, version, encryption status, metadata summary |
| `repair_pdf` | Attempt recovery of corrupted or malformed PDFs (rebuild xref, fix streams) |

---

### PILLAR 2 — Extract Content (14 tools)

*Get structured data out of any PDF.*

| Tool | Description |
|------|-------------|
| `extract_text` | All text, plain |
| `extract_text_structured` | Text with page numbers, bounding boxes, block types |
| `extract_page_text` | Text from a specific page |
| `extract_layout` | Headings, paragraphs, columns, reading order |
| `extract_tables` | Tables as JSON arrays with headers and rows |
| `extract_key_values` | Form-like key-value pairs (label → value) |
| `extract_images` | Embedded images (metadata + optional export) |
| `extract_figures` | Figures, diagrams, charts with captions |
| `extract_annotations` | Comments, highlights, stamps, markup |
| `extract_bookmarks` | Outline/bookmark tree |
| `extract_attachments` | Embedded files |
| `extract_barcodes` | QR codes and barcodes (values + positions) |
| `extract_metadata` | XMP, document info, creation/modification dates |
| `extract_reading_order` | Human reading sequence for accessibility |

---

### PILLAR 3 — OCR & Scan Processing (6 tools)

*Make the paper-based world digital. Feature: `ocr`*

| Tool | Description |
|------|-------------|
| `detect_ocr_needed` | Whether a PDF requires OCR (image-only pages) |
| `ocr_pdf` | Full document OCR → searchable text PDF; 100+ languages |
| `ocr_page` | OCR a single page (fast, targeted) |
| `ocr_region` | OCR a bounding box region on a page |
| `ocr_table` | Table-aware OCR → structured JSON with rows/cells |
| `clean_scan` | Deskew, denoise, improve contrast before OCR |

---

### PILLAR 4 — AI Intelligence (14 tools)

*Turn PDFs from static files into queryable knowledge. Feature: `ai`*

| Tool | Description |
|------|-------------|
| `summarize_pdf` | Executive summary, bullet digest, or section-level; configurable length |
| `answer_question` | Answer a question using the PDF as source; returns answer + page citations |
| `search_pdf` | Exact text search with page/position results |
| `semantic_search` | Meaning-based search across document content |
| `build_index` | Create a local searchable index over one or many PDFs |
| `extract_entities` | People, organizations, dates, amounts, locations |
| `extract_claims` | Factual claims with source locations |
| `extract_timeline` | Chronological event list from document |
| `extract_obligations` | Duties, deadlines, penalties from contracts |
| `extract_risks` | Legal/financial/compliance risks |
| `compare_documents` | Semantic + textual diff of two PDFs; structured change report |
| `translate_pdf` | Translate while preserving layout; 50+ languages |
| `assess_reading_level` | Flesch-Kincaid + language complexity analysis |
| `generate_brief` | Create executive/legal/research brief from source PDF |

---

### PILLAR 5 — Document Intelligence / IDP (9 tools)

*Structured extraction for agentic workflows. Feature: `ai`*

| Tool | Description |
|------|-------------|
| `parse_invoice` | → JSON: vendor, buyer, line items, amounts, tax, currency, due date (with confidence scores) |
| `parse_receipt` | → JSON: merchant, date, items, total, payment method |
| `parse_id_document` | → JSON: name, DOB, ID number, expiry, nationality (from ID/passport scans) |
| `parse_contract` | → JSON: parties, dates, obligations, termination clauses, signatures |
| `parse_bank_statement` | → JSON: account info, transactions, balances, period |
| `parse_form` | → JSON: all field labels + values from any form PDF |
| `parse_resume` | → JSON: name, contact, experience, education, skills |
| `extract_line_items` | Tabular line items from any document → structured rows |
| `extract_named_entities` | People, organizations, locations, dates, amounts from text |

All IDP tools return typed, validated JSON with confidence scores:

```json
{
  "document_type": "invoice",
  "confidence": 0.97,
  "vendor": { "name": "Zavora AI", "confidence": 0.99 },
  "line_items": [
    { "description": "Enterprise License", "quantity": 1, "unit_price": 12000.00, "confidence": 0.95 }
  ],
  "total": { "subtotal": 12000.00, "tax": 1920.00, "total": 13920.00, "currency": "KES" }
}
```

---

### PILLAR 6 — Format Conversion (10 tools)

*The #1 most-requested PDF capability globally. Feature: `conversion`*

| Tool | Direction | Notes |
|------|-----------|-------|
| `pdf_to_markdown` | PDF → .md | Structure-aware, tables preserved |
| `pdf_to_html` | PDF → .html | Responsive with embedded CSS |
| `pdf_to_docx` | PDF → .docx | Layout-preserving; columns, tables, images |
| `pdf_to_images` | PDF → PNG/JPEG | Batch page render; configurable DPI |
| `pdf_to_json` | PDF → .json | Structured extraction with schema |
| `pdf_to_csv` | PDF → .csv | Table detection and export |
| `markdown_to_pdf` | .md → PDF | Styled with template system |
| `html_to_pdf` | .html → PDF | Headless browser render |
| `images_to_pdf` | Images → PDF | Multi-image assembly |
| `office_to_pdf` | .docx/.xlsx/.pptx → PDF | LibreOffice headless |

---

### PILLAR 7 — Document Generation (14 tools)

*Professional documents from structured data. 8 built-in styles. 10 industry verticals. 88 templates.*

| Tool | Output |
|------|--------|
| `create_document` | General PDF from schema + content + style |
| `create_from_template` | Render a saved/vertical template with data |
| `create_template` | Define a reusable template (layout + variables) |
| `create_invoice` | Invoice with line items, tax, totals, logo |
| `create_receipt` | Payment receipt / confirmation |
| `create_quote` | Price quote / estimate with validity |
| `create_statement` | Financial/account statement |
| `create_purchase_order` | PO with buyer, vendor, items, terms |
| `create_contract` | Agreement with clauses + signature blocks |
| `create_report` | Multi-section report with headers |
| `create_proposal` | Client proposal with pricing + timeline |
| `create_letter` | Business letter with letterhead |
| `create_certificate` | Certificate of achievement/completion |
| `create_pdf_packet` | Binder: cover + TOC + sections + attachments |

**Vertical-aware:** All generation tools accept `vertical` and `template` parameters. When specified, the tool applies industry-specific layout, compliance rules, and field validation automatically.

---

### PILLAR 8 — Page & File Operations (12 tools)

*Manipulate existing PDFs.*

| Tool | Description |
|------|-------------|
| `merge_pdfs` | Merge multiple PDFs into one |
| `split_pdf` | Extract page ranges into new PDF |
| `split_by_bookmarks` | Split into separate PDFs by section |
| `rotate_pages` | Rotate pages (90/180/270°) |
| `crop_pages` | Crop to specified bounding box |
| `resize_pages` | Change page dimensions |
| `reorder_pages` | Reorder by index array |
| `delete_pages` | Remove specified pages |
| `insert_pages` | Insert pages from another PDF at position |
| `compress_pdf` | Reduce file size (image resampling, stream compression) |
| `linearize_pdf` | Optimize for fast web streaming |
| `add_watermark` | Text or image watermark on all pages |

---

### PILLAR 9 — Headers, Footers & Numbering (4 tools)

*Professional document finishing.*

| Tool | Description |
|------|-------------|
| `add_page_numbers` | Arabic, Roman, or alphanumeric; configurable position |
| `add_bates_numbers` | Legal Bates numbering with prefix/suffix |
| `add_headers_footers` | Dynamic fields: page number, date, title, author |
| `add_table_of_contents` | Auto-generate hyperlinked TOC from heading structure |

---

### PILLAR 10 — Forms & Signatures (12 tools)

*Handle real-world forms and signing workflows.*

| Tool | Description |
|------|-------------|
| `detect_form_fields` | List all fields (name, type, value, required) |
| `describe_form` | Human-readable form explanation |
| `form_to_schema` | Generate JSON schema from form structure |
| `get_form_data` | Current field values |
| `fill_form` | Fill fields by name-value map |
| `fill_form_from_text` | AI-powered: map unstructured text to form fields |
| `validate_form` | Check required fields, format rules |
| `flatten_form` | Make fields non-editable |
| `create_fillable_form` | Generate a fillable PDF from schema |
| `add_signature_field` | Place signature field at coordinates |
| `sign_pdf` | Apply digital signature (X.509 certificate) |
| `validate_signatures` | Verify all signatures; return validity report |

---

### PILLAR 11 — Security, Privacy & Compliance (12 tools)

*GDPR, CCPA, HIPAA, Kenya DPA 2019, POPIA, LGPD compliant.*

| Tool | Description |
|------|-------------|
| `scan_sensitive_data` | Detect PII/PHI/PCI: names, emails, phones, IDs, accounts |
| `preview_redactions` | Show what would be redacted (non-destructive preview) |
| `redact_pdf` | True redaction: remove text, objects, metadata, hidden content |
| `verify_redaction` | Confirm redacted terms are truly gone (text + object + metadata scan) |
| `sanitize_pdf` | Deep clean: remove JS, embedded files, hidden layers, comments |
| `remove_metadata` | Strip author, creation date, GPS, software fingerprint |
| `detect_active_content` | Find JavaScript, actions, embedded executables |
| `remove_active_content` | Strip all active/executable content |
| `encrypt_pdf` | AES-256 password protection with permission control |
| `decrypt_pdf` | Remove password from an owned document |
| `set_permissions` | Control print, copy, edit, annotate independently |
| `hash_pdf` | SHA-256 hash for integrity verification |

**Redaction safety rule:** A PDF operation is only called "redaction" if it removes the underlying text, objects, metadata, AND hidden content. Visual black boxes are "masking," not redaction.

---

### PILLAR 12 — Accessibility & Standards (10 tools)

*Legally mandated in 40+ countries for government, education, healthcare.*

| Tool | Description |
|------|-------------|
| `audit_accessibility` | Full PDF/UA compliance check; structured issue report |
| `generate_accessibility_report` | Human-readable remediation guide |
| `detect_reading_order_issues` | Find reading order problems |
| `set_reading_order` | Define logical reading order |
| `detect_missing_alt_text` | Images needing alt text |
| `add_alt_text` | Add/edit alternative text for images |
| `auto_tag_pdf` | Add semantic structure tags (headings, lists, tables) |
| `validate_pdf_a` | PDF/A archival compliance check |
| `convert_to_pdf_a` | Convert to PDF/A-1b, A-2b, or A-3b |
| `validate_pdf_ua` | PDF/UA accessibility standard validation |

---

### PILLAR 13 — Batch & Workflows (6 tools)

*Enterprise-scale automation.*

| Tool | Description |
|------|-------------|
| `batch_process` | Apply any single-document operation to a folder |
| `create_workflow` | Define a multi-step pipeline (classify → OCR → extract → rename) |
| `run_workflow` | Execute a saved workflow on input files |
| `dry_run_workflow` | Preview what a workflow would do (non-destructive) |
| `deduplicate_pdfs` | Find and report duplicate PDFs in a folder |
| `generate_report` | Processing report (CSV/JSON) from batch results |

**Workflow schema:**
```json
{
  "name": "Invoice Extraction Pipeline",
  "steps": [
    { "tool": "classify_pdf", "input": "$file" },
    { "tool": "ocr_pdf", "condition": "classification.is_scanned == true" },
    { "tool": "parse_invoice" },
    { "tool": "batch_process", "operation": "compress_pdf" }
  ]
}
```

---

## Tool Count Summary

| Pillar | Tools | Feature Flag |
|--------|-------|--------------|
| 1. Inspect & Understand | 8 | core |
| 2. Extract Content | 14 | core |
| 3. OCR & Scan Processing | 6 | ocr |
| 4. AI Intelligence | 14 | ai |
| 5. Document Intelligence / IDP | 9 | ai |
| 6. Format Conversion | 10 | conversion |
| 7. Document Generation | 14 | core |
| 8. Page & File Operations | 12 | core |
| 9. Headers, Footers & Numbering | 4 | core |
| 10. Forms & Signatures | 12 | core + signatures |
| 11. Security & Compliance | 12 | core |
| 12. Accessibility & Standards | 10 | core |
| 13. Batch & Workflows | 6 | core |
| **Total** | **131** | |

**Core (no external deps): 92 tools**
**With all features enabled: 131 tools**
**With all verticals: 131 tools + 88 industry templates across 10 verticals**

---

## Style System — Document Design System

### 8 Built-in Styles

| Style | Best for | Visual character |
|-------|----------|-----------------|
| `minimal` | Letters, simple reports | Clean, quiet, Apple-like |
| `modern` | Startups, proposals | Accent color, spacious |
| `corporate` | Enterprise, board reports | Formal, structured |
| `stripe` | Fintech, SaaS | Whitespace, subtle borders |
| `legal` | Contracts, filings | Dense, numbered, citation-friendly |
| `academic` | Papers, research | Two-column, journal-like |
| `government` | Forms, public docs | Plain, compliant, accessible |
| `finance` | Invoices, statements | Tabular, precise, compact |

### All Styles Support

- Custom logo (PNG/JPEG, auto-scaled)
- Custom primary/secondary/accent colors
- Custom fonts (TTF/OTF embedding)
- Multi-page with auto page breaks
- Page numbers + headers/footers
- Tables with proper alignment
- Bookmarks and TOC
- Signature blocks
- Bates numbering
- Language metadata
- Accessibility tags (PDF/UA mode)
- PDF/A export mode
- High-contrast mode
- RTL layout support

### Branding Configuration

```json
{
  "logo": "/path/to/logo.png",
  "primary_color": "#0A1A33",
  "secondary_color": "#008080",
  "accent_color": "#FF6B35",
  "font_family": "Inter",
  "font_path": "/path/to/Inter.ttf"
}
```

---

## Vertical Templates & Industry Packs

*This is the enterprise differentiator. Each vertical ships pre-built templates, field schemas, compliance rules, and styling tuned to that industry's standards. An AI agent deploying mcp-pdf in a hospital produces documents that look like hospital documents — not generic PDFs.*

### How Verticals Work

```json
{
  "vertical": "healthcare",
  "template": "patient_discharge_summary",
  "style": "medical",
  "data": { ... },
  "compliance": ["hipaa", "pdf_ua"]
}
```

The vertical system provides:
1. **Templates** — pre-built document layouts specific to the industry
2. **Styles** — visual design tuned to industry norms (colors, density, typography)
3. **Schemas** — validated field structures (required fields, formats, types)
4. **Compliance rules** — auto-applied (redaction, accessibility, archival)
5. **Terminology** — correct labels, headers, and section names

---

### VERTICAL 1 — Healthcare & Life Sciences

**Style:** `medical` — Clean, high-contrast, accessible, HIPAA-aware

| Template | Fields | Compliance |
|----------|--------|------------|
| `patient_discharge_summary` | patient, provider, diagnosis, medications, follow_up, vitals | HIPAA, PDF/UA |
| `lab_report` | patient, tests[], results[], reference_ranges, ordering_physician | HIPAA, PDF/A |
| `prescription` | patient, provider, medications[], dosage, refills, DEA_number | HIPAA |
| `medical_certificate` | patient, provider, condition, fitness_status, valid_until | HIPAA |
| `insurance_claim` | patient, provider, procedures[], diagnosis_codes[], amounts | HIPAA, CMS-1500 |
| `clinical_trial_report` | study_id, phases, endpoints, adverse_events, conclusions | ICH-GCP |
| `consent_form` | patient, procedure, risks, alternatives, signature_blocks | HIPAA, IRB |
| `referral_letter` | from_provider, to_provider, patient, reason, history | HIPAA |

**Auto-applied rules:**
- PII fields auto-marked for redaction capability
- PDF/UA accessibility tags on all output
- Patient identifiers in header/footer for multi-page
- Signature blocks with date/time stamps

---

### VERTICAL 2 — Legal & Compliance

**Style:** `legal` — Dense, numbered paragraphs, citation-friendly, Bates-ready

| Template | Fields | Compliance |
|----------|--------|------------|
| `contract` | parties[], effective_date, clauses[], governing_law, signatures[] | — |
| `nda` | disclosing_party, receiving_party, scope, duration, exceptions | — |
| `lease_agreement` | landlord, tenant, property, term, rent, clauses[] | Jurisdiction-specific |
| `power_of_attorney` | principal, agent, powers[], limitations, effective_date | Notarization block |
| `court_filing` | case_number, court, parties[], filing_type, body, exhibits[] | Court rules |
| `legal_opinion` | firm, client, matter, analysis[], conclusion, caveats | — |
| `cease_and_desist` | sender, recipient, violation, demands[], deadline | — |
| `affidavit` | affiant, statements[], notary_block | Jurat/acknowledgment |
| `discovery_packet` | case_number, documents[], bates_range, privilege_log | FRCP/local rules |
| `terms_of_service` | company, effective_date, sections[], jurisdiction | GDPR/CCPA notice |

**Auto-applied rules:**
- Bates numbering on all pages
- Line numbering (court filings)
- Confidentiality stamps
- Signature blocks with witness lines
- Exhibit separators with tabs

---

### VERTICAL 3 — Finance & Banking

**Style:** `finance` — Tabular, precise, compact, audit-trail ready

| Template | Fields | Compliance |
|----------|--------|------------|
| `invoice` | vendor, customer, items[], tax, totals, payment_terms | — |
| `receipt` | merchant, customer, items[], payment_method, transaction_id | — |
| `bank_statement` | account, period, opening_balance, transactions[], closing_balance | — |
| `financial_report` | company, period, sections[], tables[], charts[] | GAAP/IFRS |
| `audit_report` | auditor, client, scope, findings[], opinion, material_weaknesses | ISA/GAAS |
| `tax_return_summary` | taxpayer, period, income, deductions, tax_due, payments | Jurisdiction |
| `loan_agreement` | lender, borrower, principal, rate, term, covenants[] | Truth in Lending |
| `investment_memo` | fund, company, thesis, financials, risks, recommendation | — |
| `purchase_order` | buyer, vendor, items[], delivery_terms, payment_terms | — |
| `credit_note` | vendor, customer, original_invoice, items[], reason | — |

**Auto-applied rules:**
- Currency formatting (locale-aware)
- Running totals and subtotals
- Decimal alignment in tables
- PDF/A for archival
- Audit trail metadata (creation date, author, version)

---

### VERTICAL 4 — Government & Public Sector

**Style:** `government` — Plain, accessible, compliant, multi-language ready

| Template | Fields | Compliance |
|----------|--------|------------|
| `official_notice` | authority, recipient, subject, body, reference_number, deadline | PDF/UA |
| `permit` | authority, applicant, permit_type, conditions[], valid_until | PDF/UA |
| `license` | authority, holder, license_type, number, issued, expires | PDF/UA |
| `public_tender` | authority, project, requirements[], deadline, evaluation_criteria | Procurement rules |
| `meeting_minutes` | body, date, attendees[], agenda[], decisions[], actions[] | Open records |
| `policy_document` | authority, title, sections[], effective_date, supersedes | PDF/UA, PDF/A |
| `citizen_form` | authority, form_number, fields[], instructions, submission_info | PDF/UA, fillable |
| `budget_report` | authority, fiscal_year, departments[], line_items[], totals | PDF/A |
| `inspection_report` | inspector, location, date, findings[], violations[], corrective_actions | — |
| `certificate_of_registration` | authority, entity, registration_number, date, category | PDF/UA |

**Auto-applied rules:**
- PDF/UA accessibility mandatory
- PDF/A archival format
- Multi-language support (RTL, CJK)
- High-contrast mode available
- Government seal/logo placement
- Reference numbers in headers

---

### VERTICAL 5 — Education & Research

**Style:** `academic` — Two-column option, citation-friendly, structured

| Template | Fields | Compliance |
|----------|--------|------------|
| `transcript` | institution, student, courses[], grades[], gpa, degree | FERPA |
| `diploma` | institution, student, degree, major, date, honors | — |
| `certificate_of_completion` | institution, recipient, course, hours, date, instructor | — |
| `research_paper` | title, authors[], abstract, sections[], references[], figures[] | Journal style |
| `syllabus` | institution, course, instructor, schedule[], objectives[], grading | — |
| `recommendation_letter` | author, subject, relationship, body, signature | FERPA |
| `grant_proposal` | pi, institution, title, abstract, budget, timeline, references[] | Funder format |
| `thesis_cover` | institution, student, title, degree, committee[], date | University format |
| `report_card` | institution, student, period, subjects[], grades[], comments | FERPA |
| `accreditation_report` | institution, standards[], findings[], recommendations[] | Accreditor format |

**Auto-applied rules:**
- FERPA-aware PII handling
- Citation formatting (APA, MLA, Chicago)
- Figure/table numbering
- Cross-references
- Bookmarks from heading structure
- PDF/A for institutional archives

---

### VERTICAL 6 — Real Estate & Property

**Style:** `corporate` with property-specific extensions

| Template | Fields | Compliance |
|----------|--------|------------|
| `lease` | landlord, tenant, property, term, rent, deposit, clauses[] | Jurisdiction |
| `purchase_agreement` | buyer, seller, property, price, contingencies[], closing_date | Jurisdiction |
| `property_listing` | agent, property, description, features[], price, photos[] | Fair Housing |
| `inspection_report` | inspector, property, date, systems[], findings[], photos[] | — |
| `closing_disclosure` | buyer, seller, lender, property, costs[], amounts | TRID/RESPA |
| `property_management_report` | manager, owner, property, period, income, expenses, maintenance[] | — |
| `eviction_notice` | landlord, tenant, property, reason, cure_period, deadline | Jurisdiction |
| `rental_application` | applicant, employment, income, references[], consent | Fair Housing |

**Auto-applied rules:**
- Property address in header
- Legal description formatting
- Signature blocks with notary option
- Exhibit attachments (photos, surveys)
- Fair Housing disclaimer

---

### VERTICAL 7 — Human Resources

**Style:** `corporate` with HR-specific layouts

| Template | Fields | Compliance |
|----------|--------|------------|
| `offer_letter` | company, candidate, position, compensation, start_date, benefits | — |
| `employment_contract` | employer, employee, role, terms, compensation, clauses[] | Labor law |
| `payslip` | employee, period, earnings[], deductions[], net_pay, ytd | — |
| `performance_review` | employee, reviewer, period, goals[], ratings[], development_plan | — |
| `termination_letter` | company, employee, effective_date, reason, severance, obligations | Labor law |
| `employee_handbook` | company, sections[], policies[], acknowledgment | — |
| `job_description` | company, title, department, responsibilities[], qualifications[] | — |
| `training_certificate` | company, employee, course, date, hours, instructor | — |
| `incident_report` | reporter, date, location, description, witnesses[], actions[] | OSHA |
| `exit_interview` | employee, date, questions[], responses[], recommendations | — |

**Auto-applied rules:**
- PII redaction capability on all employee data
- Company branding (logo, colors)
- Signature blocks with date
- Confidentiality notice in footer
- PDF/A for personnel file archival

---

### VERTICAL 8 — Logistics & Supply Chain

**Style:** `finance` with logistics extensions

| Template | Fields | Compliance |
|----------|--------|------------|
| `bill_of_lading` | shipper, consignee, carrier, goods[], weight, origin, destination | UCC |
| `packing_slip` | shipper, recipient, order_number, items[], packages[] | — |
| `shipping_label` | from, to, carrier, tracking, weight, service_level, barcode | Carrier format |
| `customs_declaration` | exporter, importer, goods[], values[], hs_codes[], origin_country | Customs regs |
| `warehouse_receipt` | warehouse, depositor, goods[], quantity, condition, date | UCC |
| `delivery_note` | sender, recipient, items[], driver, vehicle, signature_block | — |
| `freight_invoice` | carrier, shipper, shipments[], charges[], fuel_surcharge | — |
| `certificate_of_origin` | exporter, goods[], origin_country, chamber_of_commerce | Trade agreements |

**Auto-applied rules:**
- Barcode/QR code generation
- Weight/dimension formatting
- Multi-currency support
- Customs-compliant layouts
- Carrier-specific label formats

---

### VERTICAL 9 — Insurance

**Style:** `corporate` with insurance-specific density

| Template | Fields | Compliance |
|----------|--------|------------|
| `policy_document` | insurer, policyholder, coverage[], limits[], deductibles[], term | State regs |
| `claim_form` | policyholder, policy_number, incident_date, description, damages[] | — |
| `loss_report` | adjuster, policyholder, property, cause, damages[], estimate | — |
| `certificate_of_insurance` | insurer, holder, coverages[], limits[], additional_insureds[] | ACORD |
| `renewal_notice` | insurer, policyholder, policy, changes[], new_premium, deadline | — |
| `denial_letter` | insurer, claimant, claim_number, reason, appeal_rights | State regs |
| `underwriting_report` | underwriter, applicant, risk_factors[], recommendation, conditions | — |
| `subrogation_demand` | insurer, responsible_party, claim, amount, deadline | — |

**Auto-applied rules:**
- Policy number in header
- Coverage tables with limits/deductibles
- Regulatory disclaimers by jurisdiction
- ACORD form compatibility
- Signature blocks with agent info

---

### VERTICAL 10 — Construction & Engineering

**Style:** `corporate` with technical drawing references

| Template | Fields | Compliance |
|----------|--------|------------|
| `project_proposal` | contractor, client, scope, timeline, budget, terms | — |
| `change_order` | project, contractor, client, changes[], cost_impact, schedule_impact | AIA format |
| `progress_report` | project, period, milestones[], completion_pct, issues[], photos[] | — |
| `safety_report` | project, date, incidents[], inspections[], corrective_actions[] | OSHA |
| `punch_list` | project, area, items[], responsible_party, deadline | — |
| `certificate_of_completion` | project, contractor, client, date, conditions[] | — |
| `material_submittal` | project, supplier, materials[], specifications[], approvals[] | — |
| `rfi` | project, from, to, question, context, response, date | — |

**Auto-applied rules:**
- Project number/name in header
- Drawing reference numbers
- Revision tracking (Rev A, B, C...)
- Photo attachments with captions
- AIA document numbering

---

### Vertical Configuration

Verticals are selected per-document and can be combined with any base style:

```json
{
  "vertical": "healthcare",
  "template": "lab_report",
  "style": "medical",
  "branding": {
    "logo": "/path/to/hospital-logo.png",
    "primary_color": "#1B4D89",
    "name": "Nairobi General Hospital"
  },
  "compliance": ["hipaa", "pdf_ua", "pdf_a"],
  "data": {
    "patient": { "name": "John Doe", "id": "MRN-12345", "dob": "1985-03-15" },
    "tests": [
      { "name": "Complete Blood Count", "result": "Normal", "reference": "4.5-11.0" }
    ],
    "ordering_physician": "Dr. Sarah Kimani"
  }
}
```

### Enterprise Vertical Packs

For ADK-Rust Enterprise customers, verticals ship as **packs** that can be licensed independently:

| Pack | Verticals | Templates | Target Market |
|------|-----------|-----------|---------------|
| **Healthcare Pack** | Healthcare, Insurance | 16 templates | Hospitals, clinics, insurers |
| **Legal Pack** | Legal, Real Estate | 18 templates | Law firms, courts, property |
| **Finance Pack** | Finance, Insurance | 18 templates | Banks, fintechs, auditors |
| **Government Pack** | Government, Education | 20 templates | Agencies, schools, universities |
| **Enterprise Pack** | HR, Logistics, Construction | 26 templates | Corporates, manufacturers |
| **Full Platform** | All 10 verticals | 88 templates | Enterprise-wide deployment |

### Custom Vertical Creation

Enterprises can define their own verticals:

```json
{
  "vertical_name": "telecom",
  "style_base": "corporate",
  "style_overrides": {
    "primary_color": "#E20074",
    "table_header_bg": "#2D0A4E"
  },
  "templates": [
    {
      "name": "service_agreement",
      "fields": ["customer", "plan", "term", "monthly_charge", "data_cap"],
      "sections": ["terms", "fair_usage", "cancellation"],
      "compliance": ["consumer_protection"]
    }
  ]
}
```

---

## Core Workflows

### 1. Make This PDF Usable

```
inspect_pdf → repair_pdf (if needed) → ocr_pdf (if scanned) → extract_text_structured → summarize_pdf
```
**Output:** Searchable PDF + structured JSON + summary + diagnostic report

### 2. Securely Redact a PDF

```
scan_sensitive_data → preview_redactions → redact_pdf → remove_metadata → sanitize_pdf → verify_redaction → hash_pdf
```
**Output:** Safe redacted PDF + verification report + SHA-256 hash

### 3. Extract Data from Many PDFs

```
batch_process(classify_pdf) → batch_process(ocr_pdf, condition: is_scanned) → batch_process(parse_invoice) → generate_report
```
**Output:** Structured dataset (JSON/CSV) + processed PDFs + exception report

### 4. Make a PDF Accessible

```
audit_accessibility → set_reading_order → auto_tag_pdf → add_alt_text → validate_pdf_ua → generate_accessibility_report
```
**Output:** Tagged accessible PDF + remediation report

### 5. Compare Two Contracts

```
extract_text_structured (both) → compare_documents → extract_obligations (both) → extract_risks (both)
```
**Output:** Semantic diff + clause comparison + risk summary

### 6. Prepare a Legal Packet

```
merge_pdfs → add_table_of_contents → add_bates_numbers → add_page_numbers → add_headers_footers → validate_signatures (warn if breaking)
```
**Output:** Court/board-ready packet PDF

### 7. Fill a Government Form

```
detect_form_fields → describe_form → fill_form_from_text → validate_form → flatten_form
```
**Output:** Completed form + validation checklist

---

## AI Intelligence Layer Design

### Configuration

```toml
[ai]
provider = "anthropic"        # or "openai", "ollama", "bedrock"
model = "claude-sonnet-4-20250514"
chunk_size = 8000
chunk_overlap = 200
max_context_pages = 50
embedding_model = "local"     # or "openai", "cohere"
```

### RAG Pipeline (for `answer_question`, `semantic_search`)

1. Extract text with `pdf-extract` (or OCR if scanned)
2. Chunk at paragraph/sentence boundaries with overlap
3. Embed chunks (local MiniLM or API)
4. Vector similarity search on query
5. Assemble context with page citations
6. LLM completion with source attribution

### IDP Pipeline (for `parse_invoice`, `parse_contract`, etc.)

1. `extract_text` or `ocr_pdf` → raw text
2. Structured LLM extraction with JSON schema enforcement
3. Validation against expected field types
4. Confidence scoring per field
5. Return typed JSON

---

## Security Model

| Rule | Description |
|------|-------------|
| Local by default | No external calls unless feature explicitly enabled |
| No silent uploads | AI features require explicit `ai` feature flag |
| Temp file cleanup | All temporary files deleted after operation |
| Hash chain | Sensitive workflows produce SHA-256 hashes |
| Redaction verification | True redaction verified at text + object + metadata + hidden content levels |
| Signature warnings | Operations that would break signatures produce warnings before proceeding |
| Active content detection | JavaScript/actions detected and reported before processing |
| Metadata sanitization | One-command removal of all identifying metadata |

---

## Industry Vertical Coverage

| Vertical | Critical Capabilities | Coverage |
|----------|----------------------|----------|
| **Legal** | Bates numbering, redaction, e-sign, contract parsing, certified PDF, legal packets | 100% |
| **Healthcare** | PII/PHI redaction, HIPAA-safe processing, accessible PDFs, form extraction | 95% |
| **Finance** | Invoice/statement parsing, bank statement extraction, audit-ready PDF/A | 100% |
| **Government** | PDF/UA accessibility, PDF/A archival, digital signatures, form filling | 95% |
| **Education** | OCR scanned materials, accessible PDFs, certificate generation, summarization | 95% |
| **Real Estate** | Contract parsing, e-sign, lease generation, Bates numbering | 90% |
| **HR** | Resume parsing, form filling, certificate generation, PII redaction | 95% |
| **Logistics** | Barcode/QR reading, invoice parsing, OCR shipping docs, batch processing | 95% |

---

## Competitive Landscape

### Market Research (May 2026)

| Server | Stars | Lang | Tools | Focus |
|--------|-------|------|-------|-------|
| **kreuzberg** | 8,387★ | Rust | ~8 | Document intelligence framework (90+ formats, OCR, embeddings, MCP mode) |
| **@sylphx/pdf-reader-mcp** | 729★ | TS | 1 | Single `read_pdf` tool with parallel processing (5-10x speed) |
| **ebook-mcp** | 366★ | Python | ~5 | EPUB/PDF reading via TOC navigation |
| **mcp-pdf-tools** | 74★ | Python | 4 | Merge, split, search, pattern-match PDFs |
| **Nutrient DWS MCP** | 65★ | TS | 6 | Cloud API (PSPDFKit): convert, OCR, redact, sign, watermark. Requires API key |
| **document-edit-mcp** | 49★ | Python | ~8 | Word/Excel/PDF creation (basic text→PDF only) |
| **jztan/pdf-mcp** | 43★ | Python | 8 | Hybrid search (BM25+semantic), OCR, paginated reading, SQLite cache |
| **pdf-rag-mcp-server** | 43★ | Python | ~3 | RAG over PDFs with vector DB + web UI |
| **mcp-pdf-utils** | — | TS | 11 | Merge, split, rotate, watermark, extract text, metadata |
| **@mcp-z/mcp-pdf** | — | TS | 4 | Generation: resume, layout, document, page-to-image |
| **@modelcontextprotocol/server-pdf** | — | TS | 1 | Official MCP reference: text extraction only |

### Detailed Capability Comparison

| Capability | kreuzberg | pdf-reader-mcp | Nutrient DWS | jztan/pdf-mcp | mcp-pdf-utils | **mcp-pdf v3** |
|------------|:---------:|:--------------:|:------------:|:-------------:|:-------------:|:--------------:|
| Total tools | ~8 | 1 | 6 | 8 | 11 | **131** |
| Inspect/classify/repair | partial | ❌ | ❌ | partial | partial | ✅ |
| Structured text extraction | ✅ | ✅ | ✅ | ✅ | basic | ✅ |
| Table extraction | ✅ | ❌ | ✅ | ✅ | ❌ | ✅ |
| OCR (multi-language) | ✅ | ❌ | ✅ (cloud) | ✅ | ❌ | ✅ (local) |
| Hybrid/semantic search | embeddings | ❌ | ❌ | ✅ (BM25+semantic) | ❌ | ✅ |
| AI summarization + Q&A | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Document Intelligence (IDP) | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Format conversion (PDF↔Office) | ❌ | ❌ | ✅ (cloud) | ❌ | ❌ | ✅ (local) |
| Professional generation (styled) | ❌ | ❌ | ❌ | ❌ | basic | ✅ (8 styles) |
| True redaction + verification | ❌ | ❌ | ✅ (cloud) | ❌ | ❌ | ✅ (local) |
| Digital signatures | ❌ | ❌ | ✅ (cloud) | ❌ | ❌ | ✅ (local) |
| Accessibility (PDF/UA) | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| PDF/A archival | ❌ | ❌ | ✅ | ❌ | ❌ | ✅ |
| Bates numbering | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Batch workflows | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Contract comparison | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Form filling + validation | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Merge/split/rotate | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ |
| Watermark | ❌ | ❌ | ✅ | ❌ | ✅ | ✅ |
| Local-first / zero-cloud | ✅ | ✅ | ❌ (cloud API) | ✅ | ✅ | ✅ |
| Modular (feature flags) | ✅ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Logo/branding support | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Multi-page auto-layout | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |

### Key Observations

1. **kreuzberg** is the closest competitor — strong extraction + OCR + embeddings, but has **zero generation, zero manipulation, zero security/redaction, zero forms**. It's a read-only intelligence layer.

2. **Nutrient DWS** (PSPDFKit) is feature-rich but **cloud-only, requires API key, costs money**. Not local-first. Not agent-autonomous.

3. **jztan/pdf-mcp** has the best search (hybrid BM25+semantic) but **no generation, no manipulation, no security**. Read-only.

4. **No existing server combines read + write + intelligence + security + generation + batch.** That's our gap.

5. **No existing server generates professional styled documents.** mcp-pdf-utils can create basic text PDFs. @mcp-z/mcp-pdf can generate resumes. Neither has a style system, logo support, or multiple document types.

---

## Implementation Plan

### Phase 1 — Core Foundation (v3.0.0) — Weeks 1–3

**Ship: 40 tools (core, no external deps)**

- Pillar 1: Inspect & Understand (8 tools)
- Pillar 2: Extract Content (14 tools)
- Pillar 8: Page & File Operations (12 tools)
- Pillar 9: Headers, Footers & Numbering (4 tools)
- Style system with 4 styles (minimal, modern, corporate, stripe)
- `create_invoice` with beautiful output (already working)

**Goal:** Reliable local-first PDF foundation that already beats every competitor.

### Phase 2 — Generation & Security (v3.1.0) — Weeks 4–6

**Ship: 38 additional tools**

- Pillar 7: Document Generation (14 tools)
- Pillar 11: Security & Compliance (12 tools)
- Pillar 10: Forms (first 9 tools, no signatures yet)
- Pillar 13: Batch & Workflows (3 tools: batch_process, create_workflow, run_workflow)
- Add 4 more styles (legal, academic, government, finance)

**Goal:** Full generation platform + enterprise security.

### Phase 3 — AI Intelligence (v3.2.0) — Weeks 7–10

**Ship: 23 additional tools (requires `ai` feature)**

- Pillar 4: AI Intelligence (14 tools)
- Pillar 5: Document Intelligence / IDP (9 tools)
- RAG pipeline with local embedding
- Configurable LLM backend

**Goal:** Turn PDFs from static files into queryable knowledge.

### Phase 4 — OCR & Conversion (v3.3.0) — Weeks 11–14

**Ship: 16 additional tools**

- Pillar 3: OCR & Scan Processing (6 tools)
- Pillar 6: Format Conversion (10 tools)
- Tesseract 5 integration
- LibreOffice headless bridge

**Goal:** Handle the paper-based world and format interop.

### Phase 5 — Accessibility, Signatures & Polish (v3.4.0) — Weeks 15–18

**Ship: 14 additional tools**

- Pillar 12: Accessibility & Standards (10 tools)
- Pillar 10: Signatures (3 remaining tools)
- Pillar 13: Remaining batch tools (3 tools)
- PDF/A, PDF/UA validation and conversion
- Full test suite

**Goal:** Legal, government, and enterprise compliance.

---

## MVP (Phase 1 Deliverable) — 40 Tools

These 40 tools ship first and already make mcp-pdf the most capable PDF MCP in existence:

**Inspect (8):** inspect_pdf, classify_pdf, health_check_pdf, detect_features, profile_complexity, get_page_count, get_info, repair_pdf

**Extract (14):** extract_text, extract_text_structured, extract_page_text, extract_layout, extract_tables, extract_key_values, extract_images, extract_figures, extract_annotations, extract_bookmarks, extract_attachments, extract_barcodes, extract_metadata, extract_reading_order

**Manipulate (12):** merge_pdfs, split_pdf, split_by_bookmarks, rotate_pages, crop_pages, resize_pages, reorder_pages, delete_pages, insert_pages, compress_pdf, linearize_pdf, add_watermark

**Numbering (4):** add_page_numbers, add_bates_numbers, add_headers_footers, add_table_of_contents

**+ create_invoice, create_receipt** (from Generation pillar, to demonstrate style system)

---

## Technical Stack Summary

| Layer | Crate/Tool | Purpose |
|-------|-----------|---------|
| PDF parsing | `lopdf` 0.40 | Read, manipulate, write PDF structure |
| PDF generation | `printpdf` 0.7 | Create new PDFs with graphics, text, images |
| Text extraction | `pdf-extract` 0.10 | Extract text content |
| Image handling | `image` 0.24 | Load/convert images (RGB for Preview compat) |
| OCR | Tesseract 5 (system) | 100+ language OCR |
| Conversion | LibreOffice (system) | Office ↔ PDF |
| AI/LLM | `reqwest` + provider APIs | Intelligence layer |
| Signatures | `openssl` | X.509 digital signatures |
| Search/index | `tantivy` | Local full-text + semantic search |
| Hashing | `sha2` | Integrity verification |
| MCP transport | `rmcp` 1.7 | Server framework |

---

## What This Means

The v1 mcp-pdf was a utility (20 tools, basic read/write).
The v2 proposal was a document generator (32 tools, styled output).
**v3 is a PDF operating system** — 131 tools that handle the full lifecycle of PDF documents across every industry, with AI intelligence, enterprise security, accessibility compliance, and batch automation.

No other MCP server — on any registry, in any language — comes close.
