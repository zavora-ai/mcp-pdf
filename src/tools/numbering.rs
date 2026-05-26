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
