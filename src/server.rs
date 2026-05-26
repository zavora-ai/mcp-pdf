use rmcp::{tool, tool_router, schemars};
use rmcp::handler::server::wrapper::Parameters;
use serde::Deserialize;
use crate::tools::{inspect, extract, manipulate, numbering, generate};

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
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InvoiceItem { pub description: String, pub quantity: u32, pub unit_price_cents: i64 }

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
        };
        generate::create_invoice(data)
    }
}
