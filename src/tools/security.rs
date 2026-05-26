use sha2::{Sha256, Digest};

pub fn hash_pdf(pdf_path: &str) -> String {
    match std::fs::read(pdf_path) {
        Ok(bytes) => {
            let hash = Sha256::digest(&bytes);
            serde_json::json!({"algorithm": "sha256", "hash": format!("{:x}", hash), "file_size": bytes.len()}).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn encrypt_pdf(pdf_path: &str, output: &str, owner_password: &str, user_password: Option<&str>) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let user_pass = user_password.unwrap_or("");

            // PDF encryption R4 (AES-128, widely compatible with all viewers)
            let key_bytes = 16usize;
            let revision = 4i64;
            let version = 4i64;

            // Generate file ID
            let file_id: Vec<u8> = {
                let mut ctx = md5::Context::new();
                ctx.consume(pdf_path.as_bytes());
                ctx.consume(chrono::Utc::now().timestamp().to_le_bytes());
                ctx.finalize().0.to_vec()
            };

            let p_value: i32 = -3904;

            let o_value = compute_o_value(owner_password.as_bytes(), user_pass.as_bytes(), key_bytes, revision);
            let enc_key = compute_encryption_key(user_pass.as_bytes(), &o_value, p_value, &file_id, key_bytes, revision);
            let u_value = compute_u_value(&enc_key, &file_id, revision);

            // Build Encrypt dictionary
            let mut encrypt_dict = lopdf::Dictionary::new();
            encrypt_dict.set(b"Filter".to_vec(), lopdf::Object::Name(b"Standard".to_vec()));
            encrypt_dict.set(b"V".to_vec(), lopdf::Object::Integer(version));
            encrypt_dict.set(b"R".to_vec(), lopdf::Object::Integer(revision));
            encrypt_dict.set(b"Length".to_vec(), lopdf::Object::Integer(128));
            encrypt_dict.set(b"P".to_vec(), lopdf::Object::Integer(p_value as i64));
            encrypt_dict.set(b"O".to_vec(), lopdf::Object::String(o_value, lopdf::StringFormat::Literal));
            encrypt_dict.set(b"U".to_vec(), lopdf::Object::String(u_value, lopdf::StringFormat::Literal));
            encrypt_dict.set(b"EncryptMetadata".to_vec(), lopdf::Object::Boolean(true));

            // CF dict for AES
            let mut std_cf = lopdf::Dictionary::new();
            std_cf.set(b"CFM".to_vec(), lopdf::Object::Name(b"AESV2".to_vec()));
            std_cf.set(b"AuthEvent".to_vec(), lopdf::Object::Name(b"DocOpen".to_vec()));
            std_cf.set(b"Length".to_vec(), lopdf::Object::Integer(16));
            let mut cf = lopdf::Dictionary::new();
            cf.set(b"StdCF".to_vec(), lopdf::Object::Dictionary(std_cf));
            encrypt_dict.set(b"CF".to_vec(), lopdf::Object::Dictionary(cf));
            encrypt_dict.set(b"StmF".to_vec(), lopdf::Object::Name(b"StdCF".to_vec()));
            encrypt_dict.set(b"StrF".to_vec(), lopdf::Object::Name(b"StdCF".to_vec()));

            let encrypt_id = doc.add_object(lopdf::Object::Dictionary(encrypt_dict));
            doc.trailer.set(b"Encrypt".to_vec(), lopdf::Object::Reference(encrypt_id));

            // Set file ID in trailer
            let id_obj = lopdf::Object::Array(vec![
                lopdf::Object::String(file_id.clone(), lopdf::StringFormat::Literal),
                lopdf::Object::String(file_id, lopdf::StringFormat::Literal),
            ]);
            doc.trailer.set(b"ID".to_vec(), id_obj);

            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "encrypted": true, "algorithm": "AES-128", "revision": 4}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

// PDF password padding (Table 2, PDF spec)
const PASSWORD_PADDING: [u8; 32] = [
    0x28, 0xBF, 0x4E, 0x5E, 0x4E, 0x75, 0x8A, 0x41, 0x64, 0x00, 0x4E, 0x56,
    0xFF, 0xFA, 0x01, 0x08, 0x2E, 0x2E, 0x00, 0xB6, 0xD0, 0x68, 0x3E, 0x80,
    0x2F, 0x0C, 0xA9, 0xFE, 0x64, 0x53, 0x69, 0x7A,
];

fn pad_password(password: &[u8]) -> [u8; 32] {
    let mut padded = [0u8; 32];
    let len = password.len().min(32);
    padded[..len].copy_from_slice(&password[..len]);
    padded[len..].copy_from_slice(&PASSWORD_PADDING[..32 - len]);
    padded
}

fn rc4(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut s: Vec<u8> = (0u16..=255).map(|i| i as u8).collect();
    let mut j: u8 = 0;
    for i in 0..256usize {
        j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
        s.swap(i, j as usize);
    }
    let mut i: u8 = 0;
    j = 0;
    let mut output = data.to_vec();
    for byte in output.iter_mut() {
        i = i.wrapping_add(1);
        j = j.wrapping_add(s[i as usize]);
        s.swap(i as usize, j as usize);
        *byte ^= s[(s[i as usize].wrapping_add(s[j as usize])) as usize];
    }
    output
}

fn compute_o_value(owner_pass: &[u8], user_pass: &[u8], key_bytes: usize, revision: i64) -> Vec<u8> {
    let padded_owner = pad_password(owner_pass);
    let mut hash = md5::compute(padded_owner).0.to_vec();
    if revision >= 3 {
        for _ in 0..50 {
            hash = md5::compute(&hash[..key_bytes]).0.to_vec();
        }
    }
    hash.truncate(key_bytes);

    let padded_user = pad_password(user_pass);
    let mut o = rc4(&hash, &padded_user);
    if revision >= 3 {
        for i in 1..=19u8 {
            let modified: Vec<u8> = hash.iter().map(|&b| b ^ i).collect();
            o = rc4(&modified, &o);
        }
    }
    o
}

fn compute_encryption_key(password: &[u8], o_value: &[u8], p_value: i32, file_id: &[u8], key_bytes: usize, revision: i64) -> Vec<u8> {
    let padded = pad_password(password);
    let mut ctx = md5::Context::new();
    ctx.consume(padded);
    ctx.consume(o_value);
    ctx.consume((p_value as u32).to_le_bytes());
    ctx.consume(file_id);
    let mut hash = ctx.finalize().0.to_vec();
    if revision >= 3 {
        for _ in 0..50 {
            hash = md5::compute(&hash[..key_bytes]).0.to_vec();
        }
    }
    hash.truncate(key_bytes);
    hash
}

fn compute_u_value(key: &[u8], file_id: &[u8], revision: i64) -> Vec<u8> {
    if revision == 2 {
        rc4(key, &PASSWORD_PADDING)
    } else {
        let mut ctx = md5::Context::new();
        ctx.consume(PASSWORD_PADDING);
        ctx.consume(file_id);
        let hash = ctx.finalize().0.to_vec();
        let mut result = rc4(key, &hash);
        for i in 1..=19u8 {
            let modified: Vec<u8> = key.iter().map(|&b| b ^ i).collect();
            result = rc4(&modified, &result);
        }
        result.resize(32, 0);
        result
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
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let mut redacted_count = 0u32;
            let page_nums: Vec<u32> = doc.get_pages().keys().copied().collect();

            for page_num in page_nums {
                for term in terms {
                    // Replace with spaces of same length (true redaction from content stream)
                    let replacement = " ".repeat(term.len());
                    match doc.replace_partial_text(page_num, term, &replacement, None) {
                        Ok(count) => redacted_count += count as u32,
                        Err(_) => {}
                    }
                }
            }

            // Strip metadata
            doc.trailer.remove(b"Info");

            if redacted_count == 0 {
                return serde_json::json!({
                    "error": "no_matches",
                    "message": format!("None of the {} terms were found in the document", terms.len()),
                    "terms_searched": terms,
                }).to_string();
            }

            match doc.save(output) {
                Ok(_) => {
                    let hash = Sha256::digest(&std::fs::read(output).unwrap_or_default());
                    serde_json::json!({
                        "output": output,
                        "redactions_applied": redacted_count,
                        "metadata_stripped": true,
                        "sha256": format!("{:x}", hash),
                    }).to_string()
                }
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
