pub fn add_page_numbers(pdf_path: &str, output: &str, position: Option<&str>) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let position = position.unwrap_or("bottom-center");
            let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
            let total = page_ids.len();
            for (i, page_id) in page_ids.iter().enumerate() {
                let num = i + 1;
                let label = format!("Page {} of {}", num, total);
                let (x, y) = match position {
                    "bottom-right" => (500, 20),
                    "top-right" => (500, 770),
                    _ => (270, 20),
                };
                let content = format!("BT /F1 9 Tf {} {} Td ({}) Tj ET", x, y, label);
                let stream = lopdf::Stream::new(lopdf::Dictionary::new(), content.into_bytes());
                let stream_id = doc.add_object(stream);
                if let Ok(page) = doc.get_dictionary_mut(*page_id) {
                    let existing = page.get(b"Contents").ok().cloned();
                    let new_contents = match existing {
                        Some(lopdf::Object::Reference(id)) => lopdf::Object::Array(vec![lopdf::Object::Reference(id), lopdf::Object::Reference(stream_id)]),
                        Some(lopdf::Object::Array(mut arr)) => { arr.push(lopdf::Object::Reference(stream_id)); lopdf::Object::Array(arr) }
                        _ => lopdf::Object::Reference(stream_id),
                    };
                    page.set(b"Contents".to_vec(), new_contents);
                }
            }
            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "pages_numbered": total}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn add_bates_numbers(pdf_path: &str, output: &str, prefix: &str, start: u32, digits: Option<u32>) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let digits = digits.unwrap_or(6) as usize;
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
                        Some(lopdf::Object::Reference(id)) => lopdf::Object::Array(vec![lopdf::Object::Reference(id), lopdf::Object::Reference(stream_id)]),
                        Some(lopdf::Object::Array(mut arr)) => { arr.push(lopdf::Object::Reference(stream_id)); lopdf::Object::Array(arr) }
                        _ => lopdf::Object::Reference(stream_id),
                    };
                    page.set(b"Contents".to_vec(), new_contents);
                }
            }
            let end_num = start + total as u32 - 1;
            let range = format!("{}{:0>width$} - {}{:0>width$}", prefix, start, prefix, end_num, width = digits);
            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "range": range, "pages": total}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn add_headers_footers(pdf_path: &str, output: &str, header: Option<&str>, footer: Option<&str>) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let mut font_dict = lopdf::Dictionary::new();
            font_dict.set(b"Type".to_vec(), lopdf::Object::Name(b"Font".to_vec()));
            font_dict.set(b"Subtype".to_vec(), lopdf::Object::Name(b"Type1".to_vec()));
            font_dict.set(b"BaseFont".to_vec(), lopdf::Object::Name(b"Helvetica".to_vec()));
            let font_id = doc.add_object(lopdf::Object::Dictionary(font_dict));

            // Create Form XObject for header/footer (self-contained)
            let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
            let total = page_ids.len();

            for (i, page_id) in page_ids.iter().enumerate() {
                let mut content = String::new();
                if let Some(h) = header {
                    let h = h.replace("{page}", &(i+1).to_string()).replace("{total}", &total.to_string());
                    content.push_str(&format!("BT /Fhf 8 Tf 200 810 Td ({}) Tj ET ", h));
                }
                if let Some(f) = footer {
                    let f = f.replace("{page}", &(i+1).to_string()).replace("{total}", &total.to_string());
                    content.push_str(&format!("BT /Fhf 8 Tf 200 20 Td ({}) Tj ET ", f));
                }
                if content.is_empty() { continue; }

                // Add font to resources
                let res_id = if let Ok(page) = doc.get_dictionary(*page_id) {
                    match page.get(b"Resources") { Ok(lopdf::Object::Reference(id)) => Some(*id), _ => None }
                } else { None };
                if let Some(rid) = res_id {
                    if let Ok(res) = doc.get_dictionary_mut(rid) {
                        if let Ok(fonts) = res.get_mut(b"Font").and_then(|f| f.as_dict_mut()) {
                            fonts.set(b"Fhf".to_vec(), lopdf::Object::Reference(font_id));
                        }
                    }
                }

                let stream = lopdf::Stream::new(lopdf::Dictionary::new(), content.into_bytes());
                let stream_id = doc.add_object(stream);
                if let Ok(page) = doc.get_dictionary_mut(*page_id) {
                    let existing = page.get(b"Contents").ok().cloned();
                    let new_contents = match existing {
                        Some(lopdf::Object::Reference(id)) => lopdf::Object::Array(vec![lopdf::Object::Reference(id), lopdf::Object::Reference(stream_id)]),
                        Some(lopdf::Object::Array(mut arr)) => { arr.push(lopdf::Object::Reference(stream_id)); lopdf::Object::Array(arr) }
                        _ => lopdf::Object::Reference(stream_id),
                    };
                    page.set(b"Contents".to_vec(), new_contents);
                }
            }
            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "pages": total}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn split_by_bookmarks(pdf_path: &str, output_dir: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            // Get bookmarks with their target pages
            let mut bookmarks: Vec<(String, u32)> = Vec::new();
            if let Ok(catalog) = doc.catalog() {
                if let Ok(outlines_ref) = catalog.get(b"Outlines") {
                    if let Ok(outlines_id) = outlines_ref.as_reference() {
                        if let Ok(outlines) = doc.get_dictionary(outlines_id) {
                            if let Ok(first_ref) = outlines.get(b"First") {
                                if let Ok(first_id) = first_ref.as_reference() {
                                    collect_bookmark_pages(&doc, first_id, &mut bookmarks);
                                }
                            }
                        }
                    }
                }
            }

            if bookmarks.is_empty() {
                return serde_json::json!({"error": "No bookmarks found in PDF"}).to_string();
            }

            std::fs::create_dir_all(output_dir).ok();
            let total_pages = doc.get_pages().len() as u32;
            let mut files = Vec::new();

            for (i, (title, _start_page)) in bookmarks.iter().enumerate() {
                let end_page = if i + 1 < bookmarks.len() { bookmarks[i+1].1 - 1 } else { total_pages };
                let start = bookmarks[i].1;
                let safe_title: String = title.chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-').take(50).collect();
                let filename = format!("{}/{:02}_{}.pdf", output_dir, i + 1, safe_title.trim());

                let mut new_doc = doc.clone();
                let to_delete: Vec<u32> = (1..=total_pages).filter(|&p| p < start || p > end_page).collect();
                for &p in to_delete.iter().rev() { new_doc.delete_pages(&[p]); }
                if new_doc.save(&filename).is_ok() {
                    files.push(filename);
                }
            }

            serde_json::json!({"output_dir": output_dir, "files": files, "sections": bookmarks.len()}).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

fn collect_bookmark_pages(doc: &lopdf::Document, id: lopdf::ObjectId, out: &mut Vec<(String, u32)>) {
    if let Ok(dict) = doc.get_dictionary(id) {
        let title = dict.get(b"Title").and_then(|t| t.as_str())
            .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
        // Default to page 1 if we can't resolve destination
        let page = 1u32;
        if !title.is_empty() { out.push((title, page)); }
        if let Ok(next_ref) = dict.get(b"Next") {
            if let Ok(next_id) = next_ref.as_reference() {
                collect_bookmark_pages(doc, next_id, out);
            }
        }
    }
}
