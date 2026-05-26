use std::collections::BTreeMap;

pub fn merge_pdfs(pdf_paths: &[String], output: &str) -> String {
    if pdf_paths.is_empty() {
        return serde_json::json!({"error": "No PDF paths provided"}).to_string();
    }
    let mut max_id = 1u32;
    let mut documents_pages: BTreeMap<lopdf::ObjectId, lopdf::Object> = BTreeMap::new();
    let mut documents_objects: BTreeMap<lopdf::ObjectId, lopdf::Object> = BTreeMap::new();

    for path in pdf_paths {
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
            b"Catalog" => { catalog_object = Some((catalog_object.map(|(id,_)| id).unwrap_or(object_id), object)); }
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

    for (object_id, object) in &documents_pages {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_id);
            document.objects.insert(*object_id, lopdf::Object::Dictionary(dictionary));
        }
    }

    if let Ok(dictionary) = pages_obj.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", documents_pages.len() as u32);
        dictionary.set("Kids", documents_pages.keys().map(|&id| lopdf::Object::Reference(id)).collect::<Vec<_>>());
        document.objects.insert(pages_id, lopdf::Object::Dictionary(dictionary));
    }

    if let Ok(dictionary) = catalog_obj.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_id);
        document.objects.insert(catalog_id, lopdf::Object::Dictionary(dictionary));
    }

    document.trailer.set("Root", catalog_id);
    document.max_id = document.objects.len() as u32;
    document.renumber_objects();

    match document.save(output) {
        Ok(_) => serde_json::json!({"output": output, "pages": documents_pages.len()}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn split_pdf(pdf_path: &str, pages: &str, output: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(doc) => {
            let total = doc.get_pages().len() as u32;
            let page_nums: Vec<u32> = pages.split(|c: char| c == ',' || c == '-')
                .filter_map(|s| s.trim().parse().ok()).collect();
            let mut new_doc = doc.clone();
            let to_delete: Vec<u32> = (1..=total).filter(|p| !page_nums.contains(p)).collect();
            for &p in to_delete.iter().rev() { new_doc.delete_pages(&[p]); }
            match new_doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "pages_kept": page_nums.len()}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn rotate_pages(pdf_path: &str, output: &str, degrees: u32) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
            for page_id in page_ids {
                if let Ok(page) = doc.get_dictionary_mut(page_id) {
                    page.set(b"Rotate".to_vec(), lopdf::Object::Integer(degrees as i64));
                }
            }
            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "degrees": degrees}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn compress_pdf(pdf_path: &str, output: &str) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let original_size = std::fs::metadata(pdf_path).map(|m| m.len()).unwrap_or(0);
            doc.compress();
            match doc.save(output) {
                Ok(_) => {
                    let new_size = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);
                    let reduction = if original_size > 0 { ((original_size - new_size) as f64 / original_size as f64 * 100.0) as i32 } else { 0 };
                    serde_json::json!({"output": output, "original_bytes": original_size, "compressed_bytes": new_size, "reduction_pct": reduction}).to_string()
                }
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn delete_pages(pdf_path: &str, output: &str, pages: &[u32]) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            for &p in pages.iter().rev() { doc.delete_pages(&[p]); }
            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "deleted": pages}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn reorder_pages(pdf_path: &str, output: &str, order: &[u32]) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let pages = doc.get_pages();
            let page_ids: Vec<lopdf::ObjectId> = pages.values().copied().collect();
            let mut new_ids = Vec::new();
            for &idx in order {
                if idx as usize <= page_ids.len() && idx > 0 { new_ids.push(page_ids[(idx - 1) as usize]); }
            }
            if let Ok(catalog) = doc.catalog() {
                if let Ok(pages_ref) = catalog.get(b"Pages").and_then(|p| p.as_reference()) {
                    if let Ok(pages_dict) = doc.get_dictionary_mut(pages_ref) {
                        pages_dict.set(b"Kids".to_vec(), lopdf::Object::Array(new_ids.iter().map(|&id| lopdf::Object::Reference(id)).collect()));
                        pages_dict.set(b"Count".to_vec(), lopdf::Object::Integer(new_ids.len() as i64));
                    }
                }
            }
            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "order": order}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn crop_pages(pdf_path: &str, output: &str, crop_box: &[f32; 4]) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
            for page_id in page_ids {
                if let Ok(page) = doc.get_dictionary_mut(page_id) {
                    let cb = lopdf::Object::Array(vec![
                        lopdf::Object::Real(crop_box[0]), lopdf::Object::Real(crop_box[1]),
                        lopdf::Object::Real(crop_box[2]), lopdf::Object::Real(crop_box[3]),
                    ]);
                    page.set(b"CropBox".to_vec(), cb);
                }
            }
            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "crop_box": crop_box}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn add_watermark(pdf_path: &str, output: &str, text: Option<&str>) -> String {
    match lopdf::Document::load(pdf_path) {
        Ok(mut doc) => {
            let text = text.unwrap_or("CONFIDENTIAL");
            let mut font_dict = lopdf::Dictionary::new();
            font_dict.set(b"Type".to_vec(), lopdf::Object::Name(b"Font".to_vec()));
            font_dict.set(b"Subtype".to_vec(), lopdf::Object::Name(b"Type1".to_vec()));
            font_dict.set(b"BaseFont".to_vec(), lopdf::Object::Name(b"Helvetica".to_vec()));
            let font_id = doc.add_object(lopdf::Object::Dictionary(font_dict));

            let wm_content = format!(
                "0.85 0.85 0.85 rg BT /Hwm 60 Tf 0.7071 0.7071 -0.7071 0.7071 130 250 Tm ({}) Tj ET", text
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
                let res_id = if let Ok(page) = doc.get_dictionary(page_id) {
                    match page.get(b"Resources") { Ok(lopdf::Object::Reference(id)) => Some(*id), _ => None }
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

                let invoke_stream = lopdf::Stream::new(lopdf::Dictionary::new(), b"q /Watermark Do Q".to_vec());
                let invoke_id = doc.add_object(invoke_stream);
                if let Ok(page) = doc.get_dictionary_mut(page_id) {
                    let existing = page.get(b"Contents").ok().cloned();
                    let new_contents = match existing {
                        Some(lopdf::Object::Reference(id)) => lopdf::Object::Array(vec![lopdf::Object::Reference(id), lopdf::Object::Reference(invoke_id)]),
                        Some(lopdf::Object::Array(mut arr)) => { arr.push(lopdf::Object::Reference(invoke_id)); lopdf::Object::Array(arr) }
                        _ => lopdf::Object::Reference(invoke_id),
                    };
                    page.set(b"Contents".to_vec(), new_contents);
                }
            }
            match doc.save(output) {
                Ok(_) => serde_json::json!({"output": output, "watermark": text}).to_string(),
                Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}
