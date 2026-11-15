# AgentRAMBench

How much memory and processor coding agents (Claude Code, Codex CLI, Pi, opencode, Crush, ...) use on one machine, and what happens when something breaks.

Site: https://agentrambench.com

This repository is the website: a Rust static-site generator that reads one JSON file per edition and writes HTML, CSS and SVG charts. Measurements come from the harness-bench runner; each edition JSON lists the runner commit and the raw run behind every number.

```
cargo run -- build                 # data/editions/*.json -> dist/
cargo run -- serve --port 8787     # build, then serve on 127.0.0.1
cargo run -- import FILE           # copy a harness-bench export into data/editions/
cargo test
```

Data: `data/editions/YYYY-MM.json` (also served at `/data/`). Add a harness: see https://agentrambench.com/submit/

Contact: contact@agentrambench.com

License: code MIT (LICENSE), data CC BY 4.0, Inter font SIL OFL 1.1 (assets/fonts/OFL-Inter.txt).
