pub fn extract_text(pdf_path: &str) -> String {
    match pdf_extract::extract_text(pdf_path) {
        Ok(text) => text,
        Err(e) => format!("error: {}", e),
    }
}

pub fn extract_page_text(pdf_path: &str, page_number: u32) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            let pages = doc.get_pages();
            if pages.get(&page_number).is_none() {
                return format!("error: page {} not found (document has {} pages)", page_number, pages.len());
            }
            match pdf_extract::extract_text(pdf_path) {
                Ok(text) => {
                    let total_pages = pages.len() as u32;
                    if total_pages == 1 { return text; }
                    let chars_per_page = text.len() / total_pages as usize;
                    let start = (page_number - 1) as usize * chars_per_page;
                    let end = (start + chars_per_page).min(text.len());
                    text[start..end].to_string()
                }
                Err(e) => format!("error: {}", e),
            }
        }
        Err(e) => format!("error: {}", e),
    }
}

pub fn extract_metadata(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
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

pub fn extract_tables(pdf_path: &str) -> String {
    match pdf_extract::extract_text(pdf_path) {
        Ok(text) => {
            let mut tables = Vec::<serde_json::Value>::new();
            let mut current_table = Vec::<Vec<String>>::new();
            for line in text.lines() {
                let cells: Vec<String> = line.split("  ").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                if cells.len() >= 2 {
                    current_table.push(cells);
                } else if !current_table.is_empty() {
                    if current_table.len() >= 2 {
                        let headers = current_table[0].clone();
                        let rows: Vec<Vec<String>> = current_table[1..].to_vec();
                        tables.push(serde_json::json!({"headers": headers, "rows": rows}));
                    }
                    current_table.clear();
                }
            }
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

pub fn extract_images(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
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
                                                images.push(serde_json::json!({"page": page_num, "name": String::from_utf8_lossy(name), "width": w, "height": h}));
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

pub fn extract_bookmarks(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            let mut bookmarks = Vec::<serde_json::Value>::new();
            if let Ok(catalog) = doc.catalog() {
                if let Ok(outlines_ref) = catalog.get(b"Outlines") {
                    if let Ok(outlines_id) = outlines_ref.as_reference() {
                        if let Ok(outlines) = doc.get_dictionary(outlines_id) {
                            if let Ok(first_ref) = outlines.get(b"First") {
                                if let Ok(first_id) = first_ref.as_reference() {
                                    collect_bookmarks(&doc, first_id, &mut bookmarks);
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

fn collect_bookmarks(doc: &lopdf::Document, id: lopdf::ObjectId, out: &mut Vec<serde_json::Value>) {
    if let Ok(dict) = doc.get_dictionary(id) {
        let title = dict.get(b"Title").and_then(|t| t.as_str())
            .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
        out.push(serde_json::json!({"title": title}));
        if let Ok(next_ref) = dict.get(b"Next") {
            if let Ok(next_id) = next_ref.as_reference() {
                collect_bookmarks(doc, next_id, out);
            }
        }
    }
}

pub fn extract_annotations(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
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
                                    annotations.push(serde_json::json!({"page": page_num, "type": subtype, "contents": contents}));
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

pub fn extract_key_values(pdf_path: &str) -> String {
    match pdf_extract::extract_text(pdf_path) {
        Ok(text) => {
            let mut pairs = Vec::<serde_json::Value>::new();
            let re = regex::Regex::new(r"([A-Za-z][A-Za-z\s]{1,30}):\s*(.+)").unwrap();
            for cap in re.captures_iter(&text) {
                pairs.push(serde_json::json!({"key": cap[1].trim(), "value": cap[2].trim()}));
            }
            serde_json::json!({"pairs": pairs, "count": pairs.len()}).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
