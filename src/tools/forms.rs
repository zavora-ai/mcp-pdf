pub fn detect_form_fields(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            let mut fields = Vec::<serde_json::Value>::new();
            if let Ok(catalog) = doc.catalog() {
                if let Ok(acroform) = catalog.get(b"AcroForm").and_then(|a| a.as_dict()) {
                    if let Ok(field_arr) = acroform.get(b"Fields").and_then(|f| f.as_array()) {
                        for field_ref in field_arr {
                            if let Ok(field_id) = field_ref.as_reference() {
                                if let Ok(field) = doc.get_dictionary(field_id) {
                                    let name = field.get(b"T").and_then(|t| t.as_str())
                                        .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
                                    let field_type = field.get(b"FT").and_then(|t| t.as_name())
                                        .map(|n| match n {
                                            b"Tx" => "text",
                                            b"Btn" => "button",
                                            b"Ch" => "choice",
                                            b"Sig" => "signature",
                                            _ => "unknown",
                                        }).unwrap_or("unknown");
                                    let value = field.get(b"V").and_then(|v| v.as_str())
                                        .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
                                    fields.push(serde_json::json!({
                                        "name": name, "type": field_type, "value": value,
                                    }));
                                }
                            }
                        }
                    }
                }
            }
            if fields.is_empty() {
                serde_json::json!({"fields": [], "count": 0, "message": "No form fields found"}).to_string()
            } else {
                serde_json::json!({"fields": fields, "count": fields.len()}).to_string()
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn fill_form(pdf_path: &str, output: &str, field_values: &serde_json::Value) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let mut filled = 0u32;
            let values = match field_values.as_object() {
                Some(m) => m,
                None => return serde_json::json!({"error": "field_values must be a JSON object"}).to_string(),
            };

            // Find AcroForm fields
            let field_ids: Vec<lopdf::ObjectId> = {
                let mut ids = Vec::new();
                if let Ok(catalog) = doc.catalog() {
                    if let Ok(acroform) = catalog.get(b"AcroForm").and_then(|a| a.as_dict()) {
                        if let Ok(field_arr) = acroform.get(b"Fields").and_then(|f| f.as_array()) {
                            for field_ref in field_arr {
                                if let Ok(id) = field_ref.as_reference() {
                                    ids.push(id);
                                }
                            }
                        }
                    }
                }
                ids
            };

            for field_id in field_ids {
                if let Ok(field) = doc.get_dictionary(field_id) {
                    let name = field.get(b"T").and_then(|t| t.as_str())
                        .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
                    if let Some(new_val) = values.get(&name) {
                        let val_str = new_val.as_str().unwrap_or("").to_string();
                        if let Ok(field_mut) = doc.get_dictionary_mut(field_id) {
                            field_mut.set(b"V".to_vec(), lopdf::Object::String(val_str.into_bytes(), lopdf::StringFormat::Literal));
                            filled += 1;
                        }
                    }
                }
            }

            if filled == 0 {
                return serde_json::json!({"error": "no_fields_matched", "message": "None of the provided field names matched form fields"}).to_string();
            }

            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "fields_filled": filled}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn flatten_form(pdf_path: &str, output: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            // Remove AcroForm from catalog to flatten (makes fields non-editable)
            let catalog_id = match doc.catalog() {
                Ok(_) => {
                    // Find catalog object ID from trailer
                    match doc.trailer.get(b"Root").and_then(|r| r.as_reference()) {
                        Ok(id) => id,
                        Err(_) => return serde_json::json!({"error": "Cannot find catalog"}).to_string(),
                    }
                }
                Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
            };

            if let Ok(catalog) = doc.get_dictionary_mut(catalog_id) {
                catalog.remove(b"AcroForm");
            }

            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "flattened": true}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
