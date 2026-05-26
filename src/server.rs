use rmcp::{tool, tool_router, schemars};
use rmcp::handler::server::wrapper::Parameters;
use serde::Deserialize;
use crate::tools::{inspect, extract, manipulate, numbering, generate, security, forms};

#[derive(Clone)]
pub struct PdfServer;

// --- Input structs ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PdfPathInput { pub pdf_path: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RepairInput { pub pdf_path: String, pub output: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PageTextInput { pub pdf_path: String, pub page_number: u32 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeInput { pub pdf_paths: Vec<String>, pub output: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SplitInput { pub pdf_path: String, pub pages: String, pub output: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RotateInput { pub pdf_path: String, pub output: String, pub degrees: u32 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OutputInput { pub pdf_path: String, pub output: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeletePagesInput { pub pdf_path: String, pub output: String, pub pages: Vec<u32> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReorderInput { pub pdf_path: String, pub output: String, pub order: Vec<u32> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CropInput { pub pdf_path: String, pub output: String, pub crop_box: [f32; 4] }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WatermarkInput { pub pdf_path: String, pub output: String, pub text: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PageNumbersInput { pub pdf_path: String, pub output: String, pub position: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatesInput { pub pdf_path: String, pub output: String, pub prefix: String, pub start: u32, pub digits: Option<u32> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InvoiceInput {
    pub output: String,
    pub company: String,
    pub items: Vec<InvoiceItem>,
    pub customer: Option<String>,
    pub invoice_number: Option<String>,
    pub logo: Option<String>,
    pub style: Option<String>,
    /// Data to encode as QR code (e.g., payment URL, invoice reference)
    pub qr_data: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InvoiceItem { pub description: String, pub quantity: u32, pub unit_price_cents: i64 }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReceiptInput {
    pub output: String,
    pub company: String,
    pub customer: String,
    pub items: Vec<InvoiceItem>,
    pub receipt_number: Option<String>,
    pub payment_method: Option<String>,
    pub logo: Option<String>,
    /// Stamp text: "received", "paid", "void", or custom
    pub stamp: Option<String>,
    /// Stamp style: "circle" (default) or "rectangle"
    pub stamp_style: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LetterInput {
    pub output: String,
    pub from_name: String,
    pub from_company: Option<String>,
    pub to_name: String,
    pub to_company: Option<String>,
    pub subject: Option<String>,
    pub body: String,
    pub logo: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CertificateInput {
    pub output: String,
    pub recipient: String,
    pub title: String,
    pub description: Option<String>,
    pub issuer: String,
    pub date: Option<String>,
    /// Style: classic, modern, elegant, academic, minimal (default: classic)
    pub style: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReportInput {
    pub output: String,
    pub title: String,
    pub author: Option<String>,
    pub sections: Vec<ReportSection>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReportSection { pub heading: String, pub body: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ContractInput {
    pub output: String,
    pub title: String,
    pub parties: Vec<String>,
    pub effective_date: String,
    pub clauses: Vec<ContractClause>,
    pub signatures: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ContractClause { pub title: String, pub body: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EncryptInput { pub pdf_path: String, pub output: String, pub owner_password: String, pub user_password: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScanSensitiveInput { pub pdf_path: String, pub categories: Option<Vec<String>> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RedactInput {
    pub pdf_path: String,
    pub output: String,
    pub terms: Vec<String>,
    /// "black" (default, █ blocks) or "space" (whitespace)
    pub mode: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FillFormInput {
    pub pdf_path: String,
    pub output: String,
    pub field_values: serde_json::Value,
}

// --- Tool Router ---

#[tool_router(server_handler)]
impl PdfServer {
    // === PILLAR 1: Inspect ===
    #[tool(description = "Full structural profile of a PDF: pages, fonts, images, forms, signatures, encryption")]
    async fn inspect_pdf(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        inspect::inspect_pdf(&input.pdf_path)
    }

    #[tool(description = "Get page count of a PDF")]
    async fn get_page_count(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        inspect::get_page_count(&input.pdf_path)
    }

    #[tool(description = "Quick metadata: pages, size, version, encryption, title, author")]
    async fn get_info(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        inspect::get_info(&input.pdf_path)
    }

    #[tool(description = "Classify document type: invoice, contract, form, scan, report, letter, certificate")]
    async fn classify_pdf(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        inspect::classify_pdf(&input.pdf_path)
    }

    #[tool(description = "Check PDF health: corruption, xref errors, broken objects")]
    async fn health_check_pdf(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        inspect::health_check_pdf(&input.pdf_path)
    }

    #[tool(description = "Detect features: forms, tags, signatures, JavaScript, embedded files")]
    async fn detect_features(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        inspect::detect_features(&input.pdf_path)
    }

    #[tool(description = "Rate extraction difficulty: simple to extreme (1-5 scale)")]
    async fn profile_complexity(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        inspect::profile_complexity(&input.pdf_path)
    }

    #[tool(description = "Repair corrupted PDF: rebuild xref, fix streams")]
    async fn repair_pdf(&self, Parameters(input): Parameters<RepairInput>) -> String {
        inspect::repair_pdf(&input.pdf_path, &input.output)
    }

    // === PILLAR 2: Extract ===
    #[tool(description = "Extract all text from a PDF")]
    async fn extract_text(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        extract::extract_text(&input.pdf_path)
    }

    #[tool(description = "Extract text from a specific page")]
    async fn extract_page_text(&self, Parameters(input): Parameters<PageTextInput>) -> String {
        extract::extract_page_text(&input.pdf_path, input.page_number)
    }

    #[tool(description = "Extract document metadata: title, author, dates, creator, producer")]
    async fn extract_metadata(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        extract::extract_metadata(&input.pdf_path)
    }

    #[tool(description = "Extract tables as JSON arrays with headers and rows")]
    async fn extract_tables(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        extract::extract_tables(&input.pdf_path)
    }

    #[tool(description = "Extract embedded images info from PDF")]
    async fn extract_images(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        extract::extract_images(&input.pdf_path)
    }

    #[tool(description = "Extract bookmark/outline tree")]
    async fn extract_bookmarks(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        extract::extract_bookmarks(&input.pdf_path)
    }

    #[tool(description = "Extract annotations: comments, highlights, stamps")]
    async fn extract_annotations(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        extract::extract_annotations(&input.pdf_path)
    }

    #[tool(description = "Extract key-value pairs (label: value patterns)")]
    async fn extract_key_values(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        extract::extract_key_values(&input.pdf_path)
    }

    // === PILLAR 8: Manipulate ===
    #[tool(description = "Merge multiple PDFs into one")]
    async fn merge_pdfs(&self, Parameters(input): Parameters<MergeInput>) -> String {
        manipulate::merge_pdfs(&input.pdf_paths, &input.output)
    }

    #[tool(description = "Split PDF by page ranges into a new file")]
    async fn split_pdf(&self, Parameters(input): Parameters<SplitInput>) -> String {
        manipulate::split_pdf(&input.pdf_path, &input.pages, &input.output)
    }

    #[tool(description = "Rotate pages by 90, 180, or 270 degrees")]
    async fn rotate_pages(&self, Parameters(input): Parameters<RotateInput>) -> String {
        manipulate::rotate_pages(&input.pdf_path, &input.output, input.degrees)
    }

    #[tool(description = "Compress PDF to reduce file size")]
    async fn compress_pdf(&self, Parameters(input): Parameters<OutputInput>) -> String {
        manipulate::compress_pdf(&input.pdf_path, &input.output)
    }

    #[tool(description = "Delete specified pages from a PDF")]
    async fn delete_pages(&self, Parameters(input): Parameters<DeletePagesInput>) -> String {
        manipulate::delete_pages(&input.pdf_path, &input.output, &input.pages)
    }

    #[tool(description = "Reorder pages by index array")]
    async fn reorder_pages(&self, Parameters(input): Parameters<ReorderInput>) -> String {
        manipulate::reorder_pages(&input.pdf_path, &input.output, &input.order)
    }

    #[tool(description = "Crop pages to a bounding box [left, bottom, right, top] in points")]
    async fn crop_pages(&self, Parameters(input): Parameters<CropInput>) -> String {
        manipulate::crop_pages(&input.pdf_path, &input.output, &input.crop_box)
    }

    #[tool(description = "Add text watermark to all pages (e.g., CONFIDENTIAL, DRAFT)")]
    async fn add_watermark(&self, Parameters(input): Parameters<WatermarkInput>) -> String {
        manipulate::add_watermark(&input.pdf_path, &input.output, input.text.as_deref())
    }

    // === PILLAR 9: Numbering ===
    #[tool(description = "Add page numbers to all pages")]
    async fn add_page_numbers(&self, Parameters(input): Parameters<PageNumbersInput>) -> String {
        numbering::add_page_numbers(&input.pdf_path, &input.output, input.position.as_deref())
    }

    #[tool(description = "Add Bates numbering for legal documents")]
    async fn add_bates_numbers(&self, Parameters(input): Parameters<BatesInput>) -> String {
        numbering::add_bates_numbers(&input.pdf_path, &input.output, &input.prefix, input.start, input.digits)
    }

    // === PILLAR 7: Generate ===
    #[tool(description = "Generate professional invoice PDF with line items, tax, logo, style")]
    async fn create_invoice(&self, Parameters(input): Parameters<InvoiceInput>) -> String {
        let data = generate::InvoiceData {
            output: input.output,
            company: input.company,
            items: input.items.into_iter().map(|i| (i.description, i.quantity, i.unit_price_cents)).collect(),
            customer: input.customer.unwrap_or("Customer".into()),
            invoice_number: input.invoice_number.unwrap_or("INV-001".into()),
            logo: input.logo,
            style: input.style.unwrap_or("minimal".into()),
            qr_data: input.qr_data,
        };
        generate::create_invoice(data)
    }

    #[tool(description = "Generate payment receipt PDF")]
    async fn create_receipt(&self, Parameters(input): Parameters<ReceiptInput>) -> String {
        let data = generate::ReceiptData {
            output: input.output, company: input.company, customer: input.customer,
            receipt_number: input.receipt_number.unwrap_or("REC-001".into()),
            items: input.items.into_iter().map(|i| (i.description, i.quantity, i.unit_price_cents)).collect(),
            payment_method: input.payment_method.unwrap_or("Card".into()),
            _logo: input.logo,
            stamp: input.stamp,
            stamp_style: input.stamp_style.unwrap_or("circle".into()),
        };
        generate::create_receipt(data)
    }

    #[tool(description = "Generate business letter with letterhead")]
    async fn create_letter(&self, Parameters(input): Parameters<LetterInput>) -> String {
        let data = generate::LetterData {
            output: input.output, from_name: input.from_name, from_company: input.from_company,
            to_name: input.to_name, to_company: input.to_company,
            subject: input.subject, body: input.body, _logo: input.logo,
        };
        generate::create_letter(data)
    }

    #[tool(description = "Generate certificate (styles: classic, modern, elegant, academic, minimal)")]
    async fn create_certificate(&self, Parameters(input): Parameters<CertificateInput>) -> String {
        let data = generate::CertificateData {
            output: input.output, recipient: input.recipient, title: input.title,
            description: input.description, issuer: input.issuer, date: input.date,
            style: input.style.unwrap_or("classic".into()),
        };
        generate::create_certificate(data)
    }

    #[tool(description = "Generate multi-section report PDF")]
    async fn create_report(&self, Parameters(input): Parameters<ReportInput>) -> String {
        let data = generate::ReportData {
            output: input.output, title: input.title, author: input.author,
            sections: input.sections.into_iter().map(|s| (s.heading, s.body)).collect(),
        };
        generate::create_report(data)
    }

    #[tool(description = "Generate contract with clauses and signature blocks")]
    async fn create_contract(&self, Parameters(input): Parameters<ContractInput>) -> String {
        let data = generate::ContractData {
            output: input.output, title: input.title, parties: input.parties,
            effective_date: input.effective_date,
            clauses: input.clauses.into_iter().map(|c| (c.title, c.body)).collect(),
            signatures: input.signatures,
        };
        generate::create_contract(data)
    }

    // === PILLAR 11: Security ===
    #[tool(description = "SHA-256 hash of a PDF for integrity verification")]
    async fn hash_pdf(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        security::hash_pdf(&input.pdf_path)
    }

    #[tool(description = "Encrypt PDF with password protection")]
    async fn encrypt_pdf(&self, Parameters(input): Parameters<EncryptInput>) -> String {
        security::encrypt_pdf(&input.pdf_path, &input.output, &input.owner_password, input.user_password.as_deref())
    }

    #[tool(description = "Scan PDF for sensitive data: emails, phones, SSNs, credit cards")]
    async fn scan_sensitive_data(&self, Parameters(input): Parameters<ScanSensitiveInput>) -> String {
        security::scan_sensitive_data(&input.pdf_path, input.categories.as_deref())
    }

    #[tool(description = "Redact terms from PDF (true redaction: removes from content streams + strips metadata)")]
    async fn redact_pdf(&self, Parameters(input): Parameters<RedactInput>) -> String {
        security::redact_pdf(&input.pdf_path, &input.output, &input.terms, input.mode.as_deref())
    }

    // === PILLAR 10: Forms ===
    #[tool(description = "Detect form fields in a PDF: names, types, current values")]
    async fn detect_form_fields(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        forms::detect_form_fields(&input.pdf_path)
    }

    #[tool(description = "Fill form fields by name-value map")]
    async fn fill_form(&self, Parameters(input): Parameters<FillFormInput>) -> String {
        forms::fill_form(&input.pdf_path, &input.output, &input.field_values)
    }

    #[tool(description = "Flatten form (make fields non-editable, burn values into page)")]
    async fn flatten_form(&self, Parameters(input): Parameters<OutputInput>) -> String {
        forms::flatten_form(&input.pdf_path, &input.output)
    }
}
