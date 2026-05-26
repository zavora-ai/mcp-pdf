pub fn inspect_pdf(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            let pages = doc.get_pages();
            let page_count = pages.len();
            let has_forms = doc.catalog().ok().and_then(|c| c.get(b"AcroForm").ok()).is_some();
            let has_encryption = doc.trailer.get(b"Encrypt").is_ok();
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
            let has_bookmarks = doc.catalog().ok().and_then(|c| c.get(b"Outlines").ok()).is_some();
            let file_size = std::fs::metadata(pdf_path).map(|m| m.len()).unwrap_or(0);
            serde_json::json!({
                "path": pdf_path, "file_size_bytes": file_size, "pdf_version": &doc.version,
                "page_count": page_count, "fonts": fonts, "has_forms": has_forms,
                "has_encryption": has_encryption, "has_bookmarks": has_bookmarks,
            }).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn get_page_count(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => doc.get_pages().len().to_string(),
        Err(e) => format!("error: {}", e),
    }
}

pub fn get_info(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            let pages = doc.get_pages().len();
            let encrypted = doc.trailer.get(b"Encrypt").is_ok();
            let file_size = std::fs::metadata(pdf_path).map(|m| m.len()).unwrap_or(0);
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
                "page_count": pages, "file_size_bytes": file_size, "pdf_version": &doc.version,
                "encrypted": encrypted, "title": title, "author": author,
            }).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn classify_pdf(pdf_path: &str) -> String {
    let text = pdf_extract::extract_text(pdf_path).map(|t| t.to_lowercase()).unwrap_or_default();
    let doc = lopdf::Document::load(pdf_path).ok();
    let has_forms = doc.as_ref().and_then(|d| d.catalog().ok()).and_then(|c| c.get(b"AcroForm").ok()).is_some();
    let is_scanned = text.len() < 50 && doc.as_ref().map(|d| d.get_pages().len()).unwrap_or(0) > 0;
    let classification = if has_forms { "form" }
        else if is_scanned { "scan" }
        else if text.contains("invoice") || text.contains("bill to") || text.contains("amount due") { "invoice" }
        else if text.contains("agreement") || text.contains("whereas") || text.contains("hereby") { "contract" }
        else if text.contains("certificate") && text.contains("awarded") { "certificate" }
        else if text.contains("dear") && text.len() < 3000 { "letter" }
        else if text.contains("table of contents") || text.contains("executive summary") { "report" }
        else { "unknown" };
    serde_json::json!({"classification": classification, "is_scanned": is_scanned, "has_selectable_text": !text.is_empty(), "has_forms": has_forms}).to_string()
}

pub fn health_check_pdf(pdf_path: &str) -> String {
    if !std::path::Path::new(pdf_path).exists() {
        return serde_json::json!({"status": "error", "issues": ["File not found"]}).to_string();
    }
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            let mut issues = Vec::<String>::new();
            let page_count = doc.get_pages().len();
            if page_count == 0 { issues.push("No pages found".into()); }
            let mut pages_ok = 0;
            for (_, &page_id) in &doc.get_pages() {
                if doc.get_dictionary(page_id).is_ok() { pages_ok += 1; }
                else { issues.push(format!("Unreadable page object {:?}", page_id)); }
            }
            let status = if issues.is_empty() { "healthy" } else { "warnings" };
            serde_json::json!({"status": status, "page_count": page_count, "pages_readable": pages_ok, "issues": issues}).to_string()
        }
        Err(e) => serde_json::json!({"status": "corrupted", "issues": [format!("Failed to load: {}", e)], "repairable": false}).to_string(),
    }
}

pub fn detect_features(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            let catalog = doc.catalog().ok();
            let has_forms = catalog.and_then(|c| c.get(b"AcroForm").ok()).is_some();
            let has_tags = catalog.and_then(|c| c.get(b"MarkInfo").ok()).is_some();
            let has_bookmarks = catalog.and_then(|c| c.get(b"Outlines").ok()).is_some();
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
                                            if subtype == b"Widget" || subtype == b"Sig" { has_signatures = true; }
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
                "has_forms": has_forms, "has_tags": has_tags, "has_signatures": has_signatures,
                "has_bookmarks": has_bookmarks, "has_encryption": has_encryption,
                "has_annotations": annotation_count > 0, "annotation_count": annotation_count,
            }).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn profile_complexity(pdf_path: &str) -> String {
    let text_len = pdf_extract::extract_text(pdf_path).map(|t| t.len()).unwrap_or(0);
    let doc = lopdf::Document::load(pdf_path).ok();
    let page_count = doc.as_ref().map(|d| d.get_pages().len()).unwrap_or(0);
    let is_scanned = text_len < 50 && page_count > 0;
    let mut score = 1u8;
    if page_count > 10 { score += 1; }
    if page_count > 50 { score += 1; }
    if is_scanned { score += 2; }
    if text_len > 50000 { score += 1; }
    let score = score.min(5);
    let level = match score { 1 => "simple", 2 => "moderate", 3 => "complex", 4 => "very_complex", _ => "extreme" };
    serde_json::json!({"complexity_score": score, "level": level, "page_count": page_count, "estimated_text_length": text_len, "is_scanned": is_scanned}).to_string()
}

pub fn repair_pdf(pdf_path: &str, output: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            doc.renumber_objects();
            doc.compress();
            match doc.save(output) {
                Ok(_) => serde_json::json!({"status": "repaired", "output": output, "pages": doc.get_pages().len()}).to_string(),
                Err(e) => serde_json::json!({"error": format!("Save failed: {}", e)}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"status": "unrecoverable", "error": e.to_string()}).to_string(),
    }
}
