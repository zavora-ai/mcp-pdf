use printpdf::*;

pub fn pdf_to_markdown(pdf_path: &str, output: Option<&str>) -> String {
    match pdf_oxide::PdfDocument::open(pdf_path) {
        Ok(doc) => {
            let page_count = doc.page_count().unwrap_or(0);
            let opts = pdf_oxide::converters::ConversionOptions::default();
            let mut md = String::new();
            for i in 0..page_count {
                match doc.to_markdown(i, &opts) {
                    Ok(page_md) => { md.push_str(&page_md); md.push_str("\n\n---\n\n"); }
                    Err(e) => md.push_str(&format!("[Page {} error: {}]\n", i + 1, e)),
                }
            }
            if let Some(out) = output {
                std::fs::write(out, &md).ok();
                serde_json::json!({"output": out, "pages": page_count}).to_string()
            } else {
                md
            }
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn pdf_to_html(pdf_path: &str, output: &str) -> String {
    match pdf_oxide::PdfDocument::open(pdf_path) {
        Ok(doc) => {
            let page_count = doc.page_count().unwrap_or(0);
            let opts = pdf_oxide::converters::ConversionOptions::default();
            let mut html = String::from("<!DOCTYPE html><html><head><meta charset=\"utf-8\"><style>body{font-family:sans-serif;max-width:800px;margin:0 auto;padding:20px;line-height:1.6}h1,h2,h3{margin-top:1.5em}table{border-collapse:collapse;width:100%}td,th{border:1px solid #ddd;padding:8px}</style></head><body>\n");
            for i in 0..page_count {
                if let Ok(md) = doc.to_markdown(i, &opts) {
                    for line in md.lines() {
                        let t = line.trim();
                        if t.starts_with("### ") { html.push_str(&format!("<h3>{}</h3>\n", &t[4..])); }
                        else if t.starts_with("## ") { html.push_str(&format!("<h2>{}</h2>\n", &t[3..])); }
                        else if t.starts_with("# ") { html.push_str(&format!("<h1>{}</h1>\n", &t[2..])); }
                        else if t.starts_with("- ") { html.push_str(&format!("<li>{}</li>\n", &t[2..])); }
                        else if !t.is_empty() { html.push_str(&format!("<p>{}</p>\n", t)); }
                    }
                }
                if i < page_count - 1 { html.push_str("<hr>\n"); }
            }
            html.push_str("</body></html>");
            std::fs::write(output, &html).ok();
            serde_json::json!({"output": output, "pages": page_count}).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn pdf_to_json(pdf_path: &str, output: &str) -> String {
    match pdf_oxide::PdfDocument::open(pdf_path) {
        Ok(doc) => {
            let page_count = doc.page_count().unwrap_or(0);
            let opts = pdf_oxide::converters::ConversionOptions::default();
            let mut pages = Vec::new();
            for i in 0..page_count {
                let text = doc.extract_text(i).unwrap_or_default();
                let md = doc.to_markdown(i, &opts).unwrap_or_default();
                pages.push(serde_json::json!({"page": i + 1, "text": text, "markdown": md}));
            }
            let result = serde_json::json!({"file": pdf_path, "page_count": page_count, "pages": pages});
            std::fs::write(output, result.to_string()).ok();
            serde_json::json!({"output": output, "pages": page_count}).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn pdf_to_csv(pdf_path: &str, output: &str) -> String {
    match pdf_oxide::PdfDocument::open(pdf_path) {
        Ok(doc) => {
            let page_count = doc.page_count().unwrap_or(0);
            let opts = pdf_oxide::converters::ConversionOptions::default();
            let mut csv = String::new();
            for i in 0..page_count {
                if let Ok(md) = doc.to_markdown(i, &opts) {
                    for line in md.lines() {
                        let t = line.trim();
                        if t.starts_with('|') && !t.contains("---") {
                            let cells: Vec<&str> = t.split('|').filter(|s| !s.trim().is_empty()).map(|s| s.trim()).collect();
                            csv.push_str(&cells.join(","));
                            csv.push('\n');
                        }
                    }
                }
            }
            if csv.is_empty() {
                return serde_json::json!({"error": "No tabular data found"}).to_string();
            }
            std::fs::write(output, &csv).ok();
            serde_json::json!({"output": output, "rows": csv.lines().count()}).to_string()
        }
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn markdown_to_pdf(markdown: &str, output: &str, title: Option<&str>) -> String {
    use comrak::{parse_document, Arena, Options};
    use comrak::nodes::NodeValue;

    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &Options::default());

    let (doc, page1, layer1) = PdfDocument::new(title.unwrap_or("Document"), Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let layer = doc.get_page(page1).get_layer(layer1);

    let mut y = 275.0f32;
    layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));

    for node in root.descendants() {
        if y < 25.0 { break; }
        match &node.data.borrow().value {
            NodeValue::Heading(h) => {
                let text = collect_text(node);
                let size = match h.level { 1 => 20.0, 2 => 16.0, _ => 13.0 };
                layer.use_text(&text, size, Mm(15.0), Mm(y), &bold);
                y -= size * 0.5 + 4.0;
            }
            NodeValue::Paragraph => {
                let text = collect_text(node);
                if !text.is_empty() {
                    for chunk in wrap_text(&text, 90) {
                        if y < 25.0 { break; }
                        layer.use_text(&chunk, 10.0, Mm(15.0), Mm(y), &font);
                        y -= 5.0;
                    }
                    y -= 3.0;
                }
            }
            NodeValue::Item(_) => {
                let text = collect_text(node);
                if !text.is_empty() {
                    layer.use_text(&format!("• {}", text), 10.0, Mm(20.0), Mm(y), &font);
                    y -= 5.0;
                }
            }
            NodeValue::CodeBlock(cb) => {
                layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
                for line in cb.literal.lines() {
                    if y < 25.0 { break; }
                    layer.use_text(line, 9.0, Mm(20.0), Mm(y), &font);
                    y -= 4.5;
                }
                layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));
                y -= 3.0;
            }
            _ => {}
        }
    }

    match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(output).unwrap())) {
        Ok(_) => serde_json::json!({"output": output}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub fn images_to_pdf(image_paths: &[String], output: &str) -> String {
    if image_paths.is_empty() {
        return serde_json::json!({"error": "No image paths provided"}).to_string();
    }
    let (doc, page1, layer1) = PdfDocument::new("Images", Mm(210.0), Mm(297.0), "Layer 1");
    let mut page_count = 0u32;

    for (i, path) in image_paths.iter().enumerate() {
        let layer = if i == 0 {
            doc.get_page(page1).get_layer(layer1)
        } else {
            let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
            doc.get_page(page).get_layer(layer)
        };
        if let Ok(img_bytes) = std::fs::read(path) {
            if let Ok(img) = image_crate::load_from_memory(&img_bytes) {
                let rgb = img.to_rgb8();
                let (w, h) = rgb.dimensions();
                let image = Image::from(ImageXObject {
                    width: Px(w as usize), height: Px(h as usize),
                    color_space: ColorSpace::Rgb, bits_per_component: ColorBits::Bit8,
                    interpolate: true, image_data: rgb.into_raw(),
                    image_filter: None, clipping_bbox: None, smask: None,
                });
                let scale = (180.0 / (w as f32 * 0.353)).min(267.0 / (h as f32 * 0.353)).min(1.0);
                image.add_to_layer(layer, ImageTransform {
                    translate_x: Some(Mm(15.0)), translate_y: Some(Mm(15.0)),
                    scale_x: Some(scale), scale_y: Some(scale),
                    ..Default::default()
                });
                page_count += 1;
            }
        }
    }
    match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(output).unwrap())) {
        Ok(_) => serde_json::json!({"output": output, "pages": page_count}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

fn collect_text<'a>(node: &'a comrak::arena_tree::Node<'a, std::cell::RefCell<comrak::nodes::Ast>>) -> String {
    use comrak::nodes::NodeValue;
    let mut text = String::new();
    for child in node.descendants() {
        match &child.data.borrow().value {
            NodeValue::Text(ref t) => text.push_str(t),
            NodeValue::Code(ref c) => text.push_str(&c.literal),
            NodeValue::SoftBreak | NodeValue::LineBreak => text.push(' '),
            _ => {}
        }
    }
    text
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > max_chars { lines.push(current.clone()); current.clear(); }
        if !current.is_empty() { current.push(' '); }
        current.push_str(word);
    }
    if !current.is_empty() { lines.push(current); }
    lines
}
