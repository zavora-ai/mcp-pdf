use printpdf::*;

pub struct InvoiceData {
    pub output: String,
    pub company: String,
    pub items: Vec<(String, u32, i64)>, // (description, qty, unit_price_cents)
    pub customer: String,
    pub invoice_number: String,
    pub logo: Option<String>,
    pub style: String,
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
