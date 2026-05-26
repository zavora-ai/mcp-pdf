use printpdf::*;

pub struct InvoiceData {
    pub output: String,
    pub company: String,
    pub items: Vec<(String, u32, i64)>, // (description, qty, unit_price_cents)
    pub customer: String,
    pub invoice_number: String,
    pub logo: Option<String>,
    pub style: String,
    pub qr_data: Option<String>,
}

pub fn create_invoice(data: InvoiceData) -> String {
    let (doc, page1, layer1) = PdfDocument::new("Invoice", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let layer = doc.get_page(page1).get_layer(layer1);

    // Logo
    let mut logo_status = "none".to_string();
    if let Some(ref logo_path) = data.logo {
        if let Ok(img_bytes) = std::fs::read(logo_path) {
            if let Ok(img) = image_crate::load_from_memory(&img_bytes) {
                let rgb = img.to_rgb8();
                let (w, h) = rgb.dimensions();
                let image = Image::from(ImageXObject {
                    width: Px(w as usize), height: Px(h as usize),
                    color_space: ColorSpace::Rgb, bits_per_component: ColorBits::Bit8,
                    interpolate: true, image_data: rgb.into_raw(),
                    image_filter: None, clipping_bbox: None, smask: None,
                });
                image.add_to_layer(layer.clone(), ImageTransform {
                    translate_x: Some(Mm(15.0)), translate_y: Some(Mm(272.0)),
                    scale_x: Some(0.8), scale_y: Some(0.8),
                    ..Default::default()
                });
                logo_status = format!("added ({}x{})", w, h);
            }
        }
    }

    // INVOICE title
    layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
    layer.use_text("INVOICE", 24.0, Mm(150.0), Mm(278.0), &bold);

    // Invoice details
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text(&data.invoice_number, 10.0, Mm(150.0), Mm(270.0), &font);
    layer.use_text(&chrono::Utc::now().format("%B %d, %Y").to_string(), 10.0, Mm(150.0), Mm(264.0), &font);

    // Separator
    layer.set_outline_color(Color::Rgb(Rgb::new(0.85, 0.85, 0.85, None)));
    layer.set_outline_thickness(0.5);
    layer.add_line(Line { points: vec![
        (Point::new(Mm(15.0), Mm(258.0)), false),
        (Point::new(Mm(195.0), Mm(258.0)), false),
    ], is_closed: false });

    // Bill To / From
    layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
    layer.use_text("Bill To", 8.0, Mm(15.0), Mm(250.0), &font);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    layer.use_text(&data.customer, 11.0, Mm(15.0), Mm(244.0), &bold);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
    layer.use_text("From", 8.0, Mm(15.0), Mm(270.0), &font);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    layer.use_text(&data.company, 11.0, Mm(15.0), Mm(264.0), &bold);

    // Table header
    let mut y = 220.0f32;
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text("Description", 8.0, Mm(15.0), Mm(y), &bold);
    layer.use_text("Qty", 8.0, Mm(120.0), Mm(y), &bold);
    layer.use_text("Price", 8.0, Mm(140.0), Mm(y), &bold);
    layer.use_text("Amount", 8.0, Mm(175.0), Mm(y), &bold);
    y -= 3.0;
    layer.add_line(Line { points: vec![
        (Point::new(Mm(15.0), Mm(y)), false),
        (Point::new(Mm(195.0), Mm(y)), false),
    ], is_closed: false });
    y -= 7.0;

    // Rows
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    let mut total: i64 = 0;
    for (desc, qty, price) in &data.items {
        let amt = *qty as i64 * price;
        total += amt;
        layer.use_text(desc, 10.0, Mm(15.0), Mm(y), &font);
        layer.use_text(&qty.to_string(), 10.0, Mm(123.0), Mm(y), &font);
        layer.use_text(&format!("${:.2}", *price as f64 / 100.0), 10.0, Mm(140.0), Mm(y), &font);
        layer.use_text(&format!("${:.2}", amt as f64 / 100.0), 10.0, Mm(175.0), Mm(y), &font);
        y -= 7.0;
    }

    // Total
    y -= 3.0;
    layer.add_line(Line { points: vec![
        (Point::new(Mm(130.0), Mm(y)), false),
        (Point::new(Mm(195.0), Mm(y)), false),
    ], is_closed: false });
    y -= 8.0;
    layer.use_text("Total", 10.0, Mm(140.0), Mm(y), &bold);
    layer.use_text(&format!("${:.2}", total as f64 / 100.0), 12.0, Mm(172.0), Mm(y), &bold);

    // QR Code
    if let Some(ref qr_data) = data.qr_data {
        if let Ok(qr) = qrcode::QrCode::new(qr_data.as_bytes()) {
            let modules = qr.to_colors();
            let size = qr.width() as u32;
            let scale = 3u32;
            let img_size = size * scale;
            // Build RGB image from QR modules
            let mut rgb = vec![255u8; (img_size * img_size * 3) as usize];
            for y in 0..size {
                for x in 0..size {
                    let is_dark = modules[(y * size + x) as usize] == qrcode::Color::Dark;
                    if is_dark {
                        for dy in 0..scale {
                            for dx in 0..scale {
                                let px = ((y * scale + dy) * img_size + (x * scale + dx)) as usize * 3;
                                rgb[px] = 0; rgb[px+1] = 0; rgb[px+2] = 0;
                            }
                        }
                    }
                }
            }
            let image = Image::from(ImageXObject {
                width: Px(img_size as usize), height: Px(img_size as usize),
                color_space: ColorSpace::Rgb, bits_per_component: ColorBits::Bit8,
                interpolate: false, image_data: rgb,
                image_filter: None, clipping_bbox: None, smask: None,
            });
            image.add_to_layer(layer.clone(), ImageTransform {
                translate_x: Some(Mm(170.0)), translate_y: Some(Mm(30.0)),
                scale_x: Some(0.35), scale_y: Some(0.35),
                ..Default::default()
            });
        }
    }

    // Footer
    layer.set_fill_color(Color::Rgb(Rgb::new(0.7, 0.7, 0.7, None)));
    layer.use_text("Thank you for your business.", 9.0, Mm(15.0), Mm(20.0), &font);

    match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(&data.output).unwrap())) {
        Ok(_) => serde_json::json!({
            "output": data.output, "total": format!("${:.2}", total as f64 / 100.0),
            "items": data.items.len(), "style": data.style, "logo": logo_status,
        }).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub struct ReceiptData {
    pub output: String,
    pub company: String,
    pub customer: String,
    pub receipt_number: String,
    pub items: Vec<(String, u32, i64)>,
    pub payment_method: String,
    pub _logo: Option<String>,
    pub stamp: Option<String>,
    pub stamp_style: String, // "circle" or "rectangle"
}

pub fn create_receipt(data: ReceiptData) -> String {
    let (doc, page1, layer1) = PdfDocument::new("Receipt", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let layer = doc.get_page(page1).get_layer(layer1);

    // Header
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    layer.use_text(&data.company, 14.0, Mm(15.0), Mm(275.0), &bold);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
    layer.use_text("RECEIPT", 20.0, Mm(155.0), Mm(275.0), &bold);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text(&data.receipt_number, 10.0, Mm(155.0), Mm(267.0), &font);
    layer.use_text(&chrono::Utc::now().format("%B %d, %Y").to_string(), 10.0, Mm(155.0), Mm(261.0), &font);

    // Customer
    layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
    layer.use_text("Received from", 8.0, Mm(15.0), Mm(250.0), &font);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    layer.use_text(&data.customer, 11.0, Mm(15.0), Mm(244.0), &bold);

    // Items
    let mut y = 220.0f32;
    let mut total: i64 = 0;
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    for (desc, qty, price) in &data.items {
        let amt = *qty as i64 * price;
        total += amt;
        layer.use_text(desc, 10.0, Mm(15.0), Mm(y), &font);
        layer.use_text(&format!("${:.2}", amt as f64 / 100.0), 10.0, Mm(170.0), Mm(y), &font);
        y -= 7.0;
    }

    // Total + payment method
    y -= 5.0;
    layer.set_outline_color(Color::Rgb(Rgb::new(0.85, 0.85, 0.85, None)));
    layer.set_outline_thickness(0.5);
    layer.add_line(Line { points: vec![(Point::new(Mm(15.0), Mm(y)), false), (Point::new(Mm(195.0), Mm(y)), false)], is_closed: false });
    y -= 8.0;
    layer.use_text("Total Paid", 10.0, Mm(15.0), Mm(y), &bold);
    layer.use_text(&format!("${:.2}", total as f64 / 100.0), 12.0, Mm(170.0), Mm(y), &bold);
    y -= 10.0;
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text(&format!("Payment method: {}", data.payment_method), 9.0, Mm(15.0), Mm(y), &font);

    // Stamp
    if let Some(ref stamp) = data.stamp {
        let stamp_text = match stamp.to_lowercase().as_str() {
            "received" => "RECEIVED",
            "paid" => "PAID",
            "void" => "VOID",
            "approved" => "APPROVED",
            "rejected" => "REJECTED",
            _ => stamp.as_str(),
        };
        let is_void = stamp_text == "VOID" || stamp_text == "REJECTED";
        let color = if is_void {
            Rgb::new(0.7, 0.0, 0.0, None)
        } else {
            Rgb::new(0.05, 0.10, 0.35, None)
        };

        if data.stamp_style == "rectangle" {
            // Rectangle stamp: double border, bold text, date
            use printpdf::path::PaintMode;
            let cx = 152.5f32;
            let cy = 115.0f32;
            let hw = 28.0f32; // half width
            let hh = 12.0f32; // half height

            layer.set_outline_color(Color::Rgb(color.clone()));
            layer.set_outline_thickness(2.5);
            layer.add_rect(Rect::new(Mm(cx - hw), Mm(cy - hh), Mm(cx + hw), Mm(cy + hh)).with_mode(PaintMode::Stroke));
            layer.set_outline_thickness(1.2);
            layer.add_rect(Rect::new(Mm(cx - hw + 2.0), Mm(cy - hh + 2.0), Mm(cx + hw - 2.0), Mm(cy + hh - 2.0)).with_mode(PaintMode::Stroke));

            // Company name top
            layer.set_fill_color(Color::Rgb(color.clone()));
            let comp = data.company.to_uppercase();
            let comp_w = comp.len() as f32 * 5.0 * 0.6 * 0.353;
            layer.use_text(&comp, 5.0, Mm(cx - comp_w / 2.0), Mm(cy + 6.0), &bold);

            // Main text centered
            let font_size = 16.0f32;
            let text_w = stamp_text.len() as f32 * font_size * 0.6 * 0.353;
            layer.use_text(stamp_text, font_size, Mm(cx - text_w / 2.0), Mm(cy - 1.0), &bold);

            // Date bottom
            let date_str = chrono::Utc::now().format("DATE: %d/%m/%Y").to_string();
            let date_w = date_str.len() as f32 * 5.0 * 0.6 * 0.353;
            layer.use_text(&date_str, 5.0, Mm(cx - date_w / 2.0), Mm(cy - 7.5), &font);
        } else {
            // Circle stamp (default)
            let cx = 152.5f32;
            let cy = 115.0f32;
            let r_outer = 24.0f32;
            let r_inner = 18.0f32;

            layer.set_outline_color(Color::Rgb(color.clone()));
            layer.set_outline_thickness(2.0);
            draw_circle(&layer, cx, cy, r_outer);
            layer.set_outline_thickness(1.5);
            draw_circle(&layer, cx, cy, r_inner);

            layer.set_fill_color(Color::Rgb(color.clone()));
            let arc_radius = (r_outer + r_inner) / 2.0;
            let top_text = data.company.to_uppercase();
            draw_text_on_arc(&layer, &bold, &top_text, cx, cy, arc_radius, 6.5, true);
            draw_text_on_arc(&layer, &bold, "BY: AUTHORIZED", cx, cy, arc_radius, 6.0, false);

            layer.set_outline_thickness(0.8);
            layer.add_line(Line { points: vec![
                (Point::new(Mm(cx - 15.0), Mm(cy + 5.0)), false),
                (Point::new(Mm(cx + 15.0), Mm(cy + 5.0)), false),
            ], is_closed: false });

            let font_size = 14.0f32;
            let char_w_mm = font_size * 0.6 * 0.353;
            let text_w = stamp_text.len() as f32 * char_w_mm;
            layer.use_text(stamp_text, font_size, Mm(cx - text_w / 2.0), Mm(cy - 1.5), &bold);

            layer.add_line(Line { points: vec![
                (Point::new(Mm(cx - 15.0), Mm(cy - 2.5)), false),
                (Point::new(Mm(cx + 15.0), Mm(cy - 2.5)), false),
            ], is_closed: false });

            let date_str = chrono::Utc::now().format("DATE: %d/%m/%Y").to_string();
            let date_w = date_str.len() as f32 * 5.0 * 0.35 * 0.353;
            layer.use_text(&date_str, 5.0, Mm(cx - date_w / 2.0), Mm(cy - 6.5), &font);
        }
    }

    match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(&data.output).unwrap())) {
        Ok(_) => serde_json::json!({"output": data.output, "total": format!("${:.2}", total as f64 / 100.0), "stamp": data.stamp}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub struct LetterData {
    pub output: String,
    pub from_name: String,
    pub from_company: Option<String>,
    pub to_name: String,
    pub to_company: Option<String>,
    pub subject: Option<String>,
    pub body: String,
    pub _logo: Option<String>,
}

pub fn create_letter(data: LetterData) -> String {
    let (doc, page1, layer1) = PdfDocument::new("Letter", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let layer = doc.get_page(page1).get_layer(layer1);

    // From (letterhead)
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    layer.use_text(&data.from_name, 12.0, Mm(15.0), Mm(275.0), &bold);
    if let Some(ref co) = data.from_company {
        layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
        layer.use_text(co, 10.0, Mm(15.0), Mm(269.0), &font);
    }

    // Date
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text(&chrono::Utc::now().format("%B %d, %Y").to_string(), 10.0, Mm(15.0), Mm(255.0), &font);

    // To
    let mut y = 240.0f32;
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    layer.use_text(&data.to_name, 10.0, Mm(15.0), Mm(y), &font);
    if let Some(ref co) = data.to_company {
        y -= 5.0;
        layer.use_text(co, 10.0, Mm(15.0), Mm(y), &font);
    }

    // Subject
    y -= 15.0;
    if let Some(ref subj) = data.subject {
        layer.use_text(&format!("Re: {}", subj), 10.0, Mm(15.0), Mm(y), &bold);
        y -= 10.0;
    }

    // Body (simple line wrapping)
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    for line in data.body.lines() {
        layer.use_text(line, 10.0, Mm(15.0), Mm(y), &font);
        y -= 5.0;
    }

    // Signature
    y -= 15.0;
    layer.use_text("Sincerely,", 10.0, Mm(15.0), Mm(y), &font);
    y -= 12.0;
    layer.use_text(&data.from_name, 10.0, Mm(15.0), Mm(y), &bold);

    match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(&data.output).unwrap())) {
        Ok(_) => serde_json::json!({"output": data.output}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub struct CertificateData {
    pub output: String,
    pub recipient: String,
    pub title: String,
    pub description: Option<String>,
    pub issuer: String,
    pub date: Option<String>,
    pub style: String, // classic, modern, elegant, academic, minimal
}

pub fn create_certificate(data: CertificateData) -> String {
    // All certificates are landscape A4
    let (doc, page1, layer1) = PdfDocument::new("Certificate", Mm(297.0), Mm(210.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let layer = doc.get_page(page1).get_layer(layer1);
    let date = data.date.unwrap_or_else(|| chrono::Utc::now().format("%B %d, %Y").to_string());

    match data.style.as_str() {
        "modern" => render_cert_modern(&layer, &font, &bold, &data.recipient, &data.title, data.description.as_deref(), &data.issuer, &date),
        "elegant" => render_cert_elegant(&layer, &font, &bold, &data.recipient, &data.title, data.description.as_deref(), &data.issuer, &date),
        "academic" => render_cert_academic(&layer, &font, &bold, &data.recipient, &data.title, data.description.as_deref(), &data.issuer, &date),
        "minimal" => render_cert_minimal(&layer, &font, &bold, &data.recipient, &data.title, data.description.as_deref(), &data.issuer, &date),
        _ => render_cert_classic(&layer, &font, &bold, &data.recipient, &data.title, data.description.as_deref(), &data.issuer, &date),
    }

    match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(&data.output).unwrap())) {
        Ok(_) => serde_json::json!({"output": data.output, "recipient": data.recipient, "style": data.style}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

fn render_cert_classic(layer: &PdfLayerReference, font: &IndirectFontRef, bold: &IndirectFontRef, recipient: &str, title: &str, desc: Option<&str>, issuer: &str, date: &str) {
    use printpdf::path::PaintMode;
    layer.set_outline_color(Color::Rgb(Rgb::new(0.72, 0.53, 0.04, None)));
    layer.set_outline_thickness(4.0);
    layer.add_rect(Rect::new(Mm(8.0), Mm(8.0), Mm(289.0), Mm(202.0)).with_mode(PaintMode::Stroke));
    layer.set_outline_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.30, None)));
    layer.set_outline_thickness(1.5);
    layer.add_rect(Rect::new(Mm(14.0), Mm(14.0), Mm(283.0), Mm(196.0)).with_mode(PaintMode::Stroke));

    // Decorative line under title area
    layer.set_outline_color(Color::Rgb(Rgb::new(0.72, 0.53, 0.04, None)));
    layer.set_outline_thickness(0.8);
    layer.add_line(Line { points: vec![(Point::new(Mm(80.0), Mm(155.0)), false), (Point::new(Mm(217.0), Mm(155.0)), false)], is_closed: false });

    // "CERTIFICATE OF"
    layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
    layer.use_text("CERTIFICATE OF", 11.0, Mm(113.0), Mm(175.0), font);
    // Title (large)
    layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.30, None)));
    layer.use_text(title, 26.0, Mm(center_x(title, 26.0)), Mm(162.0), bold);

    // "This is to certify that"
    layer.set_fill_color(Color::Rgb(Rgb::new(0.45, 0.45, 0.45, None)));
    layer.use_text("This is to certify that", 10.0, Mm(118.0), Mm(140.0), font);

    // Recipient name (large, prominent)
    layer.set_fill_color(Color::Rgb(Rgb::new(0.10, 0.10, 0.10, None)));
    layer.use_text(recipient, 22.0, Mm(center_x(recipient, 22.0)), Mm(125.0), bold);

    // Decorative line under name
    layer.set_outline_color(Color::Rgb(Rgb::new(0.72, 0.53, 0.04, None)));
    layer.add_line(Line { points: vec![(Point::new(Mm(90.0), Mm(121.0)), false), (Point::new(Mm(207.0), Mm(121.0)), false)], is_closed: false });

    // Description
    if let Some(d) = desc {
        layer.set_fill_color(Color::Rgb(Rgb::new(0.35, 0.35, 0.35, None)));
        layer.use_text(d, 10.0, Mm(center_x(d, 10.0)), Mm(108.0), font);
    }

    // Issuer and date at bottom
    layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
    // Signature line left
    layer.add_line(Line { points: vec![(Point::new(Mm(50.0), Mm(50.0)), false), (Point::new(Mm(130.0), Mm(50.0)), false)], is_closed: false });
    layer.use_text(issuer, 9.0, Mm(70.0), Mm(43.0), bold);
    layer.use_text("Authorized Signatory", 7.0, Mm(68.0), Mm(37.0), font);
    // Date right
    layer.add_line(Line { points: vec![(Point::new(Mm(167.0), Mm(50.0)), false), (Point::new(Mm(247.0), Mm(50.0)), false)], is_closed: false });
    layer.use_text(date, 9.0, Mm(190.0), Mm(43.0), font);
    layer.use_text("Date", 7.0, Mm(200.0), Mm(37.0), font);
}

fn render_cert_modern(layer: &PdfLayerReference, font: &IndirectFontRef, bold: &IndirectFontRef, recipient: &str, title: &str, desc: Option<&str>, issuer: &str, date: &str) {
    use printpdf::path::PaintMode;
    // Teal accent bar on left
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.55, 0.55, None)));
    layer.add_rect(Rect::new(Mm(0.0), Mm(0.0), Mm(12.0), Mm(210.0)).with_mode(PaintMode::Fill));

    // Title
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.55, 0.55, None)));
    layer.use_text("CERTIFICATE", 11.0, Mm(25.0), Mm(180.0), font);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.12, 0.12, 0.12, None)));
    layer.use_text(title, 24.0, Mm(25.0), Mm(165.0), bold);

    // Recipient
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text("Awarded to", 9.0, Mm(25.0), Mm(135.0), font);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.55, 0.55, None)));
    layer.use_text(recipient, 20.0, Mm(25.0), Mm(122.0), bold);

    // Description
    if let Some(d) = desc {
        layer.set_fill_color(Color::Rgb(Rgb::new(0.35, 0.35, 0.35, None)));
        layer.use_text(d, 10.0, Mm(25.0), Mm(105.0), font);
    }

    // Bottom: issuer + date
    layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.55, 0.55, None)));
    layer.set_outline_thickness(0.5);
    layer.add_line(Line { points: vec![(Point::new(Mm(25.0), Mm(55.0)), false), (Point::new(Mm(120.0), Mm(55.0)), false)], is_closed: false });
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    layer.use_text(issuer, 9.0, Mm(25.0), Mm(48.0), bold);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text(date, 9.0, Mm(25.0), Mm(40.0), font);
}

fn render_cert_elegant(layer: &PdfLayerReference, font: &IndirectFontRef, bold: &IndirectFontRef, recipient: &str, title: &str, desc: Option<&str>, issuer: &str, date: &str) {
    use printpdf::path::PaintMode;
    // Navy background header area
    layer.set_fill_color(Color::Rgb(Rgb::new(0.08, 0.10, 0.22, None)));
    layer.add_rect(Rect::new(Mm(0.0), Mm(150.0), Mm(297.0), Mm(210.0)).with_mode(PaintMode::Fill));

    // Gold border
    layer.set_outline_color(Color::Rgb(Rgb::new(0.80, 0.65, 0.15, None)));
    layer.set_outline_thickness(2.0);
    layer.add_rect(Rect::new(Mm(15.0), Mm(15.0), Mm(282.0), Mm(195.0)).with_mode(PaintMode::Stroke));

    // Title in white on navy
    layer.set_fill_color(Color::Rgb(Rgb::new(0.80, 0.65, 0.15, None)));
    layer.use_text("CERTIFICATE", 13.0, Mm(110.0), Mm(185.0), font);
    layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
    layer.use_text(title, 24.0, Mm(center_x(title, 24.0)), Mm(165.0), bold);

    // Recipient on white area
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text("Presented to", 9.0, Mm(125.0), Mm(125.0), font);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.08, 0.10, 0.22, None)));
    layer.use_text(recipient, 22.0, Mm(center_x(recipient, 22.0)), Mm(108.0), bold);

    // Gold line under name
    layer.set_outline_color(Color::Rgb(Rgb::new(0.80, 0.65, 0.15, None)));
    layer.set_outline_thickness(0.8);
    layer.add_line(Line { points: vec![(Point::new(Mm(90.0), Mm(103.0)), false), (Point::new(Mm(207.0), Mm(103.0)), false)], is_closed: false });

    if let Some(d) = desc {
        layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
        layer.use_text(d, 10.0, Mm(center_x(d, 10.0)), Mm(88.0), font);
    }

    // Bottom
    layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
    layer.add_line(Line { points: vec![(Point::new(Mm(60.0), Mm(45.0)), false), (Point::new(Mm(140.0), Mm(45.0)), false)], is_closed: false });
    layer.add_line(Line { points: vec![(Point::new(Mm(157.0), Mm(45.0)), false), (Point::new(Mm(237.0), Mm(45.0)), false)], is_closed: false });
    layer.use_text(issuer, 9.0, Mm(80.0), Mm(38.0), bold);
    layer.use_text(date, 9.0, Mm(185.0), Mm(38.0), font);
}

fn render_cert_academic(layer: &PdfLayerReference, font: &IndirectFontRef, bold: &IndirectFontRef, recipient: &str, title: &str, desc: Option<&str>, issuer: &str, date: &str) {
    use printpdf::path::PaintMode;
    // Simple dark border
    layer.set_outline_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    layer.set_outline_thickness(2.0);
    layer.add_rect(Rect::new(Mm(12.0), Mm(12.0), Mm(285.0), Mm(198.0)).with_mode(PaintMode::Stroke));

    // Seal placeholder (circle)
    layer.set_outline_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
    layer.set_outline_thickness(1.5);
    // Draw a square as seal placeholder
    layer.add_rect(Rect::new(Mm(133.0), Mm(155.0), Mm(164.0), Mm(186.0)).with_mode(PaintMode::Stroke));
    layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
    layer.use_text("SEAL", 8.0, Mm(142.0), Mm(168.0), font);

    // Institution name / title
    layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));
    layer.use_text(issuer, 14.0, Mm(center_x(issuer, 14.0)), Mm(145.0), bold);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
    layer.use_text("hereby confers upon", 9.0, Mm(120.0), Mm(128.0), font);

    // Recipient
    layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
    layer.use_text(recipient, 20.0, Mm(center_x(recipient, 20.0)), Mm(113.0), bold);

    // Title
    layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
    layer.use_text("the", 9.0, Mm(143.0), Mm(98.0), font);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));
    layer.use_text(title, 16.0, Mm(center_x(title, 16.0)), Mm(85.0), bold);

    if let Some(d) = desc {
        layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
        layer.use_text(d, 9.0, Mm(center_x(d, 9.0)), Mm(70.0), font);
    }

    // Date + signatures
    layer.set_outline_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.set_outline_thickness(0.5);
    layer.add_line(Line { points: vec![(Point::new(Mm(50.0), Mm(40.0)), false), (Point::new(Mm(130.0), Mm(40.0)), false)], is_closed: false });
    layer.add_line(Line { points: vec![(Point::new(Mm(167.0), Mm(40.0)), false), (Point::new(Mm(247.0), Mm(40.0)), false)], is_closed: false });
    layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
    layer.use_text("Dean / Director", 7.0, Mm(72.0), Mm(33.0), font);
    layer.use_text(date, 9.0, Mm(190.0), Mm(33.0), font);
}

fn render_cert_minimal(layer: &PdfLayerReference, font: &IndirectFontRef, bold: &IndirectFontRef, recipient: &str, title: &str, desc: Option<&str>, issuer: &str, date: &str) {
    // Single accent line at top
    layer.set_outline_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    layer.set_outline_thickness(2.0);
    layer.add_line(Line { points: vec![(Point::new(Mm(30.0), Mm(185.0)), false), (Point::new(Mm(267.0), Mm(185.0)), false)], is_closed: false });

    // Title
    layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
    layer.use_text("CERTIFICATE", 10.0, Mm(127.0), Mm(170.0), font);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));
    layer.use_text(title, 20.0, Mm(center_x(title, 20.0)), Mm(155.0), bold);

    // Recipient
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text("Awarded to", 9.0, Mm(133.0), Mm(125.0), font);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.1, 0.1, 0.1, None)));
    layer.use_text(recipient, 18.0, Mm(center_x(recipient, 18.0)), Mm(110.0), bold);

    if let Some(d) = desc {
        layer.set_fill_color(Color::Rgb(Rgb::new(0.45, 0.45, 0.45, None)));
        layer.use_text(d, 10.0, Mm(center_x(d, 10.0)), Mm(90.0), font);
    }

    // Bottom line
    layer.set_outline_thickness(0.5);
    layer.add_line(Line { points: vec![(Point::new(Mm(100.0), Mm(55.0)), false), (Point::new(Mm(197.0), Mm(55.0)), false)], is_closed: false });
    layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
    layer.use_text(issuer, 9.0, Mm(center_x(issuer, 9.0)), Mm(48.0), bold);
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text(date, 9.0, Mm(center_x(date, 9.0)), Mm(40.0), font);
}

/// Approximate center X for text on landscape A4 (297mm wide)
fn center_x(text: &str, font_size: f32) -> f32 {
    // Helvetica avg char width ≈ 0.5 * font_size in points; 1pt = 0.353mm
    let text_width_mm = text.len() as f32 * font_size * 0.5 * 0.353;
    ((297.0 - text_width_mm) / 2.0).max(15.0)
}

pub struct ReportData {
    pub output: String,
    pub title: String,
    pub author: Option<String>,
    pub sections: Vec<(String, String)>, // (heading, body)
}

pub fn create_report(data: ReportData) -> String {
    let (doc, page1, layer1) = PdfDocument::new(&data.title, Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let layer = doc.get_page(page1).get_layer(layer1);

    // Title page
    layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));
    layer.use_text(&data.title, 22.0, Mm(15.0), Mm(260.0), &bold);

    if let Some(ref author) = data.author {
        layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
        layer.use_text(author, 11.0, Mm(15.0), Mm(250.0), &font);
    }
    layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    layer.use_text(&chrono::Utc::now().format("%B %d, %Y").to_string(), 10.0, Mm(15.0), Mm(243.0), &font);

    // Separator
    layer.set_outline_color(Color::Rgb(Rgb::new(0.85, 0.85, 0.85, None)));
    layer.set_outline_thickness(0.5);
    layer.add_line(Line { points: vec![(Point::new(Mm(15.0), Mm(235.0)), false), (Point::new(Mm(195.0), Mm(235.0)), false)], is_closed: false });

    // Sections
    let mut y = 220.0f32;
    for (heading, body) in &data.sections {
        if y < 40.0 { break; } // Simple page overflow guard
        layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));
        layer.use_text(heading, 13.0, Mm(15.0), Mm(y), &bold);
        y -= 8.0;
        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        for line in body.lines() {
            if y < 30.0 { break; }
            layer.use_text(line, 10.0, Mm(15.0), Mm(y), &font);
            y -= 5.0;
        }
        y -= 8.0;
    }

    match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(&data.output).unwrap())) {
        Ok(_) => serde_json::json!({"output": data.output, "sections": data.sections.len()}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

pub struct ContractData {
    pub output: String,
    pub title: String,
    pub parties: Vec<String>,
    pub effective_date: String,
    pub clauses: Vec<(String, String)>, // (title, body)
    pub signatures: Vec<String>,
}

pub fn create_contract(data: ContractData) -> String {
    let (doc, page1, layer1) = PdfDocument::new(&data.title, Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let layer = doc.get_page(page1).get_layer(layer1);

    // Title
    layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));
    layer.use_text(&data.title, 16.0, Mm(15.0), Mm(270.0), &bold);

    // Parties
    let mut y = 255.0f32;
    layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
    let parties_text = format!("Between: {}", data.parties.join(" and "));
    layer.use_text(&parties_text, 10.0, Mm(15.0), Mm(y), &font);
    y -= 6.0;
    layer.use_text(&format!("Effective Date: {}", data.effective_date), 10.0, Mm(15.0), Mm(y), &font);
    y -= 12.0;

    // Clauses
    layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));
    for (i, (title, body)) in data.clauses.iter().enumerate() {
        if y < 60.0 { break; }
        layer.use_text(&format!("{}. {}", i + 1, title), 11.0, Mm(15.0), Mm(y), &bold);
        y -= 6.0;
        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        for line in body.lines() {
            if y < 60.0 { break; }
            layer.use_text(line, 9.5, Mm(20.0), Mm(y), &font);
            y -= 4.5;
        }
        y -= 6.0;
        layer.set_fill_color(Color::Rgb(Rgb::new(0.15, 0.15, 0.15, None)));
    }

    // Signature blocks
    y = 50.0;
    layer.set_outline_color(Color::Rgb(Rgb::new(0.7, 0.7, 0.7, None)));
    layer.set_outline_thickness(0.5);
    let mut x = 15.0f32;
    for name in &data.signatures {
        layer.add_line(Line { points: vec![(Point::new(Mm(x), Mm(y)), false), (Point::new(Mm(x + 70.0), Mm(y)), false)], is_closed: false });
        layer.set_fill_color(Color::Rgb(Rgb::new(0.3, 0.3, 0.3, None)));
        layer.use_text(name, 9.0, Mm(x), Mm(y - 5.0), &font);
        layer.use_text("Date: ___________", 8.0, Mm(x), Mm(y - 10.0), &font);
        x += 90.0;
    }

    match doc.save(&mut std::io::BufWriter::new(std::fs::File::create(&data.output).unwrap())) {
        Ok(_) => serde_json::json!({"output": data.output, "clauses": data.clauses.len(), "parties": data.parties.len()}).to_string(),
        Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
    }
}

/// Draw a circle approximated with 36 line segments
fn draw_circle(layer: &printpdf::PdfLayerReference, cx: f32, cy: f32, radius: f32) {
    use printpdf::*;
    let segments = 36;
    let mut points = Vec::new();
    for i in 0..=segments {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        points.push((Point::new(Mm(x), Mm(y)), false));
    }
    let line = Line { points, is_closed: true };
    layer.add_line(line);
}

/// Draw text along a circular arc (top = true for upper arc, false for lower)
fn draw_text_on_arc(layer: &printpdf::PdfLayerReference, font: &printpdf::IndirectFontRef, text: &str, cx: f32, cy: f32, radius: f32, font_size: f32, top: bool) {
    use printpdf::*;
    let char_count = text.chars().count() as f32;
    let arc_span = (char_count * 0.18).min(2.8); // wider spacing between chars

    for (i, ch) in text.chars().enumerate() {
        let t = (i as f32 + 0.5) / char_count; // 0..1 normalized position

        let angle = if top {
            // Top arc: go from left to right (PI/2+span/2 down to PI/2-span/2)
            std::f32::consts::FRAC_PI_2 + arc_span / 2.0 - t * arc_span
        } else {
            // Bottom arc: go from left to right (-PI/2-span/2 up to -PI/2+span/2)
            -std::f32::consts::FRAC_PI_2 - arc_span / 2.0 + t * arc_span
        };

        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();

        // Rotation: character upright relative to reading direction
        let rotation = if top {
            angle - std::f32::consts::FRAC_PI_2
        } else {
            angle + std::f32::consts::FRAC_PI_2
        };

        let cos_r = rotation.cos();
        let sin_r = rotation.sin();
        let tx = x * 2.8346; // mm to pt
        let ty = y * 2.8346;

        layer.begin_text_section();
        layer.set_font(font, font_size);
        layer.set_text_matrix(TextMatrix::Raw([cos_r, sin_r, -sin_r, cos_r, tx, ty]));
        layer.write_text(&ch.to_string(), font);
        layer.end_text_section();
    }
}
