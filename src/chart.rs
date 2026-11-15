use crate::site::esc;
use std::fmt::Write;

pub struct Bar {
    pub name: String,
    pub color: String,
    pub v: f64,
    pub lo: f64,
    pub hi: f64,
    pub label: String,
    pub note: String,
}

fn wrap_name(s: &str) -> Vec<String> {
    if s.chars().count() <= 11 {
        return vec![s.to_string()];
    }
    match s.rsplit_once(' ') {
        Some((a, b)) => vec![a.to_string(), b.to_string()],
        None => vec![s.to_string()],
    }
}

pub fn bars(bs: &[Bar], aria: &str) -> String {
    let has_note = bs.iter().any(|x| !x.note.is_empty());
    let (w, h) = (360.0, if has_note { 280.0 } else { 266.0 });
    let (l, r, t, b) = (4.0, 4.0, 26.0, if has_note { 60.0 } else { 46.0 });
    let vmax = bs.iter().map(|x| x.hi.max(x.v)).fold(0.0, f64::max);
    let vmax = if vmax <= 0.0 { 1.0 } else { vmax * 1.04 };
    let ph = h - t - b;
    let py = |v: f64| t + ph - v / vmax * ph;
    let n = bs.len().max(1) as f64;
    let slot = (w - l - r) / n;
    let bw = (slot * 0.62).min(56.0);
    let mut s = String::new();
    write!(s, r#"<svg viewBox="0 0 {w} {h}" role="img" aria-label="{}" xmlns="http://www.w3.org/2000/svg">"#, esc(aria)).unwrap();
    for k in 1..=3 {
        let y = t + ph * (k as f64) / 4.0 - ph / 4.0 + ph / 4.0 * 0.0;
        write!(s, r#"<line class="gl" x1="{l}" x2="{}" y1="{y:.1}" y2="{y:.1}"/>"#, w - r).unwrap();
    }
    write!(s, r#"<line x1="{l}" x2="{}" y1="{y}" y2="{y}" stroke="var(--line2)"/>"#, w - r, y = t + ph).unwrap();
    for (i, x) in bs.iter().enumerate() {
        let cx = l + slot * (i as f64 + 0.5);
        let y = py(x.v);
        let hgt = (t + ph - y).max(1.5);
        write!(s, r#"<rect x="{:.1}" y="{:.1}" width="{bw:.1}" height="{hgt:.1}" rx="4" fill="{}"/>"#, cx - bw / 2.0, t + ph - hgt, x.color).unwrap();
        if x.hi > x.lo {
            let (y1, y2) = (py(x.hi), py(x.lo));
            write!(s, r#"<path class="wsk" d="M{cx:.1} {y1:.1}V{y2:.1}M{:.1} {y1:.1}h10M{:.1} {y2:.1}h10" fill="none"/>"#, cx - 5.0, cx - 5.0).unwrap();
        }
        let ty = py(x.hi.max(x.v)) - 8.0;
        write!(s, r#"<text class="vl" x="{cx:.1}" y="{ty:.1}" text-anchor="middle">{}</text>"#, esc(&x.label)).unwrap();
        let names = wrap_name(&x.name);
        for (k, line) in names.iter().enumerate() {
            write!(s, r#"<text class="nl" x="{cx:.1}" y="{:.1}" text-anchor="middle">{}</text>"#, t + ph + 19.0 + k as f64 * 15.0, esc(line)).unwrap();
        }
        if !x.note.is_empty() {
            write!(s, r#"<text class="ax" x="{cx:.1}" y="{:.1}" text-anchor="middle">{}</text>"#, t + ph + 19.0 + names.len() as f64 * 15.0, esc(&x.note)).unwrap();
        }
    }
    s.push_str("</svg>");
    s
}
