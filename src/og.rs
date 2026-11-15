use crate::site::esc;
use resvg::{tiny_skia, usvg};
use std::fmt::Write;
use std::path::Path;

pub struct Og {
    pub kicker: String,
    pub title: String,
    pub sub: String,
    pub stat: Option<(String, String)>,
    pub bars: Vec<(String, String, f64, String)>,
    pub prelim: bool,
}

fn wrap(s: &str, width: usize, max_lines: usize) -> Vec<String> {
    let mut lines: Vec<String> = vec![String::new()];
    for w in s.split_whitespace() {
        let cur = lines.last_mut().unwrap();
        if !cur.is_empty() && cur.chars().count() + 1 + w.chars().count() > width {
            lines.push(w.to_string());
        } else {
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(w);
        }
    }
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        lines.last_mut().unwrap().push('…');
    }
    lines
}

pub fn svg(o: &Og) -> String {
    let (ink, mute, line, warn) = ("#0d1117", "#5d6673", "#e6e8ec", "#b54708");
    let f = "font-family=\"Inter Variable\"";
    let mut s = String::new();
    write!(s, r##"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630" viewBox="0 0 1200 630"><rect width="1200" height="630" fill="#ffffff"/>"##).unwrap();
    write!(s, r##"<rect x="64" y="56" width="276" height="52" rx="26" fill="{ink}"/><path d="M88 94 L112 70" stroke="#5b8cff" stroke-width="4" stroke-linecap="round"/><path d="M86 78 H114" stroke="#f97066" stroke-width="3"/><text x="126" y="90" {f} font-weight="600" font-size="22" fill="#fff">AgentRAMBench</text>"##).unwrap();
    write!(s, r#"<text x="1136" y="90" text-anchor="end" {f} font-size="20" fill="{mute}">{}</text>"#, esc(&o.kicker)).unwrap();
    if o.prelim {
        write!(s, r#"<text x="358" y="89" {f} font-weight="600" font-size="16" fill="{warn}">PRELIMINARY</text>"#).unwrap();
    }
    let stat_w = o.stat.is_some() && o.bars.is_empty();
    let tw = if stat_w { 17 } else { 24 };
    let mut y = 236;
    for line in wrap(&o.title, tw, 2) {
        write!(s, r#"<text x="64" y="{y}" {f} font-weight="700" font-size="84" letter-spacing="-3" fill="{ink}">{}</text>"#, esc(&line)).unwrap();
        y += 92;
    }
    if o.bars.is_empty() {
        for line in wrap(&o.sub, if stat_w { 38 } else { 56 }, 3) {
            write!(s, r#"<text x="64" y="{y}" {f} font-size="27" fill="{mute}">{}</text>"#, esc(&line)).unwrap();
            y += 40;
        }
    } else {
        let n = o.bars.len().min(6);
        let (x0, x1, base) = (64.0, 1136.0, 560.0);
        let slot = (x1 - x0) / n as f64;
        let bw = (slot * 0.55).min(150.0);
        write!(s, r#"<text x="64" y="{}" {f} font-size="26" fill="{mute}">{}</text>"#, y - 30, esc(&o.sub)).unwrap();
        let top = (y as f64) + 10.0;
        for (i, (name, v, frac, col)) in o.bars.iter().take(n).enumerate() {
            let cx = x0 + slot * (i as f64 + 0.5);
            let hgt = (frac.clamp(0.0, 1.0) * (base - top - 70.0)).max(4.0);
            write!(s, r##"<rect x="{:.0}" y="{:.0}" width="{bw:.0}" height="{hgt:.0}" rx="8" fill="{col}"/><text x="{cx:.0}" y="{:.0}" text-anchor="middle" {f} font-weight="600" font-size="24" fill="{ink}">{}</text><text x="{cx:.0}" y="{:.0}" text-anchor="middle" {f} font-weight="500" font-size="22" fill="{mute}">{}</text>"##, cx - bw / 2.0, base - hgt, base - hgt - 12.0, esc(v), base + 34.0, esc(name)).unwrap();
        }
        write!(s, r#"<line x1="64" x2="1136" y1="560" y2="560" stroke="{line}" stroke-width="2"/>"#).unwrap();
    }
    if let Some((v, l)) = &o.stat {
        if o.bars.is_empty() {
            write!(s, r#"<text x="1136" y="430" text-anchor="end" {f} font-weight="700" font-size="190" letter-spacing="-8" fill="{ink}">{}</text>"#, esc(v)).unwrap();
            for (i, line) in wrap(l, 30, 2).iter().enumerate() {
                write!(s, r#"<text x="1136" y="{}" text-anchor="end" {f} font-size="24" fill="{mute}">{}</text>"#, 480 + i * 34, esc(line)).unwrap();
            }
        }
    }
    if o.bars.is_empty() {
        write!(s, r#"<text x="64" y="574" {f} font-size="20" fill="{mute}">agentrambench.com</text>"#).unwrap();
    }
    s.push_str("</svg>");
    s
}

pub fn render_all(out: &Path, jobs: &[(String, Og)]) {
    let mut opt = usvg::Options::default();
    let db = opt.fontdb_mut();
    for f in [
        &include_bytes!("../assets/fonts/Inter-400.ttf")[..],
        &include_bytes!("../assets/fonts/Inter-600.ttf")[..],
        &include_bytes!("../assets/fonts/Inter-700.ttf")[..],
    ] {
        db.load_font_data(f.to_vec());
    }
    std::fs::create_dir_all(out.join("og")).unwrap();
    for (name, o) in jobs {
        let tree = usvg::Tree::from_str(&svg(o), &opt).expect("og svg");
        let mut pm = tiny_skia::Pixmap::new(1200, 630).unwrap();
        resvg::render(&tree, tiny_skia::Transform::default(), &mut pm.as_mut());
        std::fs::write(out.join(format!("og/{name}.png")), pm.encode_png().unwrap()).unwrap();
    }
}
