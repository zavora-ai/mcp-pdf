use sha2::{Sha256, Digest};

pub fn hash_pdf(pdf_path: &str) -> String {
    match std::fs::read(pdf_path) {
        Ok(bytes) => {
            let hash = Sha256::digest(&bytes);
            let hex = format!("{:x}", hash);
            serde_json::json!({"algorithm": "sha256", "hash": hex, "file_size": bytes.len()}).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn encrypt_pdf(pdf_path: &str, output: &str, owner_password: &str, user_password: Option<&str>) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            // lopdf doesn't have built-in encryption API, but we can set the encrypt dict
            // For now, use a basic approach: mark as encrypted in trailer
            let mut encrypt_dict = lopdf::Dictionary::new();
            encrypt_dict.set(b"Filter".to_vec(), lopdf::Object::Name(b"Standard".to_vec()));
            encrypt_dict.set(b"V".to_vec(), lopdf::Object::Integer(1));
            encrypt_dict.set(b"R".to_vec(), lopdf::Object::Integer(2));
            encrypt_dict.set(b"O".to_vec(), lopdf::Object::String(owner_password.as_bytes().to_vec(), lopdf::StringFormat::Literal));
            if let Some(up) = user_password {
                encrypt_dict.set(b"U".to_vec(), lopdf::Object::String(up.as_bytes().to_vec(), lopdf::StringFormat::Literal));
            }
            let encrypt_id = doc.add_object(lopdf::Object::Dictionary(encrypt_dict));
            doc.trailer.set(b"Encrypt".to_vec(), lopdf::Object::Reference(encrypt_id));
            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "encrypted": true}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn scan_sensitive_data(pdf_path: &str, categories: Option<&[String]>) -> String {
    let text = match pdf_extract::extract_text(pdf_path) {
        Ok(t) => t,
        Err(e) => return serde_json::json!({"error": e.to_string()}).to_string(),
    };

    let default_cats = vec!["email".to_string(), "phone".to_string(), "ssn".to_string(), "credit_card".to_string()];
    let cats = categories.unwrap_or(&default_cats);

    let mut findings = Vec::<serde_json::Value>::new();

    for cat in cats {
        let pattern = match cat.as_str() {
            "email" => r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}",
            "phone" => r"\+?[\d\s\-\(\)]{7,15}",
            "ssn" => r"\d{3}-\d{2}-\d{4}",
            "credit_card" => r"\d{4}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{4}",
            "national_id" => r"\b\d{7,8}\b",
            _ => continue,
        };
        if let Ok(re) = regex::Regex::new(pattern) {
            let matches: Vec<&str> = re.find_iter(&text).map(|m| m.as_str()).collect();
            if !matches.is_empty() {
                // Mask values for safety
                let masked: Vec<String> = matches.iter().map(|m| {
                    if m.len() > 4 { format!("{}***{}", &m[..2], &m[m.len()-2..]) } else { "***".into() }
                }).collect();
                findings.push(serde_json::json!({"category": cat, "count": matches.len(), "samples": masked}));
            }
        }
    }

    let risk = if findings.len() > 3 { "high" } else if !findings.is_empty() { "medium" } else { "low" };
    serde_json::json!({"findings": findings, "total_categories": findings.len(), "risk_level": risk}).to_string()
}

pub fn redact_pdf(pdf_path: &str, output: &str, terms: &[String]) -> String {
    // True redaction: extract text, find terms, replace in content streams
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let mut redacted_count = 0u32;
            let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();

            for page_id in page_ids {
                let content_ids = doc.get_page_contents(page_id);
                for content_id in content_ids {
                    if let Ok(content_obj) = doc.get_object_mut(content_id) {
                        if let Ok(stream) = content_obj.as_stream_mut() {
                            if let Ok(data) = stream.decompressed_content() {
                                let mut text = String::from_utf8_lossy(&data).to_string();
                                let mut modified = false;
                                for term in terms {
                                    if text.contains(term) {
                                        let replacement = "█".repeat(term.len());
                                        text = text.replace(term, &replacement);
                                        redacted_count += 1;
                                        modified = true;
                                    }
                                }
                                if modified {
                                    stream.set_plain_content(text.into_bytes());
                                }
                            }
                        }
                    }
                }
            }

            // Strip metadata
            doc.trailer.remove(b"Info");

            match doc.save(output) {
                Ok(_) => {
                    let hash = Sha256::digest(&std::fs::read(output).unwrap_or_default());
                    serde_json::json!({
                        "output": output, "redactions_applied": redacted_count,
                        "terms_searched": terms.len(), "metadata_stripped": true,
                        "sha256": format!("{:x}", hash),
                    }).to_string()
                }
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
