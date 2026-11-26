mod chart;
mod data;
mod og;
mod site;

use std::path::{Path, PathBuf};

const USAGE: &str = "agentrambench build [--data DIR] [--out DIR]
agentrambench serve [--out DIR] [--port N]     build, then serve on 127.0.0.1
agentrambench import FILE [--data DIR]         copy a harness-bench export to DIR/YYYY-MM.json";

fn arg(args: &[String], k: &str, d: &str) -> String {
    args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned().unwrap_or(d.to_string())
}

fn build(data: &Path, out: &Path) -> Result<(), String> {
    let eds = data::load_all(data)?;
    if out.exists() {
        std::fs::remove_dir_all(out).map_err(|e| e.to_string())?;
    }
    let mut s = site::Site { eds: &eds, out: out.to_path_buf(), og: vec![], pages: vec![] };
    s.home();
    for a in &site::AXES {
        if a.id != "hostile" || eds[0].has_axis("hostile") {
            s.axis(a);
        }
    }
    for h in &eds[0].harnesses {
        s.harness(h);
    }
    s.harnesses();
    s.method();
    s.submit();
    s.data();
    s.editions();
    s.about();
    s.notfound();
    s.extras();
    og::render_all(out, &s.og);
    eprintln!("built {} pages, edition {} ({}), into {}", s.pages.len() + 1, eds[0].id, if eds[0].preliminary() { "preliminary" } else { "final" }, out.display());
    Ok(())
}

fn serve(out: &Path, port: u16) {
    let srv = tiny_http::Server::http(("127.0.0.1", port)).expect("bind");
    eprintln!("serving {} on http://127.0.0.1:{port}/", out.display());
    for req in srv.incoming_requests() {
        let url = req.url().split(['?', '#']).next().unwrap_or("/").to_string();
        let rel = url.trim_start_matches('/');
        let mut p = out.join(rel);
        if rel.contains("..") {
            p = out.join("404.html");
        }
        if p.is_dir() {
            if !url.ends_with('/') {
                let r = tiny_http::Response::empty(301).with_header(tiny_http::Header::from_bytes("Location", format!("{url}/")).unwrap());
                let _ = req.respond(r);
                continue;
            }
            p = p.join("index.html");
        }
        let (code, p) = if p.is_file() { (200, p) } else { (404, out.join("404.html")) };
        let ct = match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
            "html" => "text/html; charset=utf-8",
            "css" => "text/css",
            "svg" => "image/svg+xml",
            "png" => "image/png",
            "woff2" => "font/woff2",
            "json" => "application/json",
            "csv" => "text/csv; charset=utf-8",
            "xml" => "application/xml",
            _ => "text/plain; charset=utf-8",
        };
        let body = std::fs::read(&p).unwrap_or_default();
        let r = tiny_http::Response::from_data(body).with_status_code(code).with_header(tiny_http::Header::from_bytes("Content-Type", ct).unwrap())
            .with_header(tiny_http::Header::from_bytes("Cache-Control", if ct.starts_with("font/") { "public, max-age=31536000, immutable" } else { "no-cache" }).unwrap());
        let _ = req.respond(r);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data = PathBuf::from(arg(&args, "--data", root.join("data/editions").to_str().unwrap()));
    let out = PathBuf::from(arg(&args, "--out", root.join("dist").to_str().unwrap()));
    let r = match args.first().map(String::as_str) {
        Some("build") => build(&data, &out),
        Some("serve") => build(&data, &out).map(|_| serve(&out, arg(&args, "--port", "8787").parse().expect("port"))),
        Some("import") if args.len() > 1 => data::Edition::load(Path::new(&args[1])).and_then(|e| {
            let dst = data.join(format!("{}.json", e.edition.get(..7).unwrap_or(&e.edition)));
            std::fs::copy(&args[1], &dst).map_err(|x| x.to_string())?;
            eprintln!("{} → {}", args[1], dst.display());
            Ok(())
        }),
        _ => Err(USAGE.to_string()),
    };
    if let Err(e) = r {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
