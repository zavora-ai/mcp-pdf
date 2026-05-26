pub fn detect_form_fields(pdf_path: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            let mut fields = Vec::<serde_json::Value>::new();

            // Method 1: Check AcroForm /Fields array
            if let Ok(catalog) = doc.catalog() {
                if let Ok(acroform_ref) = catalog.get(b"AcroForm") {
                    let acroform = match acroform_ref {
                        lopdf::Object::Reference(id) => doc.get_dictionary(*id).ok(),
                        lopdf::Object::Dictionary(d) => Some(d),
                        _ => None,
                    };
                    if let Some(af) = acroform {
                        if let Ok(field_arr) = af.get(b"Fields").and_then(|f| f.as_array()) {
                            for field_ref in field_arr {
                                if let Ok(field_id) = field_ref.as_reference() {
                                    collect_field(&doc, field_id, &mut fields);
                                }
                            }
                        }
                    }
                }
            }

            // Method 2: Scan page annotations for Widget subtypes (common in many PDFs)
            if fields.is_empty() {
                for (page_num, &page_id) in &doc.get_pages() {
                    if let Ok(page) = doc.get_dictionary(page_id) {
                        if let Ok(annots) = page.get(b"Annots").and_then(|a| a.as_array()) {
                            for obj in annots {
                                if let Ok(id) = obj.as_reference() {
                                    if let Ok(annot) = doc.get_dictionary(id) {
                                        let subtype = annot.get(b"Subtype").and_then(|s| s.as_name()).unwrap_or(b"");
                                        if subtype == b"Widget" {
                                            let name = annot.get(b"T").and_then(|t| t.as_str())
                                                .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
                                            let ft = annot.get(b"FT").and_then(|t| t.as_name())
                                                .map(|n| match n { b"Tx" => "text", b"Btn" => "button", b"Ch" => "choice", b"Sig" => "signature", _ => "unknown" })
                                                .unwrap_or("text");
                                            let value = annot.get(b"V").and_then(|v| v.as_str())
                                                .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
                                            if !name.is_empty() {
                                                fields.push(serde_json::json!({"name": name, "type": ft, "value": value, "page": page_num}));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if fields.is_empty() {
                serde_json::json!({"fields": [], "count": 0, "message": "No form fields found. This may be a flat/scanned form — use extract_text to read it."}).to_string()
            } else {
                serde_json::json!({"fields": fields, "count": fields.len()}).to_string()
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

fn collect_field(doc: &lopdf::Document, field_id: lopdf::ObjectId, fields: &mut Vec<serde_json::Value>) {
    if let Ok(field) = doc.get_dictionary(field_id) {
        let name = field.get(b"T").and_then(|t| t.as_str())
            .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();
        let field_type = field.get(b"FT").and_then(|t| t.as_name())
            .map(|n| match n { b"Tx" => "text", b"Btn" => "button", b"Ch" => "choice", b"Sig" => "signature", _ => "unknown" })
            .unwrap_or("unknown");
        let value = field.get(b"V").and_then(|v| v.as_str())
            .map(|s| String::from_utf8_lossy(s).to_string()).unwrap_or_default();

        if !name.is_empty() {
            fields.push(serde_json::json!({"name": name, "type": field_type, "value": value}));
        }

        // Recurse into Kids
        if let Ok(kids) = field.get(b"Kids").and_then(|k| k.as_array()) {
            for kid in kids {
                if let Ok(kid_id) = kid.as_reference() {
                    collect_field(doc, kid_id, fields);
                }
            }
        }
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

            // Collect all field IDs (from AcroForm and page annotations)
            let mut field_ids: Vec<lopdf::ObjectId> = Vec::new();

            // Method 1: AcroForm /Fields
            if let Ok(catalog) = doc.catalog() {
                if let Ok(acroform_ref) = catalog.get(b"AcroForm") {
                    let acroform = match acroform_ref {
                        lopdf::Object::Reference(id) => doc.get_dictionary(*id).ok(),
                        lopdf::Object::Dictionary(d) => Some(d),
                        _ => None,
                    };
                    if let Some(af) = acroform {
                        if let Ok(arr) = af.get(b"Fields").and_then(|f| f.as_array()) {
                            for r in arr { if let Ok(id) = r.as_reference() { field_ids.push(id); } }
                        }
                    }
                }
            }

            // Method 2: Page Widget annotations
            if field_ids.is_empty() {
                for (_, &page_id) in &doc.get_pages() {
                    if let Ok(page) = doc.get_dictionary(page_id) {
                        if let Ok(annots) = page.get(b"Annots").and_then(|a| a.as_array()) {
                            for obj in annots {
                                if let Ok(id) = obj.as_reference() {
                                    if let Ok(annot) = doc.get_dictionary(id) {
                                        if annot.get(b"Subtype").and_then(|s| s.as_name()).ok() == Some(b"Widget") {
                                            field_ids.push(id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Fill matching fields
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

/// Fill a flat (non-interactive) form by overlaying text at specified positions
pub fn fill_flat_form(pdf_path: &str, output: &str, entries: &[FlatFormEntry]) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            // Pre-create a Helvetica font object
            let mut fd = lopdf::Dictionary::new();
            fd.set(b"Type".to_vec(), lopdf::Object::Name(b"Font".to_vec()));
            fd.set(b"Subtype".to_vec(), lopdf::Object::Name(b"Type1".to_vec()));
            fd.set(b"BaseFont".to_vec(), lopdf::Object::Name(b"Helvetica".to_vec()));
            let font_id = doc.add_object(lopdf::Object::Dictionary(fd));

            // Add font to all pages' resources
            let page_ids: Vec<(u32, lopdf::ObjectId)> = doc.get_pages().iter().map(|(&n, &id)| (n, id)).collect();
            for &(_, page_id) in &page_ids {
                let res_id = if let Ok(page) = doc.get_dictionary(page_id) {
                    match page.get(b"Resources") {
                        Ok(lopdf::Object::Reference(id)) => Some(*id),
                        _ => None,
                    }
                } else { None };

                if let Some(rid) = res_id {
                    if let Ok(res) = doc.get_dictionary_mut(rid) {
                        if let Ok(fonts) = res.get_mut(b"Font").and_then(|f| f.as_dict_mut()) {
                            fonts.set(b"Hff".to_vec(), lopdf::Object::Reference(font_id));
                        } else {
                            let mut fonts = lopdf::Dictionary::new();
                            fonts.set(b"Hff".to_vec(), lopdf::Object::Reference(font_id));
                            res.set(b"Font".to_vec(), lopdf::Object::Dictionary(fonts));
                        }
                    }
                }
            }

            // Now overlay text entries
            // User provides x,y in mm from TOP-LEFT (natural for forms)
            // PDF uses points from BOTTOM-LEFT
            // A4: 297mm tall = 842pt
            let page_height_mm = 297.0f32;
            let mut filled = 0u32;
            for entry in entries {
                let pages = doc.get_pages();
                if let Some(&page_id) = pages.get(&entry.page) {
                    let fs = entry.font_size.unwrap_or(10.0);
                    // Convert: x stays same (mm→pt), y flips (from top to from bottom)
                    let x_pt = entry.x * 2.8346;
                    let y_pt = (page_height_mm - entry.y) * 2.8346;
                    let content = format!(
                        "BT /Hff {} Tf {} {} Td ({}) Tj ET",
                        fs, x_pt, y_pt,
                        entry.text.replace('(', "\\(").replace(')', "\\)")
                    );
                    let stream = lopdf::Stream::new(lopdf::Dictionary::new(), content.into_bytes());
                    let stream_id = doc.add_object(stream);

                    if let Ok(page) = doc.get_dictionary_mut(page_id) {
                        let existing = page.get(b"Contents").ok().cloned();
                        let new_contents = match existing {
                            Some(lopdf::Object::Reference(id)) => lopdf::Object::Array(vec![lopdf::Object::Reference(id), lopdf::Object::Reference(stream_id)]),
                            Some(lopdf::Object::Array(mut arr)) => { arr.push(lopdf::Object::Reference(stream_id)); lopdf::Object::Array(arr) }
                            _ => lopdf::Object::Reference(stream_id),
                        };
                        page.set(b"Contents".to_vec(), new_contents);
                    }
                    filled += 1;
                }
            }

            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "entries_filled": filled}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub struct FlatFormEntry {
    pub page: u32,
    pub x: f32,      // mm from left
    pub y: f32,      // mm from bottom
    pub text: String,
    pub font_size: Option<f32>,
}
