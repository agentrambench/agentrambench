use crate::chart;
use crate::data::{Edition, Harness, Metric, Res, is_binary, slug};
use crate::og::Og;
use std::fmt::Write;
use std::path::{Path, PathBuf};

pub const BASE: &str = "https://agentrambench.com";
pub const REPO: &str = "https://github.com/agentrambench/agentrambench";
pub const BENCH: &str = "https://github.com/agentrambench/bench";
const CSS: &str = include_str!("../assets/site.css");

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub fn color(ed: &Edition, id: &str, _i: usize) -> String {
    let k = match id {
        "claude" => 0,
        "codex" => 1,
        "opencode" => 2,
        "pi" => 3,
        "crush" => 4,
        "goose" => 5,
        "gemini" => 6,
        _ => ed.harnesses.iter().position(|h| h.id == id).unwrap_or(0) % 7,
    };
    format!("var(--c{k})")
}

pub fn num(v: f64) -> String {
    let a = v.abs();
    let s = if a == 0.0 {
        "0".into()
    } else if v.fract() == 0.0 {
        format!("{v:.0}")
    } else if a >= 100.0 {
        format!("{v:.0}")
    } else if a >= 10.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.2}")
    };
    s.replace('-', "−")
}

fn unit(m: &Metric) -> &str {
    match m.unit.as_str() {
        "rate" | "" => "",
        u => u,
    }
}

pub fn val(m: &Metric, r: &Res) -> String {
    if m.unit == "rate" {
        if is_binary(r) {
            let k: f64 = r.values.iter().sum();
            return format!("{}/{}", k as u32, r.values.len());
        }
        return format!("{:.0}%", r.median * 100.0);
    }
    let u = unit(m);
    if u == "agents/GiB" {
        return format!("{:.1} {u}", r.median);
    }
    if m.worst_case() && r.max > r.median {
        return format!("up to {} {u}", num(r.max)).trim_end().to_string();
    }
    if u.is_empty() { num(r.median) } else { format!("{} {}", num(r.median), u) }
}

fn spread(m: &Metric, r: &Res) -> String {
    let runs = if r.runs == 1 { "1 run".to_string() } else { format!("{} runs", r.runs) };
    let mut n = if r.n_agents > 1 && m.id != "fleet_max_agents_reached" { format!(" · {} agents", r.n_agents) } else { String::new() };
    if let Some(x) = &r.stop_reason {
        n.push_str(&format!(" · stopped: {x}"));
    }
    if m.unit == "rate" && is_binary(r) {
        return format!("{runs}{n}");
    }
    if r.runs > 1 && r.min != r.max {
        if m.unit == "rate" {
            return format!("{:.0}–{:.0}% · {runs}{n}", r.min * 100.0, r.max * 100.0);
        }
        format!("{}–{} · {runs}{n}", num(r.min), num(r.max))
    } else {
        format!("{runs}{n}")
    }
}

fn compact(m: &Metric, r: &Res) -> String {
    if r.runs > 1 && r.min != r.max && !(m.unit == "rate" && is_binary(r)) {
        if m.unit == "rate" {
            return format!("{:.0}–{:.0}% · {}", r.min * 100.0, r.max * 100.0, runs(r.runs));
        }
        return format!("{}–{} · {}", num(r.min), num(r.max), runs(r.runs));
    }
    runs(r.runs)
}

fn runs(n: u32) -> String {
    if n == 1 { "1 run".into() } else { format!("{n} runs") }
}

fn pips(m: &Metric, r: &Res) -> String {
    let mut s = String::from(r#"<span class="pips" aria-hidden="true">"#);
    for v in &r.values {
        let c = if *v == 1.0 { if m.better == "higher" { "y" } else { "x" } } else { "" };
        write!(s, r#"<i class="{c}"></i>"#).unwrap();
    }
    s.push_str("</span>");
    s
}

pub fn plain(id: &str) -> Option<(&'static str, &'static str)> {
    Some(match id {
        "marathon_idle_anon_mib" => ("RAM after a long session", "One agent after 300 tool calls (about 1M tokens of history), then 5 min waiting. MiB."),
        "marathon_peak_anon_mib" => ("Peak RAM during a long session", "Highest memory while doing those 300 tool calls. MiB."),
        "after_work_anon_mib_per_agent" => ("RAM right after working", "Each agent just finished a task (20 file reads, a written report), 50 agents running. MiB."),
        "idle_300s_anon_mib_per_agent" => ("RAM 5 min after working", "Same agents, 5 minutes after their task. MiB."),
        "idle_3600s_anon_mib_per_agent" => ("RAM 1 h after work", "Each agent did a task, then waited 1 hour with the others open. Terminal and launched commands included. MiB."),
        "work_peak_anon_mib_per_agent" => ("Peak RAM while working, agent alone", "Highest memory of the agent itself during its task, without the commands it launched. MiB."),
        "harness_anon_mib_per_agent" => ("RAM of the agent alone", "Waiting agent, without its terminal or the commands it launched. MiB."),
        "terminal_anon_mib_per_agent" => ("RAM of the terminal", "Share of the terminal window per agent (tmux here; a desktop terminal adds about 30 MiB). MiB."),
        "tool_children_anon_mib" => ("RAM of launched commands", "Peak memory of the commands the agents ran, whole fleet. MiB."),
        "burst_cpu_s_per_agent" => ("CPU when all work at once", "Every agent gets the same task at the same second: CPU seconds per agent."),
        "burst_turn_s_p50" => ("Task time when all work at once", "Typical time to finish the task while every other agent works too. Seconds."),
        "burst_turn_inflation" => ("Slowdown when all work at once", "Task time with every agent busy, divided by task time alone. 1.0 = no slowdown."),
        "burst_turn_s_p90" => ("Slowest task times when all work at once", "Time the slowest 1 in 10 agents needed. Seconds."),
        "resumed_anon_mib_per_agent" => ("RAM when you reopen a session", "Close an agent mid-project, reopen it with its history: memory 30 s later. MiB."),
        "resumed_after_turn_anon_mib_per_agent" => ("RAM after reopening and one task", "Reopened session, then one small task done. MiB."),
        "resumed_idle_300s_anon_mib_per_agent" => ("RAM 5 min after reopening", "Reopened session, one task, then 5 min waiting. MiB."),
        "resumed_peak_anon_mib_per_agent" => ("Peak RAM after reopening", "Highest memory between reopening and the end of the first task. MiB."),
        "idle_anon_mib_per_agent" => ("RAM while waiting, every run of this size", "Middle value over every fleet run of this size: task, then waiting. MiB."),
        "idle_cpu_s_per_agent_hour" => ("CPU while waiting", "CPU seconds per hour while the agent waits for input."),
        "task_anon_mib" => ("RAM for one background task", "One agent doing a small coding task with no screen, peak MiB. Less than with its interface open: no screen, short conversation."),
        "task_rss_mib" => ("RAM incl. shared libraries", "Same task, also counting memory shared with other programs. Overstates what each extra agent costs."),
        "task_cpu_s" => ("CPU for one task", "CPU seconds one agent uses for the same small coding task."),
        "task_first_request_s" => ("Start-up time", "Seconds from launch to the agent's first call to the model."),
        "disk_state_mib" => ("Disk used", "Files the agents leave on disk (history, caches) for the whole fleet, in MiB."),
        "first_request_kib" => ("Hidden prompt size", "Instructions and tool list sent with every request; cached after the first, but they still fill the context. KiB."),
        "task_uncached_ktok" => ("Tokens at full price", "Estimated input tokens for the task that the provider's cache cannot discount, in thousands."),
        "task_requests" => ("Model calls", "Requests sent to the model for the same task, background ones included."),
        "idle_net_out_kib_per_agent_hour" => ("Upload while waiting", "Data an open, idle agent sends somewhere other than the model, per hour. KiB, at least."),
        "task_net_out_kib" => ("Upload during a task", "Data sent somewhere other than the model during the task. KiB, at least."),
        "task_canary_leaks" => ("Secrets sent to the model", "We plant fake passwords (a .env file, cloud keys, an environment variable) and count how many end up in what the agent sends the model."),
        "fleet_agents_per_gib" => ("Agents per GiB", "How many open, waiting agents fit in 1 GiB of RAM."),
        "fleet_max_agents_reached" => ("Largest fleet", "Most agents started together that all stayed alive. We tried up to 50."),
        "crash_kill9_recovered_alone" => ("Crash during a command: finished alone", "Killed mid-command, then restarted: does it finish the job without anyone typing?"),
        "crash_kill9_double_execution" => ("Runs a command twice after a crash", "Killed mid-command, then restarted: does it run it again? Risky for a deploy or a payment."),
        "herd_tasks_done_retry_after" => ("Tasks finished", "20 agents share one account limited to 30 requests per minute. Share of tasks that still finish."),
        "herd_tasks_done_no_retry_after" => ("Tasks finished, no retry hint", "Same, but the provider does not say when to try again."),
        "faults" => ("Survives API and network failures", "We make the AI provider fail on purpose. Tasks still finished, nobody helping."),
        "kill9" => ("Crash during a command", "Did the agent finish alone after a restart, and did it run the interrupted command twice?"),
        _ => return None,
    })
}

pub fn fault_plain(p: &str) -> Option<&'static str> {
    Some(match p {
        "429x3" => "Rate limited 3 times in a row",
        "429-60s" => "Rate limited for a minute",
        "529x3" => "Provider overloaded 3 times",
        "500x3" => "Server error 3 times",
        "401-60s" => "API key rejected for a minute",
        "reset-30s" => "Network down for 30 s",
        "hang-30s" => "Provider silent for 30 s",
        "truncate" => "Answer cut off halfway",
        "stall-60s" => "Answer freezes for a minute",
        "trickle" => "Answer arrives very slowly",
        "sse-error" => "Error in the middle of an answer",
        "malformed" => "Garbled answer",
        "tool-hang" => "A command never finishes",
        "tool-huge" => "A command prints a huge output",
        "loop-3min" => "Model never stops talking (3 min, not in totals)",
        _ => return None,
    })
}

pub fn short(id: &str, m: Option<&Metric>) -> String {
    if let Some((t, _)) = plain(id) {
        return t.to_string();
    }
    let crash = |c: &str| match c { "kill9" => "Crash during a command", "scope" => "Everything killed", "term" => "Terminal closed", "stream" => "Crash mid-answer", _ => "Crash" };
    if let Some(c) = id.strip_prefix("crash_").and_then(|x| x.strip_suffix("_recovered_alone")) {
        return format!("{}: finished alone", crash(c));
    }
    if let Some(c) = id.strip_prefix("crash_").and_then(|x| x.strip_suffix("_double_execution")) {
        return format!("{}: command run twice", crash(c));
    }
    if let Some(f) = id.strip_prefix("fault_").and_then(|x| x.strip_suffix("_done_alone")).and_then(fault_plain) {
        return format!("{f}: finished alone");
    }
    m.map(|m| m.label.clone()).unwrap_or_else(|| id.to_string())
}

pub struct Axis {
    pub id: &'static str,
    pub slug: &'static str,
    pub title: &'static str,
    pub blurb: &'static str,
    pub coming: bool,
}

pub const AXES: [Axis; 5] = [
    Axis { id: "fleet", slug: "fleet", title: "Fleet", blurb: "How many idle agents fit in a GiB of RAM, and the largest fleet that started and stayed alive.", coming: false },
    Axis { id: "resources", slug: "resources", title: "Resources", blurb: "Memory and CPU each agent uses, working and waiting. Start-up time. Disk.", coming: false },
    Axis { id: "reliability", slug: "failures", title: "Failures", blurb: "What happens when the agent crashes, its terminal closes, the network drops or the AI provider fails.", coming: false },
    Axis { id: "tokens", slug: "tokens", title: "Context weight", blurb: "What the agent sends the model before it reads your code. Sent with every request; cached after the first, but it still fills the context.", coming: false },
    Axis { id: "governance", slug: "governance", title: "Governance", blurb: "Data sent somewhere other than the AI model, and whether planted secrets leak. Full section next edition.", coming: true },
];

fn axis_of(slug_or_id: &str) -> &'static Axis {
    AXES.iter().find(|a| a.id == slug_or_id || a.slug == slug_or_id).unwrap_or(&AXES[1])
}

pub fn hslug(h: &Harness) -> String {
    slug(&h.name)
}

pub struct Site<'a> {
    pub eds: &'a [Edition],
    pub out: PathBuf,
    pub og: Vec<(String, Og)>,
    pub pages: Vec<String>,
}

const NAV: [(&str, &str); 6] = [
    ("/", "Leaderboard"),
    ("/harnesses/", "Harnesses"),
    ("/method/", "Method"),
    ("/data/", "Data"),
    ("/submit/", "Submit"),
    ("/about/", "About"),
];

impl<'a> Site<'a> {
    fn ed(&self) -> &'a Edition {
        &self.eds[0]
    }

    pub fn write(&self, rel: &str, body: &[u8]) {
        let p = self.out.join(rel.trim_start_matches('/'));
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, body).unwrap();
    }

    fn page(&mut self, path: &str, title: &str, desc: &str, og: Og, body: &str) {
        let ed = self.ed();
        let ogname = if path == "/" { "home".to_string() } else { path.trim_matches('/').trim_end_matches(".html").replace('/', "-") };
        let full_title = if path == "/" { format!("AgentRAMBench: {title}") } else { format!("{title} · AgentRAMBench") };
        let mut s = String::with_capacity(32 * 1024);
        write!(s, r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{t}</title><meta name="description" content="{d}"><link rel="canonical" href="{BASE}{path}"><meta property="og:type" content="website"><meta property="og:site_name" content="AgentRAMBench"><meta property="og:title" content="{t}"><meta property="og:description" content="{d}"><meta property="og:url" content="{BASE}{path}"><meta property="og:image" content="{BASE}/og/{ogname}.png"><meta property="og:image:width" content="1200"><meta property="og:image:height" content="630"><meta name="twitter:card" content="summary_large_image"><meta name="twitter:title" content="{t}"><meta name="twitter:description" content="{d}"><meta name="twitter:image" content="{BASE}/og/{ogname}.png">"##, t = esc(&full_title), d = esc(desc)).unwrap();
        if ed.preliminary() {
            s.push_str(r#"<meta name="robots" content="noindex">"#);
        } else if path == "/" || path == "/data/" || path.starts_with("/editions/") && path.len() > "/editions/".len() {
            s.push_str(&dataset_ld(ed));
        }
        s.push_str(r##"<meta name="theme-color" content="#ffffff"><script>(function(){var d=document.documentElement,t=null;try{t=localStorage.getItem("theme")}catch(e){}if(t)d.dataset.theme=t;else t=matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light";d.style.colorScheme=t=="light"?"light":"dark";document.querySelector("meta[name=theme-color]").content={light:"#ffffff",grey:"#26282c",dark:"#0b0d10"}[t]||"#ffffff";d.classList.add("js")})()</script>"##);
        s.push_str(r#"<link rel="icon" href="/favicon.svg" type="image/svg+xml"><link rel="preload" href="/fonts/Inter.woff2" as="font" type="font/woff2" crossorigin><style>"#);
        s.push_str(CSS);
        s.push_str("</style></head><body><a class=\"skip\" href=\"#main\">Skip to content</a>");
        if ed.preliminary() {
            s.push_str(r#"<aside class="notice" aria-label="Edition status"><b>Preliminary edition.</b> Numbers may still change before this edition is final.</aside>"#);
        }
        s.push_str(r##"<header class="top"><div class="wrap"><a class="logo" href="/" aria-label="AgentRAMBench home"><svg width="18" height="18" viewBox="0 0 32 32" aria-hidden="true"><path d="M5 26 L26 6" stroke="#5b8cff" stroke-width="4" stroke-linecap="round"/><path d="M3 12 H29" stroke="#f97066" stroke-width="3"/></svg>AgentRAMBench</a><nav class="main" aria-label="Main">"##);
        for (href, label) in NAV {
            let axis = AXES.iter().any(|a| path == format!("/{}/", a.slug));
            let cur = if href == path || (href != "/" && path.starts_with(href)) || (href == "/harnesses/" && path.starts_with("/harness/")) || (href == "/" && axis) { r#" aria-current="page""# } else { "" };
            write!(s, r#"<a href="{href}"{cur}>{label}</a>"#).unwrap();
        }
        s.push_str(r##"</nav><div class="theme" role="group" aria-label="Color theme"><button type="button" data-t="light" aria-label="White theme" title="White"><i style="background:#fff"></i></button><button type="button" data-t="grey" aria-label="Dark grey theme" title="Dark grey"><i style="background:#2f3136"></i></button><button type="button" data-t="dark" aria-label="Dark theme" title="Dark"><i style="background:#0b0d10"></i></button></div></div></header><main id="main">"##);
        s.push_str(body);
        write!(s, r##"</main><footer><div class="wrap"><p>AgentRAMBench is maintained by Édouard Vallet · <!--email_off--><a href="mailto:contact@agentrambench.com">contact@agentrambench.com</a><!--/email_off--> · <a href="/about/">About &amp; disclosure</a></p><p>Edition {e} · bench <code>{c}</code> · <a href="/data/">raw data</a> · <a href="/editions/">editions</a> · <a href="{REPO}">site source</a> · <a href="{BENCH}">runner</a></p></div></footer><script>(function(){{if(!window.fetch||!window.DOMParser||!history.pushState)return;var c={{}},seq=0,shown=location.pathname+location.search,st,ht,live=document.createElement("div");live.setAttribute("aria-live","assertive");live.setAttribute("aria-atomic","true");live.style.cssText="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap";document.body.append(live);history.scrollRestoration="manual";if(!history.state)history.replaceState({{y:scrollY}},"");addEventListener("scroll",function(){{clearTimeout(st);st=setTimeout(function(){{history.replaceState(Object.assign({{}},history.state,{{y:scrollY}}),"")}},100)}},{{passive:true}});function ok(a){{return a&&a.origin===location.origin&&!a.target&&!a.hasAttribute("download")&&!/\.[a-z0-9]+$/i.test(a.pathname)}}function get(u){{var e=c[u];if(e&&Date.now()-e.t<30000)return e.p;var p=fetch(u,{{credentials:"same-origin"}}).then(function(r){{if(!r.ok||!/text\/html/.test(r.headers.get("content-type")||""))throw Error(r.status);return r.text().then(function(h){{return{{h:h,u:r.url}}}})}});c[u]={{t:Date.now(),p:p}};p.catch(function(){{delete c[u]}});return p}}function jump(x,y){{var t=x.hash&&document.getElementById(decodeURIComponent(x.hash.slice(1)));requestAnimationFrame(function(){{if(t)t.scrollIntoView();else scrollTo(0,y)}})}}function go(u,push){{var my=++seq,x=new URL(u,location.href),k=x.pathname+x.search;if(!push&&k===shown){{jump(x,(history.state&&history.state.y)||0);return}}get(k).then(function(r){{delete c[k];if(my!==seq)return;var n=new DOMParser().parseFromString(r.h,"text/html"),m=n.querySelector("main"),nv=n.querySelector("nav.main"),ns=n.querySelector("head style"),os=document.querySelector("head style");if(!m||m.querySelector("script")||(ns&&ns.textContent)!==(os&&os.textContent))throw Error("full");var f=new URL(r.u);f.hash=x.hash;document.title=n.title;var a=document.querySelector("meta[name=description]"),b=n.querySelector("meta[name=description]");if(a&&b)a.content=b.content;var cn=document.querySelector("link[rel=canonical]"),nc=n.querySelector("link[rel=canonical]");if(cn&&nc)cn.href=nc.href;document.querySelector("main").replaceWith(document.adoptNode(m));var v=document.querySelector("nav.main");if(v&&nv)v.innerHTML=nv.innerHTML;if(push)history.pushState({{y:0}},"",f.href);shown=f.pathname+f.search;live.textContent=document.title;var h1=document.querySelector("main h1");if(h1){{h1.tabIndex=-1;h1.style.outline="none";h1.focus({{preventScroll:true}})}}jump(f,push?0:(history.state&&history.state.y)||0)}}).catch(function(){{if(my===seq)location.href=x.href}})}}document.addEventListener("click",function(e){{if(e.defaultPrevented||e.button||e.metaKey||e.ctrlKey||e.shiftKey||e.altKey)return;var a=e.target.closest&&e.target.closest("a[href]");if(!ok(a))return;if(a.pathname===location.pathname&&a.search===location.search&&a.hash)return;e.preventDefault();if(a.href===location.href)return;go(a.href,true)}});function pre(a){{if(ok(a)&&a.pathname!==location.pathname&&!(navigator.connection&&navigator.connection.saveData))get(a.pathname+a.search).catch(function(){{}})}}document.addEventListener("mouseover",function(e){{var a=e.target.closest&&e.target.closest("a[href]");clearTimeout(ht);if(a)ht=setTimeout(function(){{pre(a)}},65)}},{{passive:true}});document.addEventListener("touchstart",function(e){{pre(e.target.closest&&e.target.closest("a[href]"))}},{{passive:true}});addEventListener("popstate",function(){{go(location.href,false)}})}})()</script><script>(function(){{var d=document.documentElement,b=document.querySelectorAll(".theme button");function cur(){{return d.dataset.theme||(matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light")}}function sync(){{b.forEach(function(x){{x.setAttribute("aria-pressed",x.dataset.t===cur())}})}}b.forEach(function(x){{x.onclick=function(){{var t=x.dataset.t;if(t===cur())return;function go(){{d.dataset.theme=t;d.style.colorScheme=t=="light"?"light":"dark";document.querySelector("meta[name=theme-color]").content={{light:"#ffffff",grey:"#26282c",dark:"#0b0d10"}}[t];try{{localStorage.setItem("theme",t)}}catch(e){{}}sync()}}if(document.startViewTransition&&!matchMedia("(prefers-reduced-motion: reduce)").matches)document.startViewTransition(go);else go()}}}});sync()}})()</script></body></html>"##, e = esc(&ed.id), c = esc(&ed.bench_commit)).unwrap();
        let s = s.replace("kill -9", "kill&nbsp;-9");
        let rel = if path.ends_with('/') { format!("{path}index.html") } else { path.to_string() };
        self.write(&rel, s.as_bytes());
        self.og.push((ogname, og));
        if !path.ends_with(".html") {
            self.pages.push(path.to_string());
        }
    }

    fn bar_card(&self, ed: &Edition, mid: &str, title: &str, link: &str, short_sub: &str) -> Option<String> {
        let mut bs: Vec<chart::Bar>;
        let (sub, better);
        let mut src: Vec<String> = vec![];
        if mid == "faults" {
            let mut v: Vec<chart::Bar> = ed.measured().iter().filter_map(|h| ed.fault_tally(&h.id).map(|(k, n)| chart::Bar { name: h.name.clone(), color: color(ed, &h.id, 0), v: k / n as f64 * 100.0, lo: 0.0, hi: 0.0, label: format!("{}/{}", k as u32, n), note: String::new() })).collect();
            if v.is_empty() {
                return None;
            }
            v.sort_by(|a, b| b.v.partial_cmp(&a.v).unwrap());
            bs = v;
            sub = plain("faults").unwrap().1.to_string();
            better = "Higher is better";
        } else {
            let m = ed.metric(mid)?;
            let rk = ed.ranking(mid);
            if rk.is_empty() {
                return None;
            }
            let rate = m.unit == "rate";
            let k = if rate { 100.0 } else { 1.0 };
            let mut bs2: Vec<chart::Bar> = rk.iter().map(|(_, h, r)| chart::Bar { name: h.name.clone(), color: color(ed, &h.id, 0), v: m.key(r) * k, lo: if rate { 0.0 } else { r.min }, hi: if rate { 0.0 } else { r.max }, label: if rate { val(m, r) } else { val(m, r).trim_end_matches(unit(m)).trim().to_string() }, note: String::new() }).collect();
            if ed.common_steps(mid).is_some() {
                for (b, x) in bs2.iter_mut().zip(rk.iter()) {
                    if let Some(mb) = x.2.session_mb { b.note = format!("{mb:.1} MB history{}", if x.2.grow_note.is_some() { " *" } else { "" }); }
                }
            }
            let ns: Vec<u32> = rk.iter().map(|x| x.2.n_agents).collect();
            if ed.common_steps(mid).is_none() && ns.iter().any(|n| *n != ns[0]) {
                for (b, x) in bs2.iter_mut().zip(rk.iter()) {
                    b.note = format!("{} open", x.2.n_agents);
                }
            }
            bs = bs2;
            let base = plain(mid).map(|p| p.1.to_string()).unwrap_or_else(|| if m.label != title { m.label.clone() } else { m.note.clone().unwrap_or_default() });
            src = rk.iter().map(|x| x.2.source.clone()).collect::<Vec<_>>();
            better = if m.better == "higher" { "Higher is better" } else { "Lower is better" };
            sub = if !short_sub.is_empty() { short_sub.to_string() } else if unit(m).is_empty() || plain(mid).is_some() { base } else { format!("{base} · {}", unit(m)) };
        }
        for b in bs.iter_mut() {
            if ed.harnesses.iter().any(|h| h.name == b.name && h.provisional.is_some()) {
                b.note = if b.note.is_empty() { "provisional".into() } else { format!("{} · provisional", b.note) };
            }
        }
        let missing: Vec<String> = ed.measured().iter().filter(|h| !bs.iter().any(|b| b.name == h.name)).map(|h| h.name.clone()).collect();
        src.sort();
        src.dedup();
        let link = if link.is_empty() { format!(r#"<details class="raw"><summary>Raw runs</summary><span class="mono">{}</span></details>"#, esc(&src.join(", "))) } else { format!(r#"<a href="{link}">Details →</a>"#) };
        let mut s = String::new();
        write!(s, r#"<section class="card" id="{}"><h2><i style="background:var(--text)"></i>{}</h2><p class="sub">{} · {}</p>{}<div class="foot"><span>{}</span>{}</div></section>"#, esc(mid), esc(title), esc(&sub), better, chart::bars(&bs, &format!("{title}: {}", bs.iter().map(|b| format!("{} {}", b.name, b.label)).collect::<Vec<_>>().join(", "))), if missing.is_empty() { if let (true, Some(n)) = (mid.starts_with("burst_"), ed.common_n(mid)) { format!("{n} agents at once, same for all") } else if let Some(st) = ed.common_steps(mid) { format!("Same work for all: {st} file reads, then reopened; history size differs (under bars){}", if bs.iter().any(|b| b.note.ends_with(" *")) { ". * compacted itself and stopped before the end" } else { "" }) } else if bs.iter().any(|b| b.note.ends_with(" open")) { "Different numbers of agents: compare with care".to_string() } else if let (true, Some((n, Some(h)))) = (mid.starts_with("idle_") || mid.starts_with("fleet_agents") || mid == "disk_state_mib", ed.ranking(mid).first().map(|x| (x.2.n_agents, x.2.hold_s))) { format!("{n} agents open for {}, same for all", if h >= 3600.0 { format!("{:.0} h", h / 3600.0) } else { format!("{:.0} min", h / 60.0) }) } else { if mid == "faults" || ed.metric(mid).is_some_and(|m| m.unit == "rate") { "Count out of all runs".to_string() } else { "Middle of repeated runs; thin line: lowest–highest".to_string() } } } else { format!("Not run yet: {}", esc(&missing.join(", "))) }, link).unwrap();
        Some(s)
    }

    fn cell(&self, ed: &Edition, h: &Harness, mid: &str) -> String {
        if mid == "kill9" {
            let mut c = String::from(r#"<td class="n">"#);
            for (id, lab) in [("crash_kill9_recovered_alone", "alone"), ("crash_kill9_double_execution", "ran twice")] {
                if let (Some(m), Some(r)) = (ed.metric(id), ed.result(&h.id, id)) {
                    write!(c, r#"<span class="v" style="display:block">{} {} <span class="sp" style="display:inline">{lab}</span></span>"#, pips(m, r), esc(&val(m, r))).unwrap();
                }
            }
            c.push_str("</td>");
            return c;
        }
        if mid == "faults" {
            return match ed.fault_tally(&h.id) {
                Some((k, n)) => format!(r#"<td class="n"><span class="v">{}/{}</span><span class="sp"> of {} runs</span></td>"#, k as u32, n, n),
                None => r#"<td class="n pend" title="not run in this edition">pending</td>"#.into(),
            };
        }
        let (Some(m), Some(r)) = (ed.metric(mid), ed.result(&h.id, mid)) else {
            return r#"<td class="n pend" title="not run in this edition">pending</td>"#.into();
        };
        let mut extra = String::new();
        if mid == primary(ed) {
            let top = ed.ranking(mid).iter().map(|x| x.2.median).fold(1e-9, f64::max);
            write!(extra, r#"<span class="bar" title="bar length = value"><i style="width:{:.1}%;background:{}"></i></span>"#, (r.median / top).min(1.0) * 100.0, color(ed, &h.id, 0)).unwrap();
        }
        let p = if m.unit == "rate" && is_binary(r) { pips(m, r) + " " } else { String::new() };
        let cls = if m.unit == "rate" && is_binary(r) {
            let k: f64 = r.values.iter().sum();
            let bad = if m.better == "higher" { k == 0.0 } else { k > 0.0 };
            if bad { " cell-bad" } else { " cell-ok" }
        } else {
            ""
        };
        format!(r#"<td class="n"><span class="v{cls}">{p}{}</span><span class="sp">{}</span>{extra}</td>"#, esc(&val(m, r)), esc(&compact(m, r)))
    }

    fn scoreboard(&self, ed: &Edition, prefix: &str) -> String {
        let cols: Vec<(&str, &str)> = [
            (primary(ed), "resources"),
            ("faults", "failures"),
            ("kill9", "failures"),
            ("idle_cpu_s_per_agent_hour", "resources"),
            ("fleet_max_agents_reached", "fleet"),
        ]
        .into_iter()
        .filter(|(m, _)| if *m == "kill9" { ed.has_data("crash_kill9_recovered_alone") } else if *m == "faults" { ed.measured().iter().any(|h| ed.fault_tally(&h.id).is_some()) } else { ed.has_data(m) })
        .collect();
        let primary = primary(ed);
        let ranked = ed.ranking(primary);
        let mut s = String::new();
        s.push_str(r#"<div class="tbl" tabindex="0"><table><thead><tr><th scope="col">#</th><th scope="col" class="sticky">Harness</th>"#);
        for (m, ax) in &cols {
            let label = short(m, ed.metric(m));
            let better = ed.metric(m).map(|x| if x.better == "higher" { "↑" } else { "↓" }).unwrap_or(if *m == "kill9" { "" } else { "↑" });
            let mark = if *m == primary { " ●" } else { "" };
            write!(s, r#"<th scope="col" class="n"><a href="{prefix}/{ax}/#{}" title="{}">{}<br>{better}{mark}</a></th>"#, if *m == "kill9" { "crash" } else { m }, esc(&plain(m).map(|p| p.1.to_string()).or_else(|| ed.metric(m).map(|x| x.label.clone())).unwrap_or_default()), esc(&label)).unwrap();
        }
        s.push_str("</tr></thead><tbody>");
        let mut seen = Vec::new();
        for (rank, h, _) in &ranked {
            seen.push(h.id.clone());
            self.row(&mut s, ed, h, &rank.to_string(), &cols);
        }
        for h in ed.measured() {
            if !seen.contains(&h.id) {
                self.row(&mut s, ed, h, "–", &cols);
            }
        }
        for h in ed.harnesses.iter().filter(|h| h.status != "measured") {
            write!(s, r#"<tr class="nm"><td class="rank">–</td><th scope="row" class="sticky"><a href="/harness/{}/">{}</a></th><td colspan="{}">not measurable offline: {}</td></tr>"#, hslug(h), esc(&h.name), cols.len(), esc(h.reason.as_deref().unwrap_or("see harness page"))).unwrap();
        }
        let why = format!("Ranked by {} (●).", plain(primary).map(|p| p.0).unwrap_or(primary));
        write!(s, r#"</tbody></table></div><p class="mut" style="font-size:13px;margin-top:10px">{why} Big number: middle value of repeated runs. Small print: lowest–highest, n = number of runs. Squares: one per run; green = recovered without a human, red = a command was run a second time after a crash. ↑ higher is better, ↓ lower is better.</p>"#).unwrap();
        s
    }

    fn row(&self, s: &mut String, ed: &Edition, h: &Harness, rank: &str, cols: &[(&str, &str)]) {
        write!(s, r#"<tr><td class="rank">{rank}</td><th scope="row" class="sticky"><span class="sw" style="background:{}"></span><a href="/harness/{}/">{}</a>{}<span class="ver">{}</span></th>"#, color(ed, &h.id, 0), hslug(h), esc(&h.name), if h.provisional.is_some() { r#" <span class="prov" title="provisional: see harness page">provisional</span>"# } else { "" }, esc(&short_ver(h).rsplit(' ').next().map(|v| format!("v{}", v.trim_start_matches('v'))).unwrap_or_default())).unwrap();
        for (m, _) in cols {
            s.push_str(&self.cell(ed, h, m));
        }
        s.push_str("</tr>");
    }

    fn findings(&self, ed: &Edition) -> String {
        let mut f: Vec<(String, String)> = Vec::new();
        let (mid, lab) = if ed.ranking("idle_3600s_anon_mib_per_agent").len() >= 2 { ("idle_3600s_anon_mib_per_agent", "RAM 1 h after work") } else if ed.ranking("marathon_idle_anon_mib").len() >= 2 { ("marathon_idle_anon_mib", "RAM after a long session") } else { ("idle_anon_mib_per_agent", "RAM at fresh start") };
        let fd = ed.ranking(mid);
        if fd.len() >= 2 {
            let (b, w) = (fd.first().unwrap(), fd.last().unwrap());
            f.push((lab.into(), format!("{} {} MiB · {} {} MiB", b.1.name, num(b.2.median), w.1.name, num(w.2.median))));
        } else {
            let tr = ed.ranking("task_anon_mib");
            if tr.len() >= 2 {
                let (b, w) = (tr.first().unwrap(), tr.last().unwrap());
                f.push(("RAM for one task".into(), format!("{} {} MiB · {} {} MiB", b.1.name, num(b.2.median), w.1.name, num(w.2.median))));
            }
        }
        let k9: Vec<_> = ed.measured().into_iter().filter_map(|h| ed.result(&h.id, "crash_kill9_recovered_alone").map(|r| (h, r))).collect();
        if !k9.is_empty() {
            let alone = k9.iter().filter(|(_, r)| r.values.iter().sum::<f64>() > 0.0).count();
            let safe: Vec<&str> = ed.measured().into_iter().filter(|h| ed.result(&h.id, "crash_kill9_double_execution").is_some_and(|r| r.values.iter().sum::<f64>() == 0.0)).map(|h| h.name.as_str()).collect();
            if !safe.is_empty() && safe.len() < k9.len() {
                f.push(("Killed mid-command".into(), format!("Only {} did not run the command a second time after restart", safe.join(", "))));
            } else {
                f.push(("Killed mid-command".into(), format!("{alone} of {} harnesses finish the job on their own", k9.len())));
            }
        }
        let oom: Vec<(&Harness, &Res)> = ed.harnesses.iter().filter_map(|h| ed.results.iter().find(|r| r.harness == h.id && r.metric == "fleet_max_agents_reached" && r.stop_reason.as_deref().is_some_and(|x| x.contains("OOM"))).map(|r| (h, r))).collect();
        if let Some((_, r)) = oom.first() {
            let cap = r.stop_reason.as_deref().and_then(|x| x.split("at the ").nth(1)).map(|x| x.trim_end_matches(" cap").replace('G', " GiB")).unwrap_or_default();
            let n = r.stop_reason.as_deref().and_then(|x| x.strip_prefix("N=")).and_then(|x| x.split(':').next()).unwrap_or("");
            let who: Vec<String> = oom.iter().map(|(h, _)| format!("{} ({})", h.name, ed.result(&h.id, "fleet_max_agents_reached").map(|r| num(r.median)).unwrap_or_default())).collect();
            f.insert(1.min(f.len()), ("Ran out of memory".into(), format!("{n} agents overflowed the {cap} limit we set: {}. Largest fleet measured in brackets", who.join(", "))));
        }
        let mut s = String::from(r#"<aside class="news" aria-label="Key findings">"#);
        for (b, t) in f.iter().take(3) {
            write!(s, r#"<div><p class="eyebrow">{}</p><p>{}</p></div>"#, esc(b), esc(t)).unwrap();
        }
        s.push_str("</aside>");
        s
    }

    fn compare() -> &'static str {
        r#"<div class="cmp"><div><h3>Harness-Bench</h3><p class="eyebrow">Qihoo 360 · arXiv 2605.27922</p><p>Same models across harnesses on realistic sandboxed tasks. Unit: quality and API cost per task.</p></div><div><h3>HarnessTax</h3><p class="eyebrow">Arena</p><p>Model–harness pairs on SWE-bench Lite and Terminal-Bench. Unit: what the harness adds to tokens and dollars per task.</p></div><div class="us"><h3>AgentRAMBench</h3><p class="eyebrow">agentrambench.com</p><p>One to fifty agents on one machine, with a mock model. Unit: the machine: memory, CPU, waste while waiting, crashes, outages.</p></div></div><p class="mut" style="font-size:14px">Use them together: they tell you which harness solves the task; this tells you how many of them your machine can run, and what breaks when it does.</p>"#
    }

    fn rerun(&self, ed: &Edition) -> String {
        let r = ed.reproduce.clone().unwrap_or_default();
        let repo = r.repo.clone().unwrap_or(BENCH.to_string());
        let cmd = if r.command.is_empty() { "python3 hb.py quick".to_string() } else { r.command.clone() };
        let dir = repo.rsplit('/').next().unwrap_or("bench").to_string();
        format!(r#"<pre tabindex="0"><code>git clone {} && cd {}
{}   # ≈{} min, local mock model, $0, no API key</code></pre><p class="mut" style="font-size:14px">The quick profile runs the fixed task ×3, kill -9 and closed-terminal crashes, three faults (429 ×3, cut stream, network reset) and a 5-agent fleet held for 5 minutes, for every harness installed. Published editions use the full profile, every harness on the same machine. <a href="/method/">Method</a>.</p>"#, esc(&repo), esc(&dir), esc(&cmd), r.minutes.map(|m| format!("{m:.0}")).unwrap_or("30".into()))
    }

    fn pills(cur: &str) -> String {
        let mut s = String::from(r#"<nav class="pills" aria-label="Axes">"#);
        for (href, t) in [("/", "Overview"), ("/fleet/", "Fleet"), ("/resources/", "Resources"), ("/failures/", "Failures"), ("/tokens/", "Context weight"), ("/governance/", "Governance")] {
            let c = if href == cur { r#" aria-current="page""# } else { "" };
            write!(s, r#"<a href="{href}"{c}>{t}</a>"#).unwrap();
        }
        s.push_str("</nav>");
        s
    }

    pub fn home(&mut self) {
        let ed = self.ed();
        let n = ed.measured().len();
        let mut b = String::new();
        write!(b, r#"<div class="wrap"><section class="hero"><div><p class="eyebrow">Edition {} · {} harnesses · measured {}</p><h1 style="margin-top:12px">How much RAM does your AI agent harness eat?</h1><p class="lede">How much memory and CPU coding agents like Claude Code and Codex CLI use, and what happens when something breaks.</p></div>"#, esc(&ed.id), n, esc(&ed.measured_span())).unwrap();
        b.push_str(&self.findings(ed));
        b.push_str("</section>");
        b.push_str(&Self::pills("/"));
        b.push_str(r#"<div class="grid">"#);
        let cards = [
            ("idle_3600s_anon_mib_per_agent", "/resources/#idle_3600s_anon_mib_per_agent"),
            ("marathon_idle_anon_mib", "/resources/#marathon_idle_anon_mib"),
            ("marathon_peak_anon_mib", "/resources/#marathon_peak_anon_mib"),
            ("resumed_after_turn_anon_mib_per_agent", "/resources/#resumed_after_turn_anon_mib_per_agent"),
            ("idle_cpu_s_per_agent_hour", "/resources/#idle_cpu_s_per_agent_hour"),
            ("faults", "/failures/#faults"),
            ("crash_kill9_recovered_alone", "/failures/#crash"),
            ("crash_kill9_double_execution", "/failures/#crash"),
            ("first_request_kib", "/tokens/"),
            ("task_first_request_s", "/resources/#task_first_request_s"),
            ("task_anon_mib", "/resources/#task_anon_mib"),
        ];        let mut shown = 0;
        for (m, l) in cards {
            let t = &short(m, ed.metric(m));
            let rk = ed.ranking(m);
            let enough = if m == "faults" { ed.measured().iter().filter(|h| ed.fault_tally(&h.id).is_some()).count() >= 2 } else { rk.len() >= 2 && rk.first().map(|x| x.0) != rk.last().map(|x| x.0) };
            if !enough {
                continue;
            }
            if let Some(c) = self.bar_card(ed, m, t, l, "") {
                b.push_str(&c);
                shown += 1;
            }
            if shown == 6 {
                break;
            }
        }
        b.push_str("</div>");
        b.push_str(r#"<p class="mut" style="font-size:13px;margin-top:10px">Default settings, no extensions, tmux terminal, a mock model instead of a real one, one run per cell, one desktop machine. Real setups with extensions and a desktop terminal use more.</p>"#);
        b.push_str(r#"<h2 id="leaderboard">Leaderboard</h2><p class="sub">Every headline number per harness. Click a harness for all measurements.</p>"#);
        b.push_str(&self.scoreboard(ed, ""));
        b.push_str(r#"<h2>Per task, or per machine</h2>"#);
        b.push_str(Self::compare());
        b.push_str(r#"<h2>Rerun it in one command</h2>"#);
        b.push_str(&self.rerun(ed));
        b.push_str("</div>");
        let stat = ed.ranking(primary(ed)).first().map(|x| (format!("{} MiB", num(x.2.median)), format!("lowest · {}", x.1.name)));
        let og = Og { kicker: format!("Edition {} · {} harnesses", ed.id, n), title: "How much RAM does your AI agent harness eat?".into(), sub: format!("{}, same mock model", plain(primary(ed)).map(|p| p.0).unwrap_or("Private RAM")), stat, bars: og_bars(ed), prelim: ed.preliminary() };
        self.page("/", "How much RAM does your AI agent harness eat?", "What a fleet of coding agents costs the machine: RAM and CPU per agent, idle cost, fleets up to 50 agents, kill -9 and provider outages. Claude Code, Codex CLI, opencode, Pi.", og, &b);
    }

    fn crash_matrix(&self, ed: &Edition) -> String {
        let modes = [
            ("kill9", "Crash during a command", "The agent is force-killed (kill -9) during a 25 s command; the command keeps running."),
            ("scope", "Everything killed", "The agent and its command are killed together, as when the machine runs out of memory or a container stops."),
            ("term", "Terminal closed", "The terminal holding the agent is closed."),
            ("stream", "Crash mid-answer", "The agent is force-killed while the model's answer is arriving."),
        ];
        let modes: Vec<_> = modes.into_iter().filter(|m| ed.has_data(&format!("crash_{}_recovered_alone", m.0))).collect();
        if modes.is_empty() {
            return r#"<p class="pend">No crash run in this edition yet.</p>"#.into();
        }
        let mut s = String::from(r#"<div class="tbl" tabindex="0"><table><thead><tr><th scope="col" class="sticky">Harness</th>"#);
        for (_, t, d) in &modes {
            write!(s, r#"<th scope="col" title="{}">{}</th>"#, esc(d), esc(t)).unwrap();
        }
        s.push_str("</tr></thead><tbody>");
        for h in ed.measured() {
            write!(s, r#"<tr><th scope="row" class="sticky"><span class="sw" style="background:{}"></span><a href="/harness/{}/">{}</a></th>"#, color(ed, &h.id, 0), hslug(h), esc(&h.name)).unwrap();
            for (m, _, _) in &modes {
                s.push_str("<td>");
                for (k, lab) in [("recovered_alone", "alone"), ("double_execution", "ran twice")] {
                    let id = format!("crash_{m}_{k}");
                    if let (Some(mm), Some(r)) = (ed.metric(&id), ed.result(&h.id, &id)) {
                        write!(s, r#"<span class="mono" style="display:block;white-space:nowrap">{} {} <span class="mut">{lab}</span></span>"#, pips(mm, r), esc(&val(mm, r))).unwrap();
                    }
                }
                s.push_str("</td>");
            }
            s.push_str("</tr>");
        }
        s.push_str(r#"</tbody></table></div><p class="mut" style="font-size:13px;margin-top:10px">One square per run. Top line, green: the interrupted turn finished without anyone typing (“alone”). Bottom line, red: the interrupted command was run a second time after restart (“ran twice”).</p><dl class="kv">"#);
        for (_, t, d) in &modes {
            write!(s, "<dt>{}</dt><dd>{}</dd>", esc(t), esc(d)).unwrap();
        }
        s.push_str("</dl>");
        s
    }

    fn fault_matrix(&self, ed: &Edition) -> String {
        let profs: Vec<String> = ed.metrics.iter().filter_map(|m| m.id.strip_prefix("fault_").and_then(|x| x.strip_suffix("_done_alone")).map(String::from)).filter(|p| ed.has_data(&format!("fault_{p}_done_alone"))).collect();
        if profs.is_empty() {
            return r#"<p class="pend">No fault run on the ranked harnesses in this edition yet.</p>"#.into();
        }
        let hs = ed.measured();
        let mut s = String::from(r#"<div class="tbl" tabindex="0"><table><thead><tr><th scope="col" class="sticky">Injected fault</th>"#);
        for h in &hs {
            write!(s, r#"<th scope="col"><span class="sw" style="background:{}"></span>{}</th>"#, color(ed, &h.id, 0), esc(&h.name)).unwrap();
        }
        s.push_str("</tr></thead><tbody>");
        for p in &profs {
            let id = format!("fault_{p}_done_alone");
            let lab = ed.fault_labels.get(p).cloned().unwrap_or(p.clone());
            write!(s, r#"<tr><th scope="row" class="sticky">{}<span class="ver">{}</span></th>"#, esc(fault_plain(p).unwrap_or(&lab)), esc(&lab)).unwrap();
            for h in &hs {
                match (ed.metric(&id), ed.result(&h.id, &id)) {
                    (Some(m), Some(r)) => write!(s, r#"<td class="mono">{} {}</td>"#, pips(m, r), esc(&val(m, r))).unwrap(),
                    _ => s.push_str(r#"<td class="pend">–</td>"#),
                }
            }
            s.push_str("</tr>");
        }
        s.push_str(r#"<tr><th scope="row" class="sticky">Total handled alone</th>"#);
        for h in &hs {
            match ed.fault_tally(&h.id) {
                Some((k, n)) => write!(s, r#"<td class="mono"><b>{}/{}</b></td>"#, k as u32, n).unwrap(),
                None => s.push_str(r#"<td class="pend">–</td>"#),
            }
        }
        s.push_str(r#"</tr></tbody></table></div><p class="mut" style="font-size:13px;margin-top:10px">Faults start after the first model request. Green square: the task finished and its tests passed with nobody touching the agent.</p>"#);
        s
    }

    pub fn axis(&mut self, a: &'static Axis) {
        let ed = self.ed();
        let mut b = String::new();
        write!(b, r#"<div class="wrap"><section class="hero solo"><div><p class="eyebrow">Axis · edition {}</p><h1>{}</h1><p class="lede">{}</p></div></section>{}"#, esc(&ed.id), a.title, a.blurb, Self::pills(&format!("/{}/", a.slug))).unwrap();
        if a.id == "fleet" {
        }
        if a.id == "reliability" {
            b.push_str(r#"<h2 id="crash">Crash and resume</h2><p>We kill the agent while it runs a command, then restart it the way its docs say. Does it finish the job without anyone typing? Does it run the interrupted command a second time? Running twice is harmless for tests, dangerous for a deploy, a payment or a database change.</p>"#);
            b.push_str(&self.crash_matrix(ed));
            b.push_str(r#"<h2 id="faults">Provider and network faults</h2><p>Our mock model fails on purpose after its first answer: too many requests, rejected key, overload, network cuts, silence, broken answers. Each row is one kind of failure.</p>"#);
            b.push_str(&self.fault_matrix(ed));
        }
        let mut ms: Vec<&Metric> = ed.metrics.iter().filter(|m| m.axis == a.id && !(a.id == "reliability" && (m.id.starts_with("crash_") || m.id.starts_with("fault_")))).collect();
        ms.sort_by_key(|m| !m.headline);
        let cards: Vec<String> = ms.iter().filter_map(|m| self.bar_card(ed, &m.id, &short(&m.id, Some(m)), "", "")).collect();
        let pending: Vec<&str> = ms.iter().filter(|m| ed.ranking(&m.id).is_empty()).map(|m| m.label.as_str()).collect();
        if !cards.is_empty() {
            b.push_str(if a.id == "reliability" { r#"<h2 id="quota">Shared quota</h2>"# } else { "<h2>Measurements</h2>" });
            write!(b, r#"<div class="grid two">{}</div>"#, cards.join("")).unwrap();
        }
        if !pending.is_empty() {
            write!(b, r#"<p class="mut" style="font-size:14px;margin-top:16px">No run yet in this edition: {}.</p>"#, esc(&pending.join("; "))).unwrap();
        }
        if a.id == "reliability" {
            for m in ed.metrics.iter().filter(|m| m.id.starts_with("crash_") || m.id.starts_with("fault_")) {
                write!(b, r#"<span id="{}"></span>"#, esc(&m.id)).unwrap();
            }
        }
        if a.coming {
            b.push_str(r#"<h2>Coming next edition</h2><p>The full governance axis: what each harness does without asking, where it writes outside the workspace, which secrets it can read, and every byte it sends somewhere other than the model. Until then: upload while waiting and during a task for every harness; planted secrets only where noted.</p>"#);
        }
        b.push_str("</div>");
        let stat = match a.id {
            "fleet" => ed.ranking("fleet_agents_per_gib").first().map(|x| (format!("{:.1}", x.2.median), format!("agents per GiB · {}", x.1.name))),
            "resources" => ed.ranking("task_anon_mib").last().zip(ed.ranking("task_anon_mib").first()).map(|(w, b)| (format!("{:.1}×", w.2.median / b.2.median), "spread in task RAM".to_string())),
            "reliability" => {
                let k9 = ed.ranking("crash_kill9_recovered_alone");
                if k9.is_empty() { None } else { Some((format!("{}/{}", k9.iter().filter(|x| x.2.median > 0.0).count(), k9.len()), "harnesses resume alone after kill -9".to_string())) }
            }
            "tokens" => ed.ranking("first_request_kib").last().zip(ed.ranking("first_request_kib").first()).map(|(w, b)| (format!("{:.0}×", w.2.median / b.2.median), "spread in first request size".to_string())),
            _ => None,
        };
        let og = Og { kicker: format!("Edition {}", ed.id), title: a.title.to_string(), sub: a.blurb.to_string(), stat, bars: vec![], prelim: ed.preliminary() };
        self.page(&format!("/{}/", a.slug), a.title, a.blurb, og, &b);
    }

    pub fn harness(&mut self, h: &Harness) {
        let ed = self.ed();
        let sl = hslug(h);
        let mut b = String::new();
        write!(b, r#"<div class="wrap"><section class="hero solo"><div><p class="eyebrow"><a href="/harnesses/">Harnesses</a> · edition {}</p><h1><span class="sw" style="background:{};width:.3em;height:.3em;vertical-align:.3em;margin-right:.18em"></span>{}</h1>"#, esc(&ed.id), color(ed, &h.id, 0), esc(&h.name)).unwrap();
        if h.status != "measured" {
            write!(b, r#"<p class="lede">Not measurable: {}.</p><p>A harness is measurable when it can talk to a model endpoint we choose, so every harness sees the same deterministic mock. <a href="/submit/">What it takes to be ranked</a>.</p></div></section></div>"#, esc(h.reason.as_deref().unwrap_or("no reason recorded"))).unwrap();
            let og = Og { kicker: "Harness".into(), title: h.name.clone(), sub: format!("Not measurable: {}", h.reason.clone().unwrap_or_default()), stat: None, bars: vec![], prelim: ed.preliminary() };
            self.page(&format!("/harness/{sl}/"), &h.name, &format!("{} is not measurable offline on AgentRAMBench: {}.", h.name, h.reason.clone().unwrap_or_default().trim_end_matches('.')), og, &b);
            return;
        }
        let pm = primary(ed);
        let pl = plain(pm).map(|p| p.0).unwrap_or(pm);
        let fd = ed.result(&h.id, pm);
        let n = ed.measured().len();
        if let Some(p) = &h.provisional {
            write!(b, r#"<div class="notice" role="note"><b>Provisional.</b> {}</div>"#, esc(p)).unwrap();
        }
        b.push_str(r#"<div class="hgrid"><div>"#);
        let stat = if let Some(r) = fd {
            let rank = ed.ranking(pm).iter().find(|x| x.1.id == h.id).map(|x| x.0).unwrap_or(0);
            write!(b, r#"<div class="stat">{}</div><p class="eyebrow" style="margin-top:8px">MiB · {} · rank {rank} of {n}</p>"#, num(r.median), esc(pl)).unwrap();
            Some((num(r.median), format!("MiB · {pl}")))
        } else if let Some(r) = ed.result(&h.id, "task_anon_mib") {
            write!(b, r#"<div class="stat">{}</div><p class="eyebrow" style="margin-top:8px">MiB private RAM for one task · fleet run pending</p>"#, num(r.median)).unwrap();
            Some((num(r.median), "MiB private RAM, one task".to_string()))
        } else {
            None
        };
        write!(b, r#"</div><dl class="kv"><dt>version</dt><dd>{}</dd><dt>non-default setup</dt><dd>{}</dd><dt>measured</dt><dd>{} · bench <code>{}</code></dd><dt>model</dt><dd>{}</dd></dl></div></div></section>"#, esc(h.version.as_deref().unwrap_or("?")), esc(h.non_default_flags.as_deref().unwrap_or("none")), esc(&ed.measured_span()), esc(&ed.bench_commit), esc(&ed.llm)).unwrap();
        b.push_str(r#"<h2>Every measurement</h2><div class="tbl" tabindex="0"><table><thead><tr><th scope="col" class="sticky">Metric</th><th scope="col" class="n">Value</th><th scope="col" class="n">Rank</th><th scope="col" class="n">Best</th></tr></thead><tbody>"#);
        for a in &AXES {
            let ms: Vec<&Metric> = ed.metrics.iter().filter(|m| m.axis == a.id && ed.result(&h.id, &m.id).is_some()).collect();
            if ms.is_empty() {
                continue;
            }
            write!(b, r#"<tr><th scope="rowgroup" colspan="4" class="eyebrow" style="padding-top:18px"><a href="/{}/">{}</a></th></tr>"#, a.slug, a.title).unwrap();
            for m in ms {
                let r = ed.result(&h.id, &m.id).unwrap();
                let rk = ed.ranking(&m.id);
                let me = rk.iter().find(|x| x.1.id == h.id).map(|x| x.0).unwrap_or(0);
                let tied = rk.len() > 1 && rk.iter().all(|x| x.0 == 1);
                let lead = rk.iter().filter(|x| x.0 == 1).count();
                let best = if tied { "all tied".to_string() } else if lead > 1 { rk.first().map(|x| format!("{} · {lead} tied", val(m, x.2))).unwrap_or_default() } else { rk.first().map(|x| format!("{} · {}", val(m, x.2), x.1.name)).unwrap_or_default() };
                let me = if tied { "–".to_string() } else { format!("{me} of {}", rk.len()) };
                let p = if m.unit == "rate" && is_binary(r) { pips(m, r) + " " } else { String::new() };
                let anchor = if a.id == "reliability" && m.id.starts_with("crash_") { "crash".to_string() } else if m.id.starts_with("fault_") { "faults".to_string() } else { m.id.clone() };
                write!(b, r#"<tr><th scope="row" class="sticky" style="white-space:normal;font-weight:400"><a href="/{}/#{}">{}</a></th><td class="n"><span class="v">{p}{}</span><span class="sp">{}</span></td><td class="n">{me}</td><td class="n mut">{}</td></tr>"#, a.slug, esc(&anchor), esc(&short(&m.id, Some(m))), esc(&val(m, r)), esc(&spread(m, r)), esc(&best)).unwrap();
            }
        }
        b.push_str("</tbody></table></div>");
        let (bsvg, bw) = badge(ed, h);
        write!(b, r#"<h2>README badge</h2><div class="badge-prev"><img src="/badge/{sl}.svg" alt="AgentRAMBench badge for {n}" width="{bw}" height="20"></div><pre tabindex="0"><code>[![AgentRAMBench]({BASE}/badge/{sl}.svg)]({BASE}/harness/{sl}/)</code></pre><p class="mut" style="font-size:14px">The badge always shows the latest edition. It is a static SVG: no tracking, no script.</p></div>"#, n = esc(&h.name)).unwrap();
        let og = Og { kicker: format!("Edition {}", ed.id), title: h.name.clone(), sub: short_ver(h), stat, bars: vec![], prelim: ed.preliminary() };
        let desc = format!("{} on AgentRAMBench: RAM 1 h after work, CPU while waiting, long sessions, crash and fault recovery. Edition {}.", h.name, ed.id);
        self.page(&format!("/harness/{sl}/"), &h.name, &desc, og, &b);
        self.write(&format!("/badge/{sl}.svg"), bsvg.as_bytes());
    }

    pub fn harnesses(&mut self) {
        let ed = self.ed();
        let mut b = String::from(r#"<div class="wrap"><section class="hero solo"><div><p class="eyebrow">Harnesses</p><h1>Who is on the bench</h1><p class="lede">Every harness runs unmodified, at a pinned version, pointed at the same local mock model. Anything we had to change from the defaults is listed. Not on the list? <a href="/submit/">Send a pull request</a>.</p></div></section><div class="tbl" tabindex="0"><table><thead><tr><th scope="col" class="sticky">Harness</th><th scope="col">Version</th><th scope="col">Status</th><th scope="col">Non-default setup</th></tr></thead><tbody>"#);
        for h in &ed.harnesses {
            let st = if h.status == "measured" { "measured".to_string() } else { format!("not measurable offline: {}", h.reason.clone().unwrap_or_default()) };
            write!(b, r#"<tr><th scope="row" class="sticky"><span class="sw" style="background:{}"></span><a href="/harness/{}/">{}</a></th><td class="mono">{}</td><td>{}</td><td class="mono" style="font-size:12px;min-width:280px">{}</td></tr>"#, color(ed, &h.id, 0), hslug(h), esc(&h.name), esc(h.version.as_deref().unwrap_or("–")), esc(&st), esc(h.non_default_flags.as_deref().unwrap_or("–"))).unwrap();
        }
        b.push_str("</tbody></table></div></div>");
        let og = Og { kicker: String::new(), title: "Who is on the bench".into(), sub: ed.harnesses.iter().map(|h| h.name.clone()).collect::<Vec<_>>().join(" · "), stat: None, bars: og_bars(ed), prelim: ed.preliminary() };
        self.page("/harnesses/", "Harnesses", "Coding-agent harnesses measured by AgentRAMBench, with versions and every non-default setting.", og, &b);
    }

    pub fn method(&mut self) {
        let ed = self.ed();
        let m = &ed.machine;
        let mut b = String::new();
        write!(b, r#"<div class="wrap prose"><section class="hero solo"><div><p class="eyebrow">Method · edition {}</p><h1>How the RAM is measured</h1><p class="lede">Unmodified harness binaries on one Linux desktop, talking to a local mock model. The mock answers the same way every time, so differences come from the harness, not the model, and anyone can rerun it for $0.</p></div></section>"#, esc(&ed.id)).unwrap();
        write!(b, r#"<h2>The machine</h2><dl class="kv"><dt>cpu</dt><dd>{}, {} threads</dd><dt>ram</dt><dd>{:.1} GiB</dd><dt>kernel</dt><dd>Linux {}</dd><dt>isolation</dt><dd>each harness in its own systemd user scope; counters from its cgroup (memory.stat, memory.peak, cpu.stat) and per-process smaps_rollup</dd><dt>bench</dt><dd><code>{}</code></dd></dl>"#, esc(&m.cpu), m.cores, m.ram_gib, esc(&m.kernel), esc(&ed.bench_commit)).unwrap();
        b.push_str(r#"<h2>The model</h2><p>A local mock model speaks the Anthropic Messages, OpenAI Responses and Gemini generateContent APIs. It answers each scenario with a fixed script of tool calls and text, with a simulated time-to-first-token and token rate, so harness overhead is not hidden behind model latency and every harness gets the same work. A harness that cannot point at a custom model endpoint cannot be measured; it is listed with the reason.</p><p>What this rules out: model quality, prompt quality, task success on real repositories. For those, read Harness-Bench and HarnessTax (see below).</p>"#);
        b.push_str(r#"<h2>Scenarios</h2><ul>
<li><b>Fixed task.</b> One agent fixes a failing test in a small Python project: read, edit, run tests, answer. Peak private RAM, CPU, start-up time, request count and size.</li>
<li><b>Fleet after work.</b> 5, 10, 25 or 50 interactive agents in one tmux server. Each does a task (20 file reads and a written report), then waits 1 hour. Memory read right after the task and at 5, 15, 30 and 60 min, split into agent, terminal and launched commands. Processor use and network traffic while waiting, disk state. 50 is the most we tried; opencode overflowed our 20 GiB limit at 50 and was measured at 25.</li>
<li><b>All at once.</b> After the wait, every agent gets the same 3-step task at the same second: CPU per agent and task time, compared with one agent alone.</li>
<li><b>Long session.</b> One agent makes 300 tool calls (about 1 million tokens of history, each harness compacting by its default), then waits 5 min. Memory during and after.</li>
<li><b>Reopened session.</b> The same work (5, 60 or 300 file reads) is done without a screen, the agent is killed, then the session is reopened in the normal interactive interface. Memory after reopening, after one task and 5 min later. The first request after reopening is checked to contain the history.</li>
<li><b>Fresh start.</b> 5 agents, one short task (about 320 KB of tool output), then 5 min waiting. The lightest case; kept for comparison.</li>
<li><b>Crash.</b> The harness is killed during a 25-second command (kill -9 on the harness only, on its whole process tree, by closing its terminal, or during the final answer), then resumed the documented way.</li>
<li><b>Faults.</b> After the first request the mock injects 429s, 401s, 5xx, connection resets, a silent server, cut, stalled, trickling or malformed streams; plus a command that never returns, a command with 50 MB of output, and a model that never ends its turn.</li>
<li><b>Shared quota.</b> Many agents against one rate-limited key, with and without Retry-After: how many tasks finish.</li></ul>"#);
        b.push_str(r#"<h2>How numbers are reported</h2><ul><li>When a cell has several runs, the big number is the middle value (median), with lowest–highest and every value in the JSON. Many fleet cells have one run; the leaderboard says n=1. Rates show one square per run.</li><li>Memory is private (anonymous) memory from the cgroup: what an extra agent really adds. RSS counts shared libraries once per process and is shown only as a secondary metric.</li><li>Headline: private RAM per agent 1 hour after work, terminal and launched commands included, at the largest fleet each harness ran (50; opencode 25). Per-hour figures come from the 1-hour wait, never scaled up from shorter windows. Not a weighted score. There is no overall score.</li><li>Network bytes are a lower bound (socket sampling every 2 s). Token counts are request bytes / 4, an estimate, not a tokenizer.</li><li>Ranks are per metric. Ties share a rank.</li></ul>"#);
        b.push_str(r#"<h2>Every metric in this edition</h2><div class="tbl" tabindex="0"><table><thead><tr><th scope="col" class="sticky">Metric</th><th scope="col">Axis</th><th scope="col">Better</th><th scope="col">How</th></tr></thead><tbody>"#);
        for mm in &ed.metrics {
            write!(b, r#"<tr><th scope="row" class="sticky" style="white-space:normal;font-weight:400">{}<span class="ver">{}</span></th><td>{}</td><td>{}</td><td style="font-size:13px">{}</td></tr>"#, esc(&short(&mm.id, Some(mm))), esc(&mm.id), axis_of(&mm.axis).title, esc(&mm.better), esc(&[plain(&mm.id).map(|p| p.1.to_string()).unwrap_or_else(|| mm.label.clone()), mm.note.clone().unwrap_or_default()].iter().filter(|x| !x.is_empty()).cloned().collect::<Vec<_>>().join(" "))).unwrap();
        }
        b.push_str("</tbody></table></div>");
        b.push_str(r#"<h2>Related work</h2>"#);
        b.push_str(Self::compare());
        b.push_str(r#"<h2>Known limits</h2><ul><li>One machine per edition. Absolute numbers depend on the CPU and kernel.</li><li>A mock model sends fixed answers. A harness that behaves differently with a real model on long sessions will not show it here.</li><li>Harnesses release often; each edition pins versions and states them.</li></ul><h2>Rerun</h2>"#);
        b.push_str(&self.rerun(ed));
        b.push_str("</div>");
        let og = Og { kicker: "Method".into(), title: "How the RAM is measured".into(), sub: "Real harness binaries, a deterministic local mock model, cgroup counters. $0 to rerun.".into(), stat: None, bars: vec![], prelim: ed.preliminary() };
        self.page("/method/", "Method", "How AgentRAMBench measures coding-agent harnesses: machine, mock model, scenarios, metric definitions, known limits.", og, &b);
    }

    pub fn submit(&mut self) {
        let ed = self.ed();
        let mut b = String::from(r#"<div class="wrap prose"><section class="hero solo"><div><p class="eyebrow">Submit</p><h1>Put your harness on the bench</h1><p class="lede">One pull request. If your harness can talk to a model endpoint we choose, it gets measured in the next edition, under the same rules as everyone else.</p></div></section>"#);
        write!(b, r#"<h2>Requirements</h2><ul><li>Runs on Linux, installs at a pinned version (npm, pip, cargo, a release binary).</li><li>Accepts a custom endpoint and API key for the Anthropic Messages, OpenAI Responses or Gemini API. Without it, the harness is listed as <i>not measurable offline</i>, with the reason.</li><li>Runs a prompt headless, and can run interactively in a terminal (for the idle fleet and closed-terminal tests).</li><li>Documents how to resume an interrupted session, if it can.</li></ul><h2>The pull request</h2><ol><li>Add an adapter to <code>harnesses.py</code> in <a href="{BENCH}">the AgentRAMBench runner</a>: install command, version, how to set the endpoint, headless and interactive commands, resume command.</li><li>List every non-default flag you need. It is printed on your harness page.</li><li>Run <code>python3 hb.py quick --harness yours</code>, then <code>python3 export.py</code>, and attach <code>results/public/agentrambench.json</code> plus a tarball of the <code>results/raw/</code> run folders it was built from.</li></ol><h2>What happens next</h2><ul><li>The maintainer reruns your adapter with the full profile, on the same machine as every other harness, for the next edition. Results are published whether they flatter the harness or not.</li><li>Vendors can contest a number by sending a reproducible run to <!--email_off--><a href="mailto:contact@agentrambench.com">contact@agentrambench.com</a><!--/email_off-->. Corrections are listed on the edition page.</li><li>No harness gets a different scenario, a longer timeout or a private rerun.</li></ul></div>"#).unwrap();
        let og = Og { kicker: "Submit".into(), title: "Put your harness on the bench".into(), sub: "One pull request. Same rules for everyone.".into(), stat: None, bars: vec![], prelim: ed.preliminary() };
        self.page("/submit/", "Submit your harness", "Add a coding-agent harness to AgentRAMBench with one pull request: requirements, adapter, quick run.", og, &b);
    }

    pub fn data(&mut self) {
        let ed = self.ed();
        let mut b = String::from(r#"<div class="wrap prose"><section class="hero solo"><div><p class="eyebrow">Data</p><h1>Every number, raw</h1><p class="lede">Each edition is one JSON file in the <code>agentrambench/1</code> schema, plus a flat CSV of the same results. Every value carries its run count, min, max, all individual runs, and the raw run directory it came from.</p></div></section><div class="tbl" tabindex="0"><table><thead><tr><th scope="col">Edition</th><th scope="col">Status</th><th scope="col">Harnesses</th><th scope="col">Results</th><th scope="col">Files</th></tr></thead><tbody>"#);
        for e in self.eds {
            write!(b, r#"<tr><td class="mono"><a href="/editions/{id}/">{id}</a></td><td>{}</td><td>{}</td><td class="mono">{}</td><td class="mono"><a href="/data/agentrambench-{id}.json" download>json</a> · <a href="/data/agentrambench-{id}.csv" download>csv</a>{raw}</td></tr>"#, if e.preliminary() { "preliminary" } else { "final" }, format!("{} measured, {} not measurable", e.measured().len(), e.harnesses.len() - e.measured().len()), e.results.len(), id = esc(&e.id), raw = e.raw_archive.as_deref().map(|u| format!(r#" · <a href="{}">raw runs</a>"#, esc(u))).unwrap_or_default()).unwrap();
        }
        b.push_str(r#"</tbody></table></div><h2>Schema</h2><pre tabindex="0"><code>{
  "schema": "agentrambench/1",
  "edition": "YYYY-MM-DD", "status": "preliminary" | "final",
  "bench_commit", "llm", "machine": {cpu, cores, ram_gib, kernel},
  "harnesses": [{id, name, version, status: "measured" | "not_measurable",
                 non_default_flags | reason}],
  "metrics":   [{id, axis, label, unit, better: "lower" | "higher", headline, note}],
  "results":   [{harness, metric, n_agents, runs, median, min, max, values[], source}]
}</code></pre><p>Fields are only ever added within <code>/1</code>. Rates are 0..1. Read <code>metrics[]</code> dynamically: new metrics appear without a schema bump.</p><p>Cite as <i>AgentRAMBench</i> with the edition, for example <i>AgentRAMBench 2026-11</i>.</p></div>"#);
        let og = Og { kicker: "Data".into(), title: "Every number, raw".into(), sub: "JSON and CSV per edition. Every run, every value.".into(), stat: None, bars: vec![], prelim: ed.preliminary() };
        self.page("/data/", "Raw data", "Download AgentRAMBench results: one JSON and CSV per edition, every run and value.", og, &b);
        for e in self.eds {
            self.write(&format!("/data/agentrambench-{}.json", e.id), e.raw.as_bytes());
            self.write(&format!("/data/agentrambench-{}.csv", e.id), csv(e).as_bytes());
        }
    }

    pub fn editions(&mut self) {
        let ed = self.ed();
        let mut b = String::from(r#"<div class="wrap"><section class="hero solo"><div><p class="eyebrow">Editions</p><h1>Dated and frozen</h1><p class="lede">A new edition when harnesses or the method change. Once final, an edition never changes; corrections ship in the next one with a note.</p></div></section><div class="tbl" tabindex="0"><table><thead><tr><th scope="col">Edition</th><th scope="col">Measured</th><th scope="col">Bench</th><th scope="col">Status</th><th scope="col">Harnesses</th></tr></thead><tbody>"#);
        for e in self.eds {
            write!(b, r#"<tr><td class="mono"><a href="/editions/{id}/">{id}</a></td><td class="mono">{}</td><td class="mono">{}</td><td>{}</td><td>{}</td></tr>"#, esc(&e.edition), esc(&e.bench_commit), if e.preliminary() { "preliminary" } else { "final" }, esc(&e.measured().iter().map(|h| h.name.as_str()).collect::<Vec<_>>().join(", ")), id = esc(&e.id)).unwrap();
        }
        b.push_str("</tbody></table></div></div>");
        let og = Og { kicker: String::new(), title: "Dated and frozen".into(), sub: "Every edition with its data.".into(), stat: None, bars: vec![], prelim: ed.preliminary() };
        self.page("/editions/", "Editions", "All AgentRAMBench editions, dated and frozen.", og, &b);
        for i in 0..self.eds.len() {
            let e = &self.eds[i];
            let mut b = String::new();
            write!(b, r#"<div class="wrap"><section class="hero solo"><div><p class="eyebrow"><a href="/editions/">Editions</a> · {}</p><h1>Edition {}</h1><p class="lede">Measured {} on {} with bench <code>{}</code>. <a href="/data/agentrambench-{}.json">JSON</a>.</p></div></section>"#, if e.preliminary() { "preliminary" } else { "final" }, esc(&e.id), esc(&e.measured_span()), esc(&e.machine.cpu), esc(&e.bench_commit), esc(&e.id)).unwrap();
            b.push_str("<h2>Leaderboard</h2>");
            b.push_str(&self.scoreboard(e, ""));
            b.push_str("</div>");
            let og = Og { kicker: String::new(), title: format!("Edition {}", e.id), sub: "Fleet cost of coding-agent harnesses.".into(), stat: None, bars: og_bars(e), prelim: e.preliminary() };
            self.page(&format!("/editions/{}/", e.id), &format!("Edition {}", e.id), &format!("Edition {}: RAM, CPU, crash and outage results for {}, measured {}.", e.id, e.measured().iter().map(|h| h.name.as_str()).collect::<Vec<_>>().join(", "), e.measured_span()), og, &b);
        }
    }

    pub fn about(&mut self) {
        let ed = self.ed();
        let b = r#"<div class="wrap prose"><section class="hero solo"><div><p class="eyebrow">About</p><h1>About AgentRAMBench</h1><p class="lede">AgentRAMBench is maintained by Édouard Vallet. It measures what coding-agent harnesses cost the machine they run on, and publishes every number with the raw data and the code to rerun it.</p></div></section><h2>Why this benchmark</h2><p>Developers run many coding agents on one machine. Public benchmarks score task quality and cost per task; a few blog posts measure the RAM of one session. AgentRAMBench measures fleets: RAM per agent after an hour, CPU while waiting, what breaks on a crash or an outage, in dated editions with raw data.</p><h2>Rules</h2><ul><li>Same scenario, same timeout, same mock model for every harness.</li><li>Every number names its raw run in the published data; every run can be repeated with one command.</li><li>One machine for all harnesses in an edition, described on the edition page. Today: the maintainer's desktop (Intel i7-10700KF, 16 threads, 64 GB RAM (62.7 GiB usable), Linux), not a dedicated server.</li><li>No vendor sees results before publication. No vendor pays for a listing or a rerun.</li><li>A vendor who disputes a number sends a run we can reproduce; confirmed corrections are listed on the edition page.</li></ul><h2>Maintainer</h2><p>Édouard Vallet.</p><p class="disclosure">Disclosure: the maintainer also builds a fleet runtime for coding agents. It is not ranked. If it ever is, it runs under the same rules and must be reproducible by anyone.</p><h2>Contributing</h2><p>Harnesses, scenarios and corrections arrive as pull requests. <a href="/submit/">How to submit a harness</a>.</p><h2>Contact</h2><p><!--email_off--><a href="mailto:contact@agentrambench.com">contact@agentrambench.com</a><!--/email_off--></p></div>"#;
        let og = Og { kicker: String::new(), title: "About AgentRAMBench".into(), sub: "Maintained by Édouard Vallet. Raw data and one-command reruns for every number.".into(), stat: None, bars: vec![], prelim: ed.preliminary() };
        self.page("/about/", "About", "About AgentRAMBench: maintainer, rules and disclosure.", og, b);
    }

    pub fn notfound(&mut self) {
        let ed = self.ed();
        let b = r#"<div class="wrap"><section class="hero solo"><div><p class="eyebrow">404</p><h1>Nothing on this bench</h1><p class="lede">This page does not exist. The <a href="/">leaderboard</a> and the <a href="/data/">raw data</a> do.</p></div></section></div>"#;
        let og = Og { kicker: String::new(), title: "Not found".into(), sub: String::new(), stat: None, bars: vec![], prelim: ed.preliminary() };
        self.page("/404.html", "Not found", "Page not found.", og, b);
    }

    pub fn extras(&self) {
        let ed = self.ed();
        let mut sm = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);
        for p in &self.pages {
            write!(sm, "<url><loc>{BASE}{p}</loc></url>").unwrap();
        }
        sm.push_str("</urlset>");
        self.write("/sitemap.xml", sm.as_bytes());
        let robots = if ed.preliminary() { "User-agent: *\nDisallow: /\n".to_string() } else { format!("User-agent: *\nAllow: /\nSitemap: {BASE}/sitemap.xml\n") };
        self.write("/robots.txt", robots.as_bytes());
        self.write("/llms.txt", llms_txt(ed).as_bytes());
        self.write("/_headers", b"/fonts/*\n  Cache-Control: public, max-age=31536000, immutable\n/og/*\n  Cache-Control: public, max-age=86400\n/badge/*\n  Cache-Control: public, max-age=3600\n  Access-Control-Allow-Origin: *\n/data/*\n  Access-Control-Allow-Origin: *\n/*\n  X-Content-Type-Options: nosniff\n  Strict-Transport-Security: max-age=31536000; includeSubDomains\n  Referrer-Policy: strict-origin-when-cross-origin\n");
        self.write("/favicon.svg", br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="8" fill="#0d1117"/><path d="M7 25 L25 7" stroke="#5b8cff" stroke-width="3.5" stroke-linecap="round"/><path d="M4 12 H28" stroke="#f97066" stroke-width="2.5"/></svg>"##);
        for f in ["Inter"] {
            let src = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("assets/fonts/{f}.woff2"));
            self.write(&format!("/fonts/{f}.woff2"), &std::fs::read(src).unwrap());
        }
    }
}

fn primary(ed: &Edition) -> &'static str {
    let n = ed.measured().len();
    ["idle_3600s_anon_mib_per_agent", "idle_anon_mib_per_agent", "task_anon_mib"].into_iter().find(|m| ed.ranking(m).len() * 2 > n).unwrap_or("task_anon_mib")
}

fn short_ver(h: &Harness) -> String {
    h.version.as_deref().unwrap_or("").split(['(', ',']).next().unwrap_or("").trim().to_string()
}

fn og_bars(ed: &Edition) -> Vec<(String, String, f64, String)> {
    let mid = primary(ed);
    let Some(m) = ed.metric(mid) else { return vec![] };
    let rk = ed.ranking(mid);
    let mx = rk.iter().map(|x| x.2.median).fold(0.0, f64::max).max(1e-9);
    let hex = |id: &str| match id { "claude" => "#d97757", "codex" => "#16181d", "opencode" => "#2563eb", "pi" => "#7c3aed", _ => "#db2777" }.to_string();
    rk.iter().map(|x| (x.1.name.clone(), val(m, x.2), x.2.median / mx, hex(&x.1.id))).collect()
}

pub fn badge(ed: &Edition, h: &Harness) -> (String, f64) {
    let left = "AgentRAMBench";
    let right = match ed.result(&h.id, "idle_3600s_anon_mib_per_agent") {
        Some(r) => format!("{} MiB per agent after work", num(r.median)),
        None => match ed.result(&h.id, "task_anon_mib") {
            Some(r) => format!("{} MiB per task", num(r.median)),
            None => "measured".into(),
        },
    };
    let wl = left.len() as f64 * 6.6 + 14.0;
    let wr = right.len() as f64 * 6.6 + 14.0;
    let w = wl + wr;
    (format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w:.0}" height="20" role="img" aria-label="{left}: {right}"><title>{left}: {right} ({})</title><clipPath id="r"><rect width="{w:.0}" height="20" rx="3"/></clipPath><g clip-path="url(#r)"><rect width="{wl:.0}" height="20" fill="#10161B"/><rect x="{wl:.0}" width="{wr:.0}" height="20" fill="#C41A2E"/></g><g fill="#fff" font-family="Verdana,DejaVu Sans,sans-serif" font-size="11" text-anchor="middle"><text x="{:.1}" y="14">{left}</text><text x="{:.1}" y="14">{}</text></g></svg>"##, esc(&ed.id), wl / 2.0, wl + wr / 2.0, esc(&right)), w.round())
}

pub fn csv(ed: &Edition) -> String {
    let mut s = String::from("edition,harness,metric,axis,unit,better,n_agents,runs,median,min,max,values,source\n");
    for r in &ed.results {
        let m = ed.metric(&r.metric);
        let vals: Vec<String> = r.values.iter().map(|v| v.to_string()).collect();
        writeln!(s, "{},{},{},{},{},{},{},{},{},{},{},\"{}\",{}", ed.id, r.harness, r.metric, m.map(|m| m.axis.as_str()).unwrap_or(""), m.map(|m| m.unit.as_str()).unwrap_or(""), m.map(|m| m.better.as_str()).unwrap_or(""), r.n_agents, r.runs, r.median, r.min, r.max, vals.join(" "), r.source).unwrap();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::tests::{ed, m, r};

    fn site() -> Site<'static> {
        Site { eds: &[], out: PathBuf::new(), og: vec![], pages: vec![] }
    }

    #[test]
    fn numbers_as_displayed() {
        assert_eq!(num(28.448), "28.4");
        assert_eq!(num(526.224), "526");
        assert_eq!(num(0.35), "0.35");
        assert_eq!(num(25.0), "25");
        assert_eq!(num(-1.5), "−1.50");
    }

    #[test]
    fn headline_metric_needs_more_than_half_the_harnesses() {
        let a = "idle_3600s_anon_mib_per_agent";
        let ms = [m(a, "lower"), m("task_anon_mib", "lower")].join(",");
        let two = [r("codex", a, 50, Some(3600.0), 28.4, ""), r("pi", a, 50, Some(3600.0), 94.8, "")];
        assert_eq!(primary(&ed(&ms, &two.join(","))), "task_anon_mib");
        let three = [&two[..], &[r("claude", a, 50, Some(3600.0), 145.0, "")]].concat();
        assert_eq!(primary(&ed(&ms, &three.join(","))), a);
    }

    #[test]
    fn findings_from_07_11_data() {
        let a = "idle_3600s_anon_mib_per_agent";
        let ms = [m(a, "lower"), m("fleet_max_agents_reached", "higher"), m("crash_kill9_recovered_alone", "higher"), m("crash_kill9_double_execution", "lower")].join(",");
        let rows = |codex_twice: &str| {
            let mut rs = vec![
                r("codex", a, 50, Some(3600.0), 28.448, ""),
                r("pi", a, 50, Some(3600.0), 94.8, ""),
                r("opencode", a, 25, Some(3600.0), 526.224, ""),
                r("opencode", "fleet_max_agents_reached", 25, None, 25.0, r#","stop_reason":"N=50: OOM-killed at the 20G cap""#),
                r("codex", "fleet_max_agents_reached", 50, None, 50.0, ""),
            ];
            for (h, twice) in [("claude", "[1,1,1]"), ("codex", codex_twice), ("opencode", "[1,1,1]"), ("pi", "[1,1,1]")] {
                rs.push(r(h, "crash_kill9_recovered_alone", 1, None, 0.0, r#","values":[0,0,0]"#));
                rs.push(r(h, "crash_kill9_double_execution", 1, None, 1.0, &format!(r#","values":{twice}"#)));
            }
            rs.join(",")
        };
        let f = site().findings(&ed(&ms, &rows("[0,0,0]")));
        assert!(f.contains("Codex CLI 28.4 MiB · opencode 526 MiB"), "{f}");
        assert!(f.contains("50 agents overflowed the 20 GiB limit we set: opencode (25)"), "{f}");
        assert!(f.contains("Only Codex CLI did not run the command a second time after restart"), "{f}");
        assert!(f.find("RAM 1 h after work") < f.find("Ran out of memory") && f.find("Ran out of memory") < f.find("Killed mid-command"), "{f}");
        let f = site().findings(&ed(&ms, &rows("[1,1,1]")));
        assert!(f.contains("0 of 4 harnesses finish the job on their own"), "{f}");
    }
}

fn json_str(x: &str) -> String {
    serde_json::to_string(x).unwrap().replace("</", "<\\/")
}

fn dataset_ld(ed: &Edition) -> String {
    let names: Vec<&str> = ed.measured().iter().map(|h| h.name.as_str()).collect();
    let date = ed.frozen.clone().unwrap_or_else(|| ed.edition.clone());
    format!(
        r#"<script type="application/ld+json">{{"@context":"https://schema.org","@type":"Dataset","name":{},"description":{},"url":"{BASE}/editions/{id}/","version":"{id}","datePublished":"{date}","temporalCoverage":{},"license":"https://creativecommons.org/licenses/by/4.0/","isAccessibleForFree":true,"creator":{{"@type":"Person","name":"Édouard Vallet"}},"keywords":["coding agents","agent harness","RAM","benchmark"],"variableMeasured":["private RAM per agent","CPU while waiting","fault recovery","crash recovery"],"distribution":[{{"@type":"DataDownload","encodingFormat":"application/json","contentUrl":"{BASE}/data/agentrambench-{id}.json"}},{{"@type":"DataDownload","encodingFormat":"text/csv","contentUrl":"{BASE}/data/agentrambench-{id}.csv"}}]}}</script>"#,
        json_str(&format!("AgentRAMBench edition {}", ed.id)),
        json_str(&format!("RAM, CPU, crash and outage results for {}, measured on one Linux desktop with a mock model.", names.join(", "))),
        json_str(&ed.measured_span().replace(" to ", "/")),
        id = ed.id,
    )
}

fn llms_txt(ed: &Edition) -> String {
    let mut s = format!(
        "# AgentRAMBench\n\n> How much memory and CPU coding-agent harnesses ({}) use on one machine, and what happens on crashes and API outages. Edition {} ({}), measured {} with a local mock model.\n\n## Data\n- [Edition JSON]({BASE}/data/agentrambench-{id}.json): every metric, run and value\n- [Edition CSV]({BASE}/data/agentrambench-{id}.csv)\n",
        ed.measured().iter().map(|h| h.name.as_str()).collect::<Vec<_>>().join(", "),
        ed.id,
        if ed.preliminary() { "preliminary" } else { "final" },
        ed.measured_span(),
        id = ed.id,
    );
    if let Some(u) = &ed.raw_archive {
        s.push_str(&format!("- [Raw runs]({u})\n"));
    }
    s.push_str(&format!("\n## Pages\n- [Leaderboard]({BASE}/)\n- [Method]({BASE}/method/)\n- [Resources]({BASE}/resources/)\n- [Failures]({BASE}/failures/)\n- [Harnesses]({BASE}/harnesses/)\n\n## Code\n- [Runner]({BENCH})\n- [Site]({REPO})\n"));
    s
}
