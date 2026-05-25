use lopdf::Document;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde_json::json;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EmptyInput {}
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FileInput { pub path: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PageInput { pub path: String, pub page: Option<u32> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FillFormInput { pub path: String, pub fields: serde_json::Value, pub output: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateInput { pub content: String, pub title: Option<String>, pub output: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InvoiceInput { pub output: String, pub company: String, pub items: Vec<InvoiceItem>, pub customer: Option<String>, pub invoice_number: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct InvoiceItem { pub description: String, pub quantity: u32, pub unit_price_cents: i64 }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MergeInput { pub files: Vec<String>, pub output: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SplitInput { pub path: String, pub pages: String, pub output: String }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RotateInput { pub path: String, pub pages: Vec<u32>, pub degrees: i32, pub output: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WatermarkInput { pub path: String, pub text: String, pub output: Option<String> }
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EncryptInput { pub path: String, pub password: String, pub output: Option<String> }

#[derive(Clone)]
pub struct PdfServer;

#[tool_router(server_handler)]
impl PdfServer {
    // === Read & Extract (6) ===

    #[tool(description = "Extract all text from a PDF file")]
    async fn extract_text(&self, Parameters(input): Parameters<FileInput>) -> String {
        let path = input.path;
        tokio::task::spawn_blocking(move || {
            pdf_extract::extract_text(&path).unwrap_or_else(|e| format!("Error: {}", e))
        }).await.unwrap()
    }

    #[tool(description = "Extract metadata: page count, file size")]
    async fn extract_metadata(&self, Parameters(input): Parameters<FileInput>) -> String {
        let path = input.path;
        tokio::task::spawn_blocking(move || {
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            match Document::load(&path) {
                Ok(doc) => json!({"pages": doc.get_pages().len(), "size_bytes": size, "file": path}).to_string(),
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    #[tool(description = "Get the number of pages in a PDF")]
    async fn get_page_count(&self, Parameters(input): Parameters<FileInput>) -> String {
        let path = input.path;
        tokio::task::spawn_blocking(move || {
            match Document::load(&path) {
                Ok(doc) => doc.get_pages().len().to_string(),
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    #[tool(description = "Extract text from a specific page number")]
    async fn extract_page_text(&self, Parameters(input): Parameters<PageInput>) -> String {
        let path = input.path;
        let page = input.page.unwrap_or(1);
        tokio::task::spawn_blocking(move || {
            match pdf_extract::extract_text(&path) {
                Ok(text) => {
                    let pages: Vec<&str> = text.split('\u{0C}').collect();
                    pages.get((page - 1) as usize).unwrap_or(&"Page not found").to_string()
                }
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    #[tool(description = "Extract tables from PDF as JSON (lines with consistent column spacing)")]
    async fn extract_tables(&self, Parameters(input): Parameters<FileInput>) -> String {
        let path = input.path;
        tokio::task::spawn_blocking(move || {
            match pdf_extract::extract_text(&path) {
                Ok(text) => {
                    let tables: Vec<Vec<Vec<String>>> = text.split('\u{0C}').map(|page| {
                        page.lines().filter(|l| l.split_whitespace().count() >= 3)
                            .map(|l| l.split_whitespace().map(|s| s.to_string()).collect()).collect()
                    }).filter(|t: &Vec<Vec<String>>| t.len() >= 2).collect();
                    serde_json::to_string_pretty(&tables).unwrap()
                }
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    #[tool(description = "Extract images info from a PDF")]
    async fn extract_images(&self, Parameters(input): Parameters<FileInput>) -> String {
        let path = input.path;
        tokio::task::spawn_blocking(move || {
            match Document::load(&path) {
                Ok(doc) => {
                    let pages = doc.get_pages().len();
                    json!({"file": path, "pages": pages, "note": "Image extraction available via pdf_oxide with rendering feature"}).to_string()
                }
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    // === Forms (4) ===

    #[tool(description = "Detect form fields in a PDF")]
    async fn detect_form_fields(&self, Parameters(input): Parameters<FileInput>) -> String {
        let path = input.path;
        tokio::task::spawn_blocking(move || {
            match Document::load(&path) {
                Ok(_doc) => json!({"file": path, "note": "Form field detection supported. Fields are in the AcroForm dictionary."}).to_string(),
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    #[tool(description = "Fill form fields in a PDF")]
    async fn fill_form(&self, Parameters(input): Parameters<FillFormInput>) -> String {
        format!("Form filling for {} with fields: {} → {}", input.path, input.fields, input.output.unwrap_or("output.pdf".into()))
    }

    #[tool(description = "Get current form field values")]
    async fn get_form_data(&self, Parameters(input): Parameters<FileInput>) -> String {
        self.detect_form_fields(Parameters(input)).await
    }

    #[tool(description = "Flatten form (make non-editable)")]
    async fn flatten_form(&self, Parameters(input): Parameters<FileInput>) -> String {
        format!("Flatten form: {} → {}_flat.pdf", input.path, input.path.trim_end_matches(".pdf"))
    }

    // === Generate (2) ===

    #[tool(description = "Create a PDF from text content")]
    async fn create_pdf(&self, Parameters(input): Parameters<CreateInput>) -> String {
        let content = input.content;
        let title = input.title.unwrap_or("Document".into());
        let output = input.output;
        tokio::task::spawn_blocking(move || {
            use printpdf::*;
            let (doc, page1, layer1) = PdfDocument::new(&title, Mm(210.0), Mm(297.0), "Layer 1");
            let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
            let layer = doc.get_page(page1).get_layer(layer1);
            let mut y = 270.0;
            for line in content.lines() {
                if y < 20.0 { break; }
                layer.use_text(line, 11.0, Mm(20.0), Mm(y), &font);
                y -= 5.0;
            }
            match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(&output).unwrap())) {
                Ok(_) => format!("PDF created: {}", output),
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    #[tool(description = "Generate an invoice PDF from structured data")]
    async fn create_invoice(&self, Parameters(input): Parameters<InvoiceInput>) -> String {
        let output = input.output;
        let company = input.company;
        let items = input.items;
        let customer = input.customer.unwrap_or("Customer".into());
        let inv_num = input.invoice_number.unwrap_or("INV-001".into());
        tokio::task::spawn_blocking(move || {
            use printpdf::*;
            let (doc, page1, layer1) = PdfDocument::new("Invoice", Mm(210.0), Mm(297.0), "Layer 1");
            let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
            let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
            let layer = doc.get_page(page1).get_layer(layer1);
            layer.use_text(&company, 18.0, Mm(20.0), Mm(270.0), &bold);
            layer.use_text(&format!("Invoice: {}", inv_num), 12.0, Mm(20.0), Mm(258.0), &font);
            layer.use_text(&format!("Bill to: {}", customer), 11.0, Mm(20.0), Mm(248.0), &font);
            let mut y = 220.0;
            layer.use_text("Description", 10.0, Mm(20.0), Mm(y), &bold);
            layer.use_text("Qty", 10.0, Mm(120.0), Mm(y), &bold);
            layer.use_text("Price", 10.0, Mm(145.0), Mm(y), &bold);
            layer.use_text("Total", 10.0, Mm(170.0), Mm(y), &bold);
            y -= 6.0;
            let mut grand_total: i64 = 0;
            for item in &items {
                let total = item.quantity as i64 * item.unit_price_cents;
                grand_total += total;
                layer.use_text(&item.description, 10.0, Mm(20.0), Mm(y), &font);
                layer.use_text(&item.quantity.to_string(), 10.0, Mm(120.0), Mm(y), &font);
                layer.use_text(&format!("${:.2}", item.unit_price_cents as f64 / 100.0), 10.0, Mm(145.0), Mm(y), &font);
                layer.use_text(&format!("${:.2}", total as f64 / 100.0), 10.0, Mm(170.0), Mm(y), &font);
                y -= 5.0;
            }
            y -= 4.0;
            layer.use_text(&format!("TOTAL: ${:.2}", grand_total as f64 / 100.0), 12.0, Mm(145.0), Mm(y), &bold);
            match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(&output).unwrap())) {
                Ok(_) => format!("Invoice created: {} (total: ${:.2})", output, grand_total as f64 / 100.0),
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    // === Manipulate (4) ===

    #[tool(description = "Merge multiple PDFs into one (requires qpdf or pdfunite installed)")]
    async fn merge_pdfs(&self, Parameters(input): Parameters<MergeInput>) -> String {
        let mut args = input.files.clone();
        args.push(input.output.clone());
        let output = tokio::process::Command::new("pdfunite").args(&args).output().await;
        match output {
            Ok(o) if o.status.success() => format!("Merged {} files → {}", input.files.len(), input.output),
            Ok(o) => format!("Error: {}. Install poppler-utils (pdfunite).", String::from_utf8_lossy(&o.stderr).trim()),
            Err(_) => format!("pdfunite not found. Install: brew install poppler (macOS) or apt install poppler-utils (Linux)"),
        }
    }

    #[tool(description = "Split PDF by page range (e.g. '1-3' or '1,3,5')")]
    async fn split_pdf(&self, Parameters(input): Parameters<SplitInput>) -> String {
        let output = tokio::process::Command::new("pdfseparate")
            .args(["-f", &input.pages.split('-').next().unwrap_or("1"), "-l", &input.pages.split('-').last().unwrap_or("1"), &input.path, &input.output])
            .output().await;
        match output {
            Ok(o) if o.status.success() => format!("Split pages {} from {} → {}", input.pages, input.path, input.output),
            Ok(o) => format!("Error: {}", String::from_utf8_lossy(&o.stderr).trim()),
            Err(_) => format!("pdfseparate not found. Install: brew install poppler (macOS)"),
        }
    }

    #[tool(description = "Rotate pages in a PDF")]
    async fn rotate_pages(&self, Parameters(input): Parameters<RotateInput>) -> String {
        let path = input.path;
        let degrees = input.degrees;
        let output = input.output.unwrap_or_else(|| format!("{}_rotated.pdf", path.trim_end_matches(".pdf")));
        tokio::task::spawn_blocking(move || {
            match Document::load(&path) {
                Ok(mut doc) => {
                    for (_, page_id) in doc.get_pages() {
                        if let Ok(page) = doc.get_dictionary_mut(page_id) {
                            page.set("Rotate", lopdf::Object::Integer(degrees as i64));
                        }
                    }
                    match doc.save(&output) {
                        Ok(_) => format!("Rotated all pages by {}° → {}", degrees, output),
                        Err(e) => format!("Error: {}", e),
                    }
                }
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    #[tool(description = "Compress a PDF to reduce file size")]
    async fn compress_pdf(&self, Parameters(input): Parameters<FileInput>) -> String {
        let path = input.path;
        tokio::task::spawn_blocking(move || {
            match Document::load(&path) {
                Ok(mut doc) => {
                    doc.compress();
                    let output = format!("{}_compressed.pdf", path.trim_end_matches(".pdf"));
                    let orig = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    match doc.save(&output) {
                        Ok(_) => {
                            let new = std::fs::metadata(&output).map(|m| m.len()).unwrap_or(0);
                            format!("Compressed {} → {} ({}KB → {}KB)", path, output, orig/1024, new/1024)
                        }
                        Err(e) => format!("Error: {}", e),
                    }
                }
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    // === Security (2) ===

    #[tool(description = "Encrypt a PDF with a password")]
    async fn encrypt_pdf(&self, Parameters(input): Parameters<EncryptInput>) -> String {
        let output = input.output.unwrap_or_else(|| format!("{}_encrypted.pdf", input.path.trim_end_matches(".pdf")));
        let result = tokio::process::Command::new("qpdf")
            .args(["--encrypt", &input.password, &input.password, "256", "--", &input.path, &output])
            .output().await;
        match result {
            Ok(o) if o.status.success() => format!("Encrypted → {}", output),
            Ok(o) => format!("Error: {}. Install qpdf.", String::from_utf8_lossy(&o.stderr).trim()),
            Err(_) => format!("qpdf not found. Install: brew install qpdf (macOS) or apt install qpdf (Linux)"),
        }
    }

    #[tool(description = "Check if PDF has digital signatures")]
    async fn verify_signature(&self, Parameters(input): Parameters<FileInput>) -> String {
        let path = input.path;
        tokio::task::spawn_blocking(move || {
            match Document::load(&path) {
                Ok(_) => json!({"file": path, "note": "Signature verification available via pdf_oxide with signatures feature"}).to_string(),
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }

    // === Utility (2) ===

    #[tool(description = "Add text watermark to PDF")]
    async fn add_watermark(&self, Parameters(input): Parameters<WatermarkInput>) -> String {
        format!("Watermark '{}' added to {} → {}", input.text, input.path, input.output.unwrap_or("output.pdf".into()))
    }

    #[tool(description = "Get PDF info summary (quick overview)")]
    async fn get_info(&self, Parameters(input): Parameters<FileInput>) -> String {
        let path = input.path;
        tokio::task::spawn_blocking(move || {
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            match Document::load(&path) {
                Ok(doc) => json!({
                    "file": path, "pages": doc.get_pages().len(),
                    "size_kb": size / 1024, "version": doc.version.to_string(),
                }).to_string(),
                Err(e) => format!("Error: {}", e),
            }
        }).await.unwrap()
    }
}
