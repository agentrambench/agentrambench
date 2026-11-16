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

const NAME_PX: f64 = 7.2;
const NOTE_PX: f64 = 6.7;

fn text_px(s: &str, px: f64) -> f64 {
    s.chars().count() as f64 * px
}

pub fn wrap(s: &str, max_px: f64, px: f64) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for part in s.split(" · ").filter(|p| !p.is_empty()) {
        let mut line = String::new();
        for word in part.split(' ') {
            let cand = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if !line.is_empty() && text_px(&cand, px) > max_px {
                out.push(std::mem::replace(&mut line, word.to_string()));
            } else {
                line = cand;
            }
        }
        if !line.is_empty() {
            out.push(line);
        }
    }
    out
}

fn longest_word_px(bs: &[Bar]) -> f64 {
    bs.iter()
        .flat_map(|x| x.name.split(' ').map(|w| text_px(w, NAME_PX)).chain(x.note.split(' ').filter(|w| *w != "·").map(|w| text_px(w, NOTE_PX))))
        .fold(0.0, f64::max)
}

fn columns(bs: &[Bar], aria: &str) -> String {
    let n = bs.len().max(1) as f64;
    let (l, r, t) = (4.0, 4.0, 26.0);
    let slot = (longest_word_px(bs) + 12.0).max(62.0);
    let w = 360f64.max(n * slot + l + r);
    let slot = (w - l - r) / n;
    let names: Vec<Vec<String>> = bs.iter().map(|x| wrap(&x.name, slot - 6.0, NAME_PX)).collect();
    let notes: Vec<Vec<String>> = bs.iter().map(|x| wrap(&x.note, slot - 6.0, NOTE_PX)).collect();
    let name_lines = names.iter().map(|x| x.len()).max().unwrap_or(1) as f64;
    let note_lines = notes.iter().map(|x| x.len()).max().unwrap_or(0) as f64;
    let ph = 194.0;
    let h = t + ph + 26.0 + name_lines * 15.0 + note_lines * 14.0;
    let vmax = bs.iter().map(|x| x.hi.max(x.v)).fold(0.0, f64::max);
    let vmax = if vmax <= 0.0 { 1.0 } else { vmax * 1.04 };
    let py = |v: f64| t + ph - v / vmax * ph;
    let bw = (slot * 0.62).min(56.0);
    let mut s = String::new();
    write!(s, r#"<svg class="cols" viewBox="0 0 {w:.0} {h:.0}" role="img" aria-label="{}" xmlns="http://www.w3.org/2000/svg">"#, esc(aria)).unwrap();
    for k in 0..3 {
        let y = t + ph * k as f64 / 4.0;
        write!(s, r#"<line class="gl" x1="{l}" x2="{:.1}" y1="{y:.1}" y2="{y:.1}"/>"#, w - r).unwrap();
    }
    write!(s, r#"<line x1="{l}" x2="{:.1}" y1="{y}" y2="{y}" stroke="var(--line2)"/>"#, w - r, y = t + ph).unwrap();
    for (i, x) in bs.iter().enumerate() {
        let cx = l + slot * (i as f64 + 0.5);
        let hgt = (t + ph - py(x.v)).max(1.5);
        write!(s, r#"<rect x="{:.1}" y="{:.1}" width="{bw:.1}" height="{hgt:.1}" rx="4" fill="{}"/>"#, cx - bw / 2.0, t + ph - hgt, x.color).unwrap();
        if x.hi > x.lo {
            let (y1, y2) = (py(x.hi), py(x.lo));
            write!(s, r#"<path class="wsk" d="M{cx:.1} {y1:.1}V{y2:.1}M{:.1} {y1:.1}h10M{:.1} {y2:.1}h10" fill="none"/>"#, cx - 5.0, cx - 5.0).unwrap();
        }
        write!(s, r#"<text class="vl" x="{cx:.1}" y="{:.1}" text-anchor="middle">{}</text>"#, py(x.hi.max(x.v)) - 8.0, esc(&x.label)).unwrap();
        for (k, line) in names[i].iter().enumerate() {
            write!(s, r#"<text class="nl" x="{cx:.1}" y="{:.1}" text-anchor="middle">{}</text>"#, t + ph + 19.0 + k as f64 * 15.0, esc(line)).unwrap();
        }
        for (k, line) in notes[i].iter().enumerate() {
            write!(s, r#"<text class="ax" x="{cx:.1}" y="{:.1}" text-anchor="middle">{}</text>"#, t + ph + 19.0 + name_lines * 15.0 + k as f64 * 14.0, esc(line)).unwrap();
        }
    }
    s.push_str("</svg>");
    s
}

fn rows(bs: &[Bar], aria: &str) -> String {
    let w = 320.0;
    let name_w = bs.iter().map(|x| text_px(&x.name, NAME_PX)).fold(60.0, f64::max).min(104.0);
    let x0 = name_w + 8.0;
    let label_w = bs.iter().map(|x| text_px(&x.label, 7.6)).fold(0.0, f64::max) + 6.0;
    let bar_w = w - x0 - label_w;
    let notes: Vec<Vec<String>> = bs.iter().map(|x| if !x.note.is_empty() && text_px(&x.note, NOTE_PX) <= w - x0 { vec![x.note.clone()] } else { wrap(&x.note, w - x0, NOTE_PX) }).collect();
    let names: Vec<Vec<String>> = bs.iter().map(|x| wrap(&x.name, name_w, NAME_PX)).collect();
    let row_h: Vec<f64> = (0..bs.len()).map(|i| 22.0 + (notes[i].len() as f64 * 14.0).max((names[i].len() as f64 - 1.0) * 15.0)).collect();
    let h = row_h.iter().sum::<f64>() + 4.0;
    let vmax = bs.iter().map(|x| x.hi.max(x.v)).fold(0.0, f64::max);
    let vmax = if vmax <= 0.0 { 1.0 } else { vmax * 1.02 };
    let mut s = String::new();
    write!(s, r#"<svg class="rows" viewBox="0 0 {w:.0} {h:.0}" role="img" aria-label="{}" xmlns="http://www.w3.org/2000/svg">"#, esc(aria)).unwrap();
    let mut y = 2.0;
    for (i, x) in bs.iter().enumerate() {
        let len = (x.v / vmax * bar_w).max(1.5);
        for (k, line) in names[i].iter().enumerate() {
            write!(s, r#"<text class="nl" x="0" y="{:.1}">{}</text>"#, y + 13.0 + k as f64 * 15.0, esc(line)).unwrap();
        }
        write!(s, r#"<rect x="{x0:.1}" y="{:.1}" width="{len:.1}" height="14" rx="3" fill="{}"/>"#, y + 2.0, x.color).unwrap();
        write!(s, r#"<text class="vl" x="{:.1}" y="{:.1}">{}</text>"#, x0 + len + 5.0, y + 13.5, esc(&x.label)).unwrap();
        for (k, line) in notes[i].iter().enumerate() {
            write!(s, r#"<text class="ax" x="{x0:.1}" y="{:.1}">{}</text>"#, y + 30.0 + k as f64 * 14.0, esc(line)).unwrap();
        }
        y += row_h[i];
    }
    s.push_str("</svg>");
    s
}

pub fn bars(bs: &[Bar], aria: &str) -> String {
    if bs.len() > 5 {
        return rows(bs, aria).replacen(r#"class="rows""#, r#"class="rows all""#, 1);
    }
    format!("{}{}", columns(bs, aria), rows(bs, aria))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_splits_notes_and_long_words() {
        // overlap seen 15/11 on /fleet/ with 6 harnesses: "Claude Code", "50 open · provisional"
        assert_eq!(wrap("50 open · provisional", 62.0, NOTE_PX), vec!["50 open", "provisional"]);
        assert_eq!(wrap("Claude Code", 60.0, NAME_PX), vec!["Claude", "Code"]);
        assert_eq!(wrap("Pi", 60.0, NAME_PX), vec!["Pi"]);
        assert_eq!(wrap("11.2 MB history *", 62.0, NOTE_PX), vec!["11.2 MB", "history *"]);
        assert!(wrap("", 60.0, NOTE_PX).is_empty());
    }
}
