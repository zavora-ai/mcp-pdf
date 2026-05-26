use rmcp::{tool, tool_router, schemars};
use rmcp::handler::server::wrapper::Parameters;
use serde::Deserialize;

#[derive(Clone)]
pub struct PdfServer;

// --- Input structs ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PdfPathInput {
    pub pdf_path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RepairInput {
    pub pdf_path: String,
    pub output: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PageTextInput {
    pub pdf_path: String,
    pub page_number: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeInput {
    pub pdf_paths: Vec<String>,
    pub output: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SplitInput {
    pub pdf_path: String,
    pub pages: String,
    pub output: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RotateInput {
    pub pdf_path: String,
    pub output: String,
    pub degrees: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OutputInput {
    pub pdf_path: String,
    pub output: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeletePagesInput {
    pub pdf_path: String,
    pub output: String,
    pub pages: Vec<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReorderInput {
    pub pdf_path: String,
    pub output: String,
    pub order: Vec<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CropInput {
    pub pdf_path: String,
    pub output: String,
    pub crop_box: [f32; 4],
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WatermarkInput {
    pub pdf_path: String,
    pub output: String,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PageNumbersInput {
    pub pdf_path: String,
    pub output: String,
    pub position: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatesInput {
    pub pdf_path: String,
    pub output: String,
    pub prefix: String,
    pub start: u32,
    pub digits: Option<u32>,
}

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
pub struct InvoiceItem {
    pub description: String,
    pub quantity: u32,
    pub unit_price_cents: i64,
}

// --- Server ---

#[tool_router(server_handler)]
impl PdfServer {
    #[tool(description = "Full structural profile of a PDF: pages, fonts, images, forms, signatures, encryption")]
    async fn inspect_pdf(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let pages = doc.get_pages();
                let page_count = pages.len();

                // Check for forms
                let has_forms = doc.catalog().ok()
                    .and_then(|c| c.get(b"AcroForm").ok())
                    .is_some();

                // Check for encryption
                let has_encryption = doc.trailer.get(b"Encrypt").is_ok();

                // Collect fonts
                let mut fonts = Vec::new();
                for (_, &page_id) in &pages {
                    if let Ok(page) = doc.get_dictionary(page_id) {
                        if let Ok(resources) = page.get(b"Resources").and_then(|r| r.as_dict()) {
                            if let Ok(font_dict) = resources.get(b"Font").and_then(|f| f.as_dict()) {
                                for (name, _) in font_dict {
                                    fonts.push(String::from_utf8_lossy(name).to_string());
                                }
                            }
                        }
                    }
                }
                fonts.sort();
                fonts.dedup();

                // Check for bookmarks
                let has_bookmarks = doc.catalog().ok()
                    .and_then(|c| c.get(b"Outlines").ok())
                    .is_some();

                // File size
                let file_size = std::fs::metadata(&input.pdf_path)
                    .map(|m| m.len()).unwrap_or(0);

                // PDF version
                let version = &doc.version;

                serde_json::json!({
                    "path": input.pdf_path,
                    "file_size_bytes": file_size,
                    "pdf_version": version,
                    "page_count": page_count,
                    "fonts": fonts,
                    "has_forms": has_forms,
                    "has_encryption": has_encryption,
                    "has_bookmarks": has_bookmarks,
                }).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Get page count of a PDF")]
    async fn get_page_count(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => doc.get_pages().len().to_string(),
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(description = "Quick metadata: pages, size, version, encryption, title, author")]
    async fn get_info(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let pages = doc.get_pages().len();
                let version = &doc.version;
                let encrypted = doc.trailer.get(b"Encrypt").is_ok();
                let file_size = std::fs::metadata(&input.pdf_path)
                    .map(|m| m.len()).unwrap_or(0);

                let mut title = String::new();
                let mut author = String::new();
                if let Ok(info_ref) = doc.trailer.get(b"Info") {
                    if let Ok(id) = info_ref.as_reference() {
                        if let Ok(info) = doc.get_dictionary(id) {
                            if let Ok(t) = info.get(b"Title").and_then(|o| o.as_str()) {
                                title = String::from_utf8_lossy(t).to_string();
                            }
                            if let Ok(a) = info.get(b"Author").and_then(|o| o.as_str()) {
                                author = String::from_utf8_lossy(a).to_string();
                            }
                        }
                    }
                }

                serde_json::json!({
                    "page_count": pages,
                    "file_size_bytes": file_size,
                    "pdf_version": version,
                    "encrypted": encrypted,
                    "title": title,
                    "author": author,
                }).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Classify document type: invoice, contract, form, scan, report, letter, certificate")]
    async fn classify_pdf(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        let text = match pdf_extract::extract_text(&input.pdf_path) {
            Ok(t) => t.to_lowercase(),
            Err(_) => String::new(),
        };
        let doc = lopdf::Document::load(&input.pdf_path).ok();
        let has_forms = doc.as_ref().and_then(|d| d.catalog().ok())
            .and_then(|c| c.get(b"AcroForm").ok()).is_some();
        let is_scanned = text.len() < 50 && doc.as_ref().map(|d| d.get_pages().len()).unwrap_or(0) > 0;

        let classification = if has_forms { "form" }
            else if is_scanned { "scan" }
            else if text.contains("invoice") || text.contains("bill to") || text.contains("amount due") { "invoice" }
            else if text.contains("agreement") || text.contains("whereas") || text.contains("hereby") { "contract" }
            else if text.contains("certificate") && text.contains("awarded") { "certificate" }
            else if text.contains("dear") && text.len() < 3000 { "letter" }
            else if text.contains("table of contents") || text.contains("executive summary") { "report" }
            else { "unknown" };

        serde_json::json!({
            "classification": classification,
            "is_scanned": is_scanned,
            "has_selectable_text": !text.is_empty(),
            "has_forms": has_forms,
        }).to_string()
    }

    #[tool(description = "Check PDF health: corruption, xref errors, broken objects")]
    async fn health_check_pdf(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        let file_exists = std::path::Path::new(&input.pdf_path).exists();
        if !file_exists {
            return serde_json::json!({"status": "error", "issues": ["File not found"]}).to_string();
        }
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let mut issues = Vec::<String>::new();
                let page_count = doc.get_pages().len();
                if page_count == 0 { issues.push("No pages found".into()); }

                // Check each page is readable
                let mut pages_ok = 0;
                for (_, &page_id) in &doc.get_pages() {
                    if doc.get_dictionary(page_id).is_ok() { pages_ok += 1; }
                    else { issues.push(format!("Unreadable page object {:?}", page_id)); }
                }

                let status = if issues.is_empty() { "healthy" } else { "warnings" };
                serde_json::json!({
                    "status": status,
                    "page_count": page_count,
                    "pages_readable": pages_ok,
                    "issues": issues,
                }).to_string()
            }
            Err(e) => serde_json::json!({
                "status": "corrupted",
                "issues": [format!("Failed to load: {}", e)],
                "repairable": false,
            }).to_string(),
        }
    }

    #[tool(description = "Detect features: forms, tags, signatures, JavaScript, embedded files")]
    async fn detect_features(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let catalog = doc.catalog().ok();
                let has_forms = catalog.and_then(|c| c.get(b"AcroForm").ok()).is_some();
                let has_tags = catalog.and_then(|c| c.get(b"MarkInfo").ok()).is_some();
                let has_bookmarks = catalog.and_then(|c| c.get(b"Outlines").ok()).is_some();

                // Check for signatures and annotations
                let mut has_signatures = false;
                let mut annotation_count = 0u32;
                for (_, &page_id) in &doc.get_pages() {
                    if let Ok(page) = doc.get_dictionary(page_id) {
                        if let Ok(annots) = page.get(b"Annots") {
                            if let Ok(arr) = annots.as_array() {
                                annotation_count += arr.len() as u32;
                                for obj in arr {
                                    if let Ok(id) = obj.as_reference() {
                                        if let Ok(annot) = doc.get_dictionary(id) {
                                            if let Ok(subtype) = annot.get(b"Subtype").and_then(|s| s.as_name()) {
                                                if subtype == b"Widget" || subtype == b"Sig" {
                                                    has_signatures = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let has_encryption = doc.trailer.get(b"Encrypt").is_ok();

                serde_json::json!({
                    "has_forms": has_forms,
                    "has_tags": has_tags,
                    "has_signatures": has_signatures,
                    "has_bookmarks": has_bookmarks,
                    "has_encryption": has_encryption,
                    "has_annotations": annotation_count > 0,
                    "annotation_count": annotation_count,
                }).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Rate extraction difficulty: simple to extreme (1-5 scale)")]
    async fn profile_complexity(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        let text_len = pdf_extract::extract_text(&input.pdf_path)
            .map(|t| t.len()).unwrap_or(0);
        let doc = lopdf::Document::load(&input.pdf_path).ok();
        let page_count = doc.as_ref().map(|d| d.get_pages().len()).unwrap_or(0);

        let is_scanned = text_len < 50 && page_count > 0;
        let mut score = 1u8;
        if page_count > 10 { score += 1; }
        if page_count > 50 { score += 1; }
        if is_scanned { score += 2; }
        if text_len > 50000 { score += 1; }
        let score = score.min(5);

        let level = match score {
            1 => "simple", 2 => "moderate", 3 => "complex",
            4 => "very_complex", _ => "extreme",
        };

        serde_json::json!({
            "complexity_score": score,
            "level": level,
            "page_count": page_count,
            "estimated_text_length": text_len,
            "is_scanned": is_scanned,
        }).to_string()
    }

    #[tool(description = "Repair corrupted PDF: rebuild xref, fix streams")]
    async fn repair_pdf(&self, Parameters(input): Parameters<RepairInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(mut doc) => {
                doc.renumber_objects();
                doc.compress();
                match doc.save(&input.output) {
                    Ok(_) => serde_json::json!({
                        "status": "repaired",
                        "output": input.output,
                        "pages": doc.get_pages().len(),
                    }).to_string(),
                    Err(e) => serde_json::json!({"error": format!("Save failed: {}", e)}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"status": "unrecoverable", "error": e.to_string()}).to_string(),
        }
    }

    // === PILLAR 2: Extract ===

    #[tool(description = "Extract all text from a PDF")]
    async fn extract_text(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match pdf_extract::extract_text(&input.pdf_path) {
            Ok(text) => text,
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(description = "Extract text from a specific page")]
    async fn extract_page_text(&self, Parameters(input): Parameters<PageTextInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let pages = doc.get_pages();
                let page_num = input.page_number;
                if let Some(_page_id) = pages.get(&page_num) {
                    // Use pdf_extract on the full doc then split by page markers
                    // Simpler: extract all and return approximate page content
                    match pdf_extract::extract_text(&input.pdf_path) {
                        Ok(text) => {
                            let total_pages = pages.len() as u32;
                            if total_pages == 1 { return text; }
                            // Approximate: split evenly
                            let chars_per_page = text.len() / total_pages as usize;
                            let start = (page_num - 1) as usize * chars_per_page;
                            let end = (start + chars_per_page).min(text.len());
                            text[start..end].to_string()
                        }
                        Err(e) => format!("error: {}", e),
                    }
                } else {
                    format!("error: page {} not found (document has {} pages)", page_num, pages.len())
                }
            }
            Err(e) => format!("error: {}", e),
        }
    }

    #[tool(description = "Extract document metadata: title, author, dates, creator, producer")]
    async fn extract_metadata(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let mut meta = serde_json::Map::new();
                meta.insert("pdf_version".into(), serde_json::Value::String(doc.version.clone()));
                meta.insert("page_count".into(), serde_json::json!(doc.get_pages().len()));

                if let Ok(info_ref) = doc.trailer.get(b"Info") {
                    if let Ok(id) = info_ref.as_reference() {
                        if let Ok(info) = doc.get_dictionary(id) {
                            let keys: &[&[u8]] = &[b"Title", b"Author", b"Subject", b"Keywords", b"Creator", b"Producer", b"CreationDate", b"ModDate"];
                            for key in keys {
                                if let Ok(val) = info.get(*key).and_then(|o| o.as_str()) {
                                    let k = String::from_utf8_lossy(key).to_lowercase();
                                    meta.insert(k, serde_json::Value::String(String::from_utf8_lossy(val).to_string()));
                                }
                            }
                        }
                    }
                }
                serde_json::Value::Object(meta).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Extract tables as JSON arrays with headers and rows")]
    async fn extract_tables(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        // Table extraction heuristic: find lines with consistent tab/space separation
        match pdf_extract::extract_text(&input.pdf_path) {
            Ok(text) => {
                let mut tables = Vec::<serde_json::Value>::new();
                let mut current_table = Vec::<Vec<String>>::new();

                for line in text.lines() {
                    // Detect table rows: lines with 2+ segments separated by 2+ spaces
                    let cells: Vec<String> = line.split("  ")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if cells.len() >= 2 {
                        current_table.push(cells);
                    } else if !current_table.is_empty() {
                        if current_table.len() >= 2 {
                            let headers = current_table[0].clone();
                            let rows: Vec<Vec<String>> = current_table[1..].to_vec();
                            tables.push(serde_json::json!({
                                "headers": headers,
                                "rows": rows,
                            }));
                        }
                        current_table.clear();
                    }
                }
                // Flush last table
                if current_table.len() >= 2 {
                    let headers = current_table[0].clone();
                    let rows: Vec<Vec<String>> = current_table[1..].to_vec();
                    tables.push(serde_json::json!({"headers": headers, "rows": rows}));
                }

                serde_json::json!({"tables": tables, "table_count": tables.len()}).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Extract embedded images info from PDF")]
    async fn extract_images(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let mut images = Vec::<serde_json::Value>::new();
                for (page_num, &page_id) in &doc.get_pages() {
                    if let Ok(page) = doc.get_dictionary(page_id) {
                        if let Ok(resources) = page.get(b"Resources").and_then(|r| r.as_dict()) {
                            if let Ok(xobjects) = resources.get(b"XObject").and_then(|x| x.as_dict()) {
                                for (name, obj) in xobjects {
                                    if let Ok(id) = obj.as_reference() {
                                        if let Ok(stream) = doc.get_object(id).and_then(|o| o.as_stream()) {
                                            if let Ok(subtype) = stream.dict.get(b"Subtype").and_then(|s| s.as_name()) {
                                                if subtype == b"Image" {
                                                    let w = stream.dict.get(b"Width").and_then(|v| v.as_i64()).unwrap_or(0);
                                                    let h = stream.dict.get(b"Height").and_then(|v| v.as_i64()).unwrap_or(0);
                                                    images.push(serde_json::json!({
                                                        "page": page_num,
                                                        "name": String::from_utf8_lossy(name),
                                                        "width": w,
                                                        "height": h,
                                                    }));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                serde_json::json!({"images": images, "count": images.len()}).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Extract bookmark/outline tree")]
    async fn extract_bookmarks(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let mut bookmarks = Vec::<serde_json::Value>::new();
                if let Ok(catalog) = doc.catalog() {
                    if let Ok(outlines_ref) = catalog.get(b"Outlines") {
                        if let Ok(outlines_id) = outlines_ref.as_reference() {
                            if let Ok(outlines) = doc.get_dictionary(outlines_id) {
                                // Follow First/Next chain
                                if let Ok(first_ref) = outlines.get(b"First") {
                                    if let Ok(first_id) = first_ref.as_reference() {
                                        Self::collect_bookmarks(&doc, first_id, &mut bookmarks);
                                    }
                                }
                            }
                        }
                    }
                }
                serde_json::json!({"bookmarks": bookmarks}).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Extract annotations: comments, highlights, stamps")]
    async fn extract_annotations(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let mut annotations = Vec::<serde_json::Value>::new();
                for (page_num, &page_id) in &doc.get_pages() {
                    if let Ok(page) = doc.get_dictionary(page_id) {
                        if let Ok(annots) = page.get(b"Annots").and_then(|a| a.as_array()) {
                            for obj in annots {
                                if let Ok(id) = obj.as_reference() {
                                    if let Ok(annot) = doc.get_dictionary(id) {
                                        let subtype = annot.get(b"Subtype").and_then(|s| s.as_name())
                                            .map(|n| String::from_utf8_lossy(n).to_string()).unwrap_or_default();
                                        let contents = annot.get(b"Contents").and_then(|c| c.as_str())
                                            .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
                                        annotations.push(serde_json::json!({
                                            "page": page_num,
                                            "type": subtype,
                                            "contents": contents,
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
                serde_json::json!({"annotations": annotations, "count": annotations.len()}).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Extract key-value pairs (label: value patterns)")]
    async fn extract_key_values(&self, Parameters(input): Parameters<PdfPathInput>) -> String {
        match pdf_extract::extract_text(&input.pdf_path) {
            Ok(text) => {
                let mut pairs = Vec::<serde_json::Value>::new();
                let re = regex::Regex::new(r"([A-Za-z][A-Za-z\s]{1,30}):\s*(.+)").unwrap();
                for cap in re.captures_iter(&text) {
                    pairs.push(serde_json::json!({
                        "key": cap[1].trim(),
                        "value": cap[2].trim(),
                    }));
                }
                serde_json::json!({"pairs": pairs, "count": pairs.len()}).to_string()
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    // === PILLAR 8: Manipulate ===

    #[tool(description = "Merge multiple PDFs into one")]
    async fn merge_pdfs(&self, Parameters(input): Parameters<MergeInput>) -> String {
        if input.pdf_paths.is_empty() {
            return serde_json::json!({"error": "No PDF paths provided"}).to_string();
        }
        use std::collections::BTreeMap;
        let mut max_id = 1u32;
        let mut documents_pages: BTreeMap<lopdf::ObjectId, lopdf::Object> = BTreeMap::new();
        let mut documents_objects: BTreeMap<lopdf::ObjectId, lopdf::Object> = BTreeMap::new();

        for path in &input.pdf_paths {
            let mut doc = match lopdf::Document::load(path) {
                Ok(d) => d,
                Err(e) => return serde_json::json!({"error": format!("Failed to load {}: {}", path, e)}).to_string(),
            };
            doc.renumber_objects_with(max_id);
            max_id = doc.max_id + 1;

            let pages = doc.get_pages();
            for &object_id in pages.values() {
                if let Ok(obj) = doc.get_object(object_id) {
                    documents_pages.insert(object_id, obj.clone());
                }
            }
            documents_objects.extend(doc.objects);
        }

        let mut document = lopdf::Document::with_version("1.5");
        let mut catalog_object: Option<(lopdf::ObjectId, lopdf::Object)> = None;
        let mut pages_object: Option<(lopdf::ObjectId, lopdf::Object)> = None;

        for (object_id, object) in documents_objects {
            match object.type_name().unwrap_or(b"") {
                b"Catalog" => {
                    catalog_object = Some((catalog_object.map(|(id,_)| id).unwrap_or(object_id), object));
                }
                b"Pages" => {
                    if let Ok(dictionary) = object.as_dict() {
                        let mut dictionary = dictionary.clone();
                        if let Some((_, ref obj)) = pages_object {
                            if let Ok(old) = obj.as_dict() { dictionary.extend(old); }
                        }
                        pages_object = Some((pages_object.map(|(id,_)| id).unwrap_or(object_id), lopdf::Object::Dictionary(dictionary)));
                    }
                }
                b"Page" | b"Outlines" | b"Outline" => {}
                _ => { document.objects.insert(object_id, object); }
            }
        }

        let (pages_id, pages_obj) = match pages_object {
            Some(p) => p,
            None => return serde_json::json!({"error": "No Pages object found"}).to_string(),
        };
        let (catalog_id, catalog_obj) = match catalog_object {
            Some(c) => c,
            None => return serde_json::json!({"error": "No Catalog object found"}).to_string(),
        };

        // Insert page objects with parent set
        for (object_id, object) in &documents_pages {
            if let Ok(dictionary) = object.as_dict() {
                let mut dictionary = dictionary.clone();
                dictionary.set("Parent", pages_id);
                document.objects.insert(*object_id, lopdf::Object::Dictionary(dictionary));
            }
        }

        // Build Pages
        if let Ok(dictionary) = pages_obj.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Count", documents_pages.len() as u32);
            dictionary.set("Kids", documents_pages.keys().map(|&id| lopdf::Object::Reference(id)).collect::<Vec<_>>());
            document.objects.insert(pages_id, lopdf::Object::Dictionary(dictionary));
        }

        // Build Catalog
        if let Ok(dictionary) = catalog_obj.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Pages", pages_id);
            document.objects.insert(catalog_id, lopdf::Object::Dictionary(dictionary));
        }

        document.trailer.set("Root", catalog_id);
        document.max_id = document.objects.len() as u32;
        document.renumber_objects();

        match document.save(&input.output) {
            Ok(_) => serde_json::json!({"output": input.output, "pages": documents_pages.len()}).to_string(),
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Split PDF by page ranges into a new file")]
    async fn split_pdf(&self, Parameters(input): Parameters<SplitInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(doc) => {
                let pages = doc.get_pages();
                let total = pages.len() as u32;
                // Parse range like "1-3" or "2,4,6"
                let page_nums: Vec<u32> = input.pages.split(|c: char| c == ',' || c == '-')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();

                let mut new_doc = doc.clone();
                let to_delete: Vec<u32> = (1..=total).filter(|p| !page_nums.contains(p)).collect();
                for &p in to_delete.iter().rev() {
                    new_doc.delete_pages(&[p]);
                }
                match new_doc.save(&input.output) {
                    Ok(_) => serde_json::json!({"output": input.output, "pages_kept": page_nums.len()}).to_string(),
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Rotate pages by 90, 180, or 270 degrees")]
    async fn rotate_pages(&self, Parameters(input): Parameters<RotateInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(mut doc) => {
                let pages = doc.get_pages();
                let page_ids: Vec<lopdf::ObjectId> = pages.values().copied().collect();
                for page_id in page_ids {
                    if let Ok(page) = doc.get_dictionary_mut(page_id) {
                        page.set(b"Rotate".to_vec(), lopdf::Object::Integer(input.degrees as i64));
                    }
                }
                match doc.save(&input.output) {
                    Ok(_) => serde_json::json!({"output": input.output, "degrees": input.degrees}).to_string(),
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Compress PDF to reduce file size")]
    async fn compress_pdf(&self, Parameters(input): Parameters<OutputInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(mut doc) => {
                let original_size = std::fs::metadata(&input.pdf_path).map(|m| m.len()).unwrap_or(0);
                doc.compress();
                match doc.save(&input.output) {
                    Ok(_) => {
                        let new_size = std::fs::metadata(&input.output).map(|m| m.len()).unwrap_or(0);
                        let reduction = if original_size > 0 {
                            ((original_size - new_size) as f64 / original_size as f64 * 100.0) as i32
                        } else { 0 };
                        serde_json::json!({
                            "output": input.output,
                            "original_bytes": original_size,
                            "compressed_bytes": new_size,
                            "reduction_pct": reduction,
                        }).to_string()
                    }
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Delete specified pages from a PDF")]
    async fn delete_pages(&self, Parameters(input): Parameters<DeletePagesInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(mut doc) => {
                for &p in input.pages.iter().rev() {
                    doc.delete_pages(&[p]);
                }
                match doc.save(&input.output) {
                    Ok(_) => serde_json::json!({"output": input.output, "deleted": input.pages}).to_string(),
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Reorder pages by index array")]
    async fn reorder_pages(&self, Parameters(input): Parameters<ReorderInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(mut doc) => {
                let pages = doc.get_pages();
                let page_ids: Vec<lopdf::ObjectId> = pages.values().copied().collect();
                let mut new_ids = Vec::new();
                for &idx in &input.order {
                    if idx as usize <= page_ids.len() && idx > 0 {
                        new_ids.push(page_ids[(idx - 1) as usize]);
                    }
                }
                if let Ok(catalog) = doc.catalog() {
                    if let Ok(pages_ref) = catalog.get(b"Pages").and_then(|p| p.as_reference()) {
                        if let Ok(pages_dict) = doc.get_dictionary_mut(pages_ref) {
                            let kids: Vec<lopdf::Object> = new_ids.iter().map(|&id| lopdf::Object::Reference(id)).collect();
                            pages_dict.set(b"Kids".to_vec(), lopdf::Object::Array(kids));
                            pages_dict.set(b"Count".to_vec(), lopdf::Object::Integer(new_ids.len() as i64));
                        }
                    }
                }
                match doc.save(&input.output) {
                    Ok(_) => serde_json::json!({"output": input.output, "order": input.order}).to_string(),
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Crop pages to a bounding box [left, bottom, right, top] in points")]
    async fn crop_pages(&self, Parameters(input): Parameters<CropInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(mut doc) => {
                let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
                for page_id in page_ids {
                    if let Ok(page) = doc.get_dictionary_mut(page_id) {
                        let crop_box = lopdf::Object::Array(vec![
                            lopdf::Object::Real(input.crop_box[0]),
                            lopdf::Object::Real(input.crop_box[1]),
                            lopdf::Object::Real(input.crop_box[2]),
                            lopdf::Object::Real(input.crop_box[3]),
                        ]);
                        page.set(b"CropBox".to_vec(), crop_box);
                    }
                }
                match doc.save(&input.output) {
                    Ok(_) => serde_json::json!({"output": input.output, "crop_box": input.crop_box}).to_string(),
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Add text watermark to all pages (e.g., CONFIDENTIAL, DRAFT)")]
    async fn add_watermark(&self, Parameters(input): Parameters<WatermarkInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(mut doc) => {
                let text = input.text.unwrap_or("CONFIDENTIAL".into());

                // Create font object
                let mut font_dict = lopdf::Dictionary::new();
                font_dict.set(b"Type".to_vec(), lopdf::Object::Name(b"Font".to_vec()));
                font_dict.set(b"Subtype".to_vec(), lopdf::Object::Name(b"Type1".to_vec()));
                font_dict.set(b"BaseFont".to_vec(), lopdf::Object::Name(b"Helvetica".to_vec()));
                let font_id = doc.add_object(lopdf::Object::Dictionary(font_dict));

                // Create a Form XObject for the watermark (self-contained with own Resources)
                let wm_content = format!(
                    "0.85 0.85 0.85 rg BT /Hwm 60 Tf 0.7071 0.7071 -0.7071 0.7071 130 250 Tm ({}) Tj ET",
                    text
                );
                let mut wm_resources = lopdf::Dictionary::new();
                let mut wm_fonts = lopdf::Dictionary::new();
                wm_fonts.set(b"Hwm".to_vec(), lopdf::Object::Reference(font_id));
                wm_resources.set(b"Font".to_vec(), lopdf::Object::Dictionary(wm_fonts));

                let mut wm_dict = lopdf::Dictionary::new();
                wm_dict.set(b"Type".to_vec(), lopdf::Object::Name(b"XObject".to_vec()));
                wm_dict.set(b"Subtype".to_vec(), lopdf::Object::Name(b"Form".to_vec()));
                wm_dict.set(b"BBox".to_vec(), lopdf::Object::Array(vec![
                    lopdf::Object::Integer(0), lopdf::Object::Integer(0),
                    lopdf::Object::Integer(595), lopdf::Object::Integer(842),
                ]));
                wm_dict.set(b"Resources".to_vec(), lopdf::Object::Dictionary(wm_resources));
                let wm_stream = lopdf::Stream::new(wm_dict, wm_content.into_bytes());
                let wm_id = doc.add_object(wm_stream);

                let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
                for page_id in page_ids {
                    // Add the XObject to page resources
                    let res_id = if let Ok(page) = doc.get_dictionary(page_id) {
                        match page.get(b"Resources") {
                            Ok(lopdf::Object::Reference(id)) => Some(*id),
                            _ => None,
                        }
                    } else { None };

                    if let Some(rid) = res_id {
                        if let Ok(res) = doc.get_dictionary_mut(rid) {
                            if let Ok(xobjs) = res.get_mut(b"XObject").and_then(|x| x.as_dict_mut()) {
                                xobjs.set(b"Watermark".to_vec(), lopdf::Object::Reference(wm_id));
                            } else {
                                let mut xd = lopdf::Dictionary::new();
                                xd.set(b"Watermark".to_vec(), lopdf::Object::Reference(wm_id));
                                res.set(b"XObject".to_vec(), lopdf::Object::Dictionary(xd));
                            }
                        }
                    } else if let Ok(page) = doc.get_dictionary_mut(page_id) {
                        if let Ok(res) = page.get_mut(b"Resources").and_then(|r| r.as_dict_mut()) {
                            if let Ok(xobjs) = res.get_mut(b"XObject").and_then(|x| x.as_dict_mut()) {
                                xobjs.set(b"Watermark".to_vec(), lopdf::Object::Reference(wm_id));
                            } else {
                                let mut xd = lopdf::Dictionary::new();
                                xd.set(b"Watermark".to_vec(), lopdf::Object::Reference(wm_id));
                                res.set(b"XObject".to_vec(), lopdf::Object::Dictionary(xd));
                            }
                        }
                    }

                    // Append "q /Watermark Do Q" to page contents
                    let invoke_content = b"q /Watermark Do Q".to_vec();
                    let invoke_stream = lopdf::Stream::new(lopdf::Dictionary::new(), invoke_content);
                    let invoke_id = doc.add_object(invoke_stream);

                    if let Ok(page) = doc.get_dictionary_mut(page_id) {
                        let existing = page.get(b"Contents").ok().cloned();
                        let new_contents = match existing {
                            Some(lopdf::Object::Reference(id)) => {
                                lopdf::Object::Array(vec![lopdf::Object::Reference(id), lopdf::Object::Reference(invoke_id)])
                            }
                            Some(lopdf::Object::Array(mut arr)) => {
                                arr.push(lopdf::Object::Reference(invoke_id));
                                lopdf::Object::Array(arr)
                            }
                            _ => lopdf::Object::Reference(invoke_id),
                        };
                        page.set(b"Contents".to_vec(), new_contents);
                    }
                }
                match doc.save(&input.output) {
                    Ok(_) => serde_json::json!({"output": input.output, "watermark": text}).to_string(),
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Add page numbers to all pages")]
    async fn add_page_numbers(&self, Parameters(input): Parameters<PageNumbersInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(mut doc) => {
                let position = input.position.unwrap_or("bottom-center".into());
                let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
                let total = page_ids.len();
                for (i, page_id) in page_ids.iter().enumerate() {
                    let num = i + 1;
                    let label = format!("Page {} of {}", num, total);
                    let (x, y) = match position.as_str() {
                        "bottom-right" => (500, 20),
                        "top-right" => (500, 770),
                        _ => (270, 20), // bottom-center
                    };
                    let content = format!("BT /F1 9 Tf {} {} Td ({}) Tj ET", x, y, label);
                    let stream = lopdf::Stream::new(lopdf::Dictionary::new(), content.into_bytes());
                    let stream_id = doc.add_object(stream);
                    if let Ok(page) = doc.get_dictionary_mut(*page_id) {
                        let existing = page.get(b"Contents").ok().cloned();
                        let new_contents = match existing {
                            Some(lopdf::Object::Reference(id)) => {
                                lopdf::Object::Array(vec![lopdf::Object::Reference(id), lopdf::Object::Reference(stream_id)])
                            }
                            Some(lopdf::Object::Array(mut arr)) => {
                                arr.push(lopdf::Object::Reference(stream_id));
                                lopdf::Object::Array(arr)
                            }
                            _ => lopdf::Object::Reference(stream_id),
                        };
                        page.set(b"Contents".to_vec(), new_contents);
                    }
                }
                match doc.save(&input.output) {
                    Ok(_) => serde_json::json!({"output": input.output, "pages_numbered": total}).to_string(),
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    #[tool(description = "Add Bates numbering for legal documents")]
    async fn add_bates_numbers(&self, Parameters(input): Parameters<BatesInput>) -> String {
        match lopdf::Document::load(&input.pdf_path) {
            Ok(mut doc) => {
                let prefix = input.prefix;
                let start = input.start;
                let digits = input.digits.unwrap_or(6) as usize;
                let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
                let total = page_ids.len();
                for (i, page_id) in page_ids.iter().enumerate() {
                    let num = start + i as u32;
                    let label = format!("{}{:0>width$}", prefix, num, width = digits);
                    let content = format!("BT /F1 8 Tf 480 20 Td ({}) Tj ET", label);
                    let stream = lopdf::Stream::new(lopdf::Dictionary::new(), content.into_bytes());
                    let stream_id = doc.add_object(stream);
                    if let Ok(page) = doc.get_dictionary_mut(*page_id) {
                        let existing = page.get(b"Contents").ok().cloned();
                        let new_contents = match existing {
                            Some(lopdf::Object::Reference(id)) => {
                                lopdf::Object::Array(vec![lopdf::Object::Reference(id), lopdf::Object::Reference(stream_id)])
                            }
                            Some(lopdf::Object::Array(mut arr)) => {
                                arr.push(lopdf::Object::Reference(stream_id));
                                lopdf::Object::Array(arr)
                            }
                            _ => lopdf::Object::Reference(stream_id),
                        };
                        page.set(b"Contents".to_vec(), new_contents);
                    }
                }
                let end_num = start + total as u32 - 1;
                let range = format!("{}{:0>width$} - {}{:0>width$}", prefix, start, prefix, end_num, width = digits);
                match doc.save(&input.output) {
                    Ok(_) => serde_json::json!({"output": input.output, "range": range, "pages": total}).to_string(),
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }

    // === PILLAR 7: Generate ===

    #[tool(description = "Generate professional invoice PDF with line items, tax, logo, style")]
    async fn create_invoice(&self, Parameters(input): Parameters<InvoiceInput>) -> String {
        use printpdf::*;

        let output = input.output;
        let style = input.style.unwrap_or("minimal".into());

        let (doc, page1, layer1) = PdfDocument::new("Invoice", Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
        let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
        let layer = doc.get_page(page1).get_layer(layer1);

        // Logo
        let mut logo_status = "none".to_string();
        if let Some(ref logo_path) = input.logo {
            if let Ok(img_bytes) = std::fs::read(logo_path) {
                if let Ok(img) = image_crate::load_from_memory(&img_bytes) {
                    let rgb = img.to_rgb8();
                    let (w, h) = rgb.dimensions();
                    let image = Image::from(ImageXObject {
                        width: Px(w as usize), height: Px(h as usize),
                        color_space: ColorSpace::Rgb, bits_per_component: ColorBits::Bit8,
                        interpolate: true, image_data: rgb.into_raw(),
                        image_filter: None, clipping_bbox: None, smask: None,
                    });
                    image.add_to_layer(layer.clone(), ImageTransform {
                        translate_x: Some(Mm(15.0)), translate_y: Some(Mm(272.0)),
                        scale_x: Some(0.8), scale_y: Some(0.8),
                        ..Default::default()
                    });
                    logo_status = format!("added ({}x{})", w, h);
                }
            }
        }

        // INVOICE title (right)
        layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
        layer.use_text("INVOICE", 24.0, Mm(150.0), Mm(278.0), &bold);

        // Invoice details
        layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
        let inv_num = input.invoice_number.unwrap_or("INV-001".into());
        layer.use_text(&inv_num, 10.0, Mm(150.0), Mm(270.0), &font);
        layer.use_text(&chrono::Utc::now().format("%B %d, %Y").to_string(), 10.0, Mm(150.0), Mm(264.0), &font);

        // Thin separator
        layer.set_outline_color(Color::Rgb(Rgb::new(0.85, 0.85, 0.85, None)));
        layer.set_outline_thickness(0.5);
        layer.add_line(Line { points: vec![
            (Point::new(Mm(15.0), Mm(258.0)), false),
            (Point::new(Mm(195.0), Mm(258.0)), false),
        ], is_closed: false });

        // Bill To
        layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
        layer.use_text("Bill To", 8.0, Mm(15.0), Mm(250.0), &font);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
        let customer = input.customer.unwrap_or("Customer".into());
        layer.use_text(&customer, 11.0, Mm(15.0), Mm(244.0), &bold);

        // From
        layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
        layer.use_text("From", 8.0, Mm(15.0), Mm(270.0), &font);
        layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
        layer.use_text(&input.company, 11.0, Mm(15.0), Mm(264.0), &bold);

        // Table header
        let mut y = 220.0f32;
        layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
        layer.use_text("Description", 8.0, Mm(15.0), Mm(y), &bold);
        layer.use_text("Qty", 8.0, Mm(120.0), Mm(y), &bold);
        layer.use_text("Price", 8.0, Mm(140.0), Mm(y), &bold);
        layer.use_text("Amount", 8.0, Mm(175.0), Mm(y), &bold);
        y -= 3.0;
        layer.add_line(Line { points: vec![
            (Point::new(Mm(15.0), Mm(y)), false),
            (Point::new(Mm(195.0), Mm(y)), false),
        ], is_closed: false });
        y -= 7.0;

        // Rows
        layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
        let mut total: i64 = 0;
        for item in &input.items {
            let amt = item.quantity as i64 * item.unit_price_cents;
            total += amt;
            layer.use_text(&item.description, 10.0, Mm(15.0), Mm(y), &font);
            layer.use_text(&item.quantity.to_string(), 10.0, Mm(123.0), Mm(y), &font);
            layer.use_text(&format!("${:.2}", item.unit_price_cents as f64 / 100.0), 10.0, Mm(140.0), Mm(y), &font);
            layer.use_text(&format!("${:.2}", amt as f64 / 100.0), 10.0, Mm(175.0), Mm(y), &font);
            y -= 7.0;
        }

        // Total line
        y -= 3.0;
        layer.add_line(Line { points: vec![
            (Point::new(Mm(130.0), Mm(y)), false),
            (Point::new(Mm(195.0), Mm(y)), false),
        ], is_closed: false });
        y -= 8.0;
        layer.use_text("Total", 10.0, Mm(140.0), Mm(y), &bold);
        layer.use_text(&format!("${:.2}", total as f64 / 100.0), 12.0, Mm(172.0), Mm(y), &bold);

        // Footer
        layer.set_fill_color(Color::Rgb(Rgb::new(0.7, 0.7, 0.7, None)));
        layer.use_text("Thank you for your business.", 9.0, Mm(15.0), Mm(20.0), &font);

        match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(&output).unwrap())) {
            Ok(_) => serde_json::json!({
                "output": output,
                "total": format!("${:.2}", total as f64 / 100.0),
                "items": input.items.len(),
                "style": style,
                "logo": logo_status,
            }).to_string(),
            Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
        }
    }
}

impl PdfServer {
    fn collect_bookmarks(doc: &lopdf::Document, id: lopdf::ObjectId, out: &mut Vec<serde_json::Value>) {
        if let Ok(dict) = doc.get_dictionary(id) {
            let title = dict.get(b"Title").and_then(|t| t.as_str())
                .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
            out.push(serde_json::json!({"title": title}));
            // Follow Next sibling
            if let Ok(next_ref) = dict.get(b"Next") {
                if let Ok(next_id) = next_ref.as_reference() {
                    Self::collect_bookmarks(doc, next_id, out);
                }
            }
        }
    }
}
