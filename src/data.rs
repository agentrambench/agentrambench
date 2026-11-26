use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Deserialize, Clone, Default)]
pub struct Machine {
    #[serde(default)]
    pub cpu: String,
    #[serde(default)]
    pub cores: u32,
    #[serde(default)]
    pub ram_gib: f64,
    #[serde(default)]
    pub kernel: String,
}

#[derive(Deserialize, Clone, Default)]
pub struct Reproduce {
    #[serde(default)]
    pub command: String,
    pub minutes: Option<f64>,
    pub repo: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Gap {
    pub metrics: Vec<String>,
    pub reason: String,
}

#[derive(Deserialize, Clone)]
pub struct Erratum {
    pub harness: String,
    pub metrics: Vec<String>,
    pub date: String,
    pub note: String,
}

#[derive(Deserialize, Clone)]
pub struct Harness {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub status: String,
    pub non_default_flags: Option<String>,
    pub reason: Option<String>,
    pub provisional: Option<String>,
    pub notes: Option<Vec<String>>,
}

#[derive(Deserialize, Clone)]
pub struct Metric {
    pub id: String,
    pub axis: String,
    pub label: String,
    #[serde(default)]
    pub unit: String,
    pub better: String,
    #[serde(default)]
    pub headline: bool,
    pub note: Option<String>,
    pub in_totals: Option<bool>,
    pub group: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct Res {
    pub harness: String,
    pub metric: String,
    #[serde(default)]
    pub n_agents: u32,
    pub runs: u32,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    #[serde(default)]
    pub values: Vec<f64>,
    #[serde(default)]
    pub source: String,
    pub stop_reason: Option<String>,
    pub hold_s: Option<f64>,
    #[serde(default)]
    pub steps: Option<u32>,
    #[serde(default)]
    pub session_mb: Option<f64>,
    pub grow_note: Option<String>,
    pub applicable: Option<u32>,
    pub na: Option<u32>,
}

#[derive(Deserialize, Clone)]
pub struct Edition {
    pub schema: String,
    pub edition: String,
    pub frozen: Option<String>,
    #[serde(default)]
    pub bench_commit: String,
    #[serde(default)]
    pub llm: String,
    #[serde(default)]
    pub machine: Machine,
    pub status: Option<String>,
    pub reproduce: Option<Reproduce>,
    pub raw_archive: Option<String>,
    #[serde(default)]
    pub fault_labels: BTreeMap<String, String>,
    pub harnesses: Vec<Harness>,
    pub metrics: Vec<Metric>,
    pub results: Vec<Res>,
    #[serde(skip)]
    pub id: String,
    #[serde(skip)]
    pub raw: String,
    #[serde(skip)]
    pub errata: Vec<Erratum>,
    #[serde(default)]
    pub fault_na: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub withheld: BTreeMap<String, Gap>,
    #[serde(default)]
    pub not_measured: BTreeMap<String, BTreeMap<String, Gap>>,
    #[serde(default)]
    pub hostile_labels: BTreeMap<String, String>,
    #[serde(default)]
    pub hostile_na: BTreeMap<String, BTreeMap<String, String>>,
}

impl Metric {
    pub fn worst_case(&self) -> bool {
        self.axis == "governance" && self.better == "lower"
    }

    pub fn key(&self, r: &Res) -> f64 {
        if self.worst_case() { r.max } else { r.median }
    }
}

impl Edition {
    pub fn load(path: &Path) -> Result<Edition, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut e: Edition = serde_json::from_str(&raw).map_err(|e| format!("{}: {e}", path.display()))?;
        if !e.schema.starts_with("agentrambench/1") {
            return Err(format!("{}: unsupported schema {}", path.display(), e.schema));
        }
        e.id = path.file_stem().unwrap().to_string_lossy().into_owned();
        e.raw = raw;
        let ep = path.parent().and_then(|d| d.parent()).map(|d| d.join("errata").join(format!("{}.json", e.id)));
        if let Some(ep) = ep.filter(|p| p.exists()) {
            let txt = std::fs::read_to_string(&ep).map_err(|x| format!("{}: {x}", ep.display()))?;
            e.errata = serde_json::from_str(&txt).map_err(|x| format!("{}: {x}", ep.display()))?;
            let er = e.errata.clone();
            e.results.retain(|r| !er.iter().any(|x| x.harness == r.harness && x.metrics.contains(&r.metric)));
        }
        let sd = path.parent().and_then(|d| d.parent()).map(|d| d.join("selfrun").join(&e.id));
        if let Some(Ok(rd)) = sd.map(std::fs::read_dir) {
            let mut fs: Vec<_> = rd.filter_map(|x| x.ok().map(|x| x.path())).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
            fs.sort();
            for f in fs {
                let txt = std::fs::read_to_string(&f).map_err(|x| format!("{}: {x}", f.display()))?;
                let x: Edition = serde_json::from_str(&txt).map_err(|err| format!("{}: {err}", f.display()))?;
                for mut h in x.harnesses.into_iter().filter(|h| h.status == "self_run") {
                    let id = h.id.clone();
                    e.results.extend(x.results.iter().filter(|r| r.harness == id).cloned());
                    h.status = "self_run".into();
                    e.harnesses.push(h);
                }
                for m in x.metrics {
                    if e.metric(&m.id).is_none() {
                        e.metrics.push(m);
                    }
                }
            }
        }
        Ok(e)
    }

    pub fn erratum(&self, harness: &str, metric: &str) -> Option<&Erratum> {
        self.errata.iter().find(|x| x.harness == harness && x.metrics.iter().any(|m| m == metric))
    }

    pub fn gap(&self, harness: &str, metric: &str) -> Option<&str> {
        if let Some(c) = metric.strip_prefix("hostile_").and_then(|x| x.rsplit_once('_')).map(|x| x.0) {
            if let Some(w) = self.hostile_na.get(harness).and_then(|m| m.get(c)) {
                return Some(w.as_str());
            }
        }
        let has = |g: &Gap| g.metrics.iter().any(|m| m == metric);
        self.withheld.get(harness).filter(|g| has(g)).map(|g| g.reason.as_str())
            .or_else(|| self.not_measured.get(harness).and_then(|m| m.values().find(|g| has(g))).map(|g| g.reason.as_str()))
    }

    pub fn has_axis(&self, axis: &str) -> bool {
        self.metrics.iter().any(|m| m.axis == axis && self.has_data(&m.id))
    }

    pub fn preliminary(&self) -> bool {
        self.status.as_deref() != Some("final")
    }

    pub fn metric(&self, id: &str) -> Option<&Metric> {
        self.metrics.iter().find(|m| m.id == id)
    }

    pub fn result(&self, harness: &str, metric: &str) -> Option<&Res> {
        let rs: Vec<&Res> = self.results.iter().filter(|r| r.metric == metric).collect();
        let steps = self.common_steps(metric);
        let burst = metric.starts_with("burst_");
        let bn = if burst { self.common_n(metric) } else { None };
        rs.into_iter()
            .filter(|r| r.harness == harness && (steps.is_none() || r.steps == steps) && (!burst || r.n_agents > 1) && (bn.is_none() || Some(r.n_agents) == bn))
            .max_by_key(|r| (r.hold_s.unwrap_or(0.0) as u64, r.n_agents))
    }

    pub fn ranked_id(&self, id: &str) -> bool {
        self.harnesses.iter().any(|h| h.id == id && h.status == "measured")
    }

    pub fn self_runs(&self) -> Vec<&Harness> {
        self.harnesses.iter().filter(|h| h.status == "self_run").collect()
    }

    pub fn common_steps(&self, metric: &str) -> Option<u32> {
        let rs: Vec<&Res> = self.results.iter().filter(|r| r.metric == metric && r.steps.is_some() && self.ranked_id(&r.harness)).collect();
        let mut hs: Vec<&str> = rs.iter().map(|r| r.harness.as_str()).collect();
        hs.sort();
        hs.dedup();
        let mut cand: Vec<u32> = rs.iter().filter_map(|r| r.steps).collect();
        cand.sort();
        cand.dedup();
        cand.into_iter().rev().find(|st| hs.iter().all(|h| rs.iter().any(|r| r.harness == *h && r.steps == Some(*st))))
    }

    pub fn common_n(&self, metric: &str) -> Option<u32> {
        let rs: Vec<&Res> = self.results.iter().filter(|r| r.metric == metric && r.n_agents > 1 && self.ranked_id(&r.harness)).collect();
        let mut hs: Vec<&str> = rs.iter().map(|r| r.harness.as_str()).collect();
        hs.sort();
        hs.dedup();
        let mut cand: Vec<u32> = rs.iter().map(|r| r.n_agents).collect();
        cand.sort();
        cand.dedup();
        cand.into_iter().rev().find(|n| hs.iter().all(|h| rs.iter().any(|r| r.harness == *h && r.n_agents == *n)))
    }

    pub fn measured_span(&self) -> String {
        let mut ds: Vec<&str> = self.results.iter().filter_map(|r| {
            let src = r.source.as_str();
            let i = src.find("2026").or_else(|| src.find("2027"))?;
            let d = src.get(i..i + 8)?;
            d.bytes().all(|b| b.is_ascii_digit()).then_some(d)
        }).collect();
        ds.sort();
        let f = |d: &str| format!("{}-{}-{}", &d[..4], &d[4..6], &d[6..]);
        match (ds.first(), ds.last()) {
            (Some(a), Some(b)) if a != b => format!("{} to {}", f(a), f(b)),
            (Some(a), _) => f(a),
            _ => self.edition.clone(),
        }
    }

        pub fn measured(&self) -> Vec<&Harness> {
        self.harnesses.iter().filter(|h| h.status == "measured").collect()
    }

    pub fn has_data(&self, metric: &str) -> bool {
        self.results.iter().any(|r| r.metric == metric)
    }

    pub fn ranking(&self, metric: &str) -> Vec<(u32, &Harness, &Res)> {
        let Some(m) = self.metric(metric) else { return vec![] };
        let mut rows: Vec<(&Harness, &Res)> = self
            .measured()
            .into_iter()
            .filter_map(|h| self.result(&h.id, metric).map(|r| (h, r)))
            .collect();
        let higher = m.better == "higher";
        rows.sort_by(|a, b| {
            let o = m.key(a.1).partial_cmp(&m.key(b.1)).unwrap();
            if higher { o.reverse() } else { o }
        });
        let mut out = Vec::new();
        for (i, (h, r)) in rows.iter().enumerate() {
            let rank = if i > 0 && m.key(rows[i - 1].1) == m.key(r) { out.last().map(|x: &(u32, &Harness, &Res)| x.0).unwrap() } else { i as u32 + 1 };
            out.push((rank, *h, *r));
        }
        out
    }

    pub fn graded(&self) -> bool {
        self.results.iter().any(|r| r.metric.starts_with("fault_") && r.metric.ends_with("_score"))
    }

    pub fn fault_suffix(&self) -> &'static str {
        if self.graded() { "_score" } else { "_done_alone" }
    }

    pub fn fault_na(&self, harness: &str, profile: &str) -> bool {
        self.fault_na.get(harness).is_some_and(|v| v.iter().any(|p| p == profile))
    }

    pub fn fault_tally(&self, harness: &str) -> Option<(f64, usize)> {
        let sfx = self.fault_suffix();
        let rs: Vec<&Res> = self
            .results
            .iter()
            .filter(|r| r.harness == harness && r.metric.starts_with("fault_") && r.metric.ends_with(sfx) && self.metric(&r.metric).and_then(|m| m.in_totals) != Some(false))
            .collect();
        if rs.is_empty() {
            return None;
        }
        if self.graded() {
            return Some((rs.iter().map(|r| r.median).sum(), rs.len()));
        }
        Some((rs.iter().flat_map(|r| r.values.iter()).sum(), rs.iter().map(|r| r.values.len()).sum()))
    }
}

pub fn load_all(dir: &Path) -> Result<Vec<Edition>, String> {
    let mut eds = Vec::new();
    for ent in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = ent.map_err(|e| e.to_string())?.path();
        if p.extension().is_some_and(|x| x == "json") {
            eds.push(Edition::load(&p)?);
        }
    }
    if eds.is_empty() {
        return Err(format!("no editions in {}", dir.display()));
    }
    eds.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(eds)
}

pub fn is_binary(r: &Res) -> bool {
    !r.values.is_empty() && r.values.iter().all(|v| *v == 0.0 || *v == 1.0)
}

pub fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn ed(metrics: &str, results: &str) -> Edition {
        let j = format!(
            r#"{{"schema":"agentrambench/1","edition":"t","harnesses":[
            {{"id":"claude","name":"Claude Code","status":"measured"}},
            {{"id":"codex","name":"Codex CLI","status":"measured"}},
            {{"id":"opencode","name":"opencode","status":"measured"}},
            {{"id":"pi","name":"Pi","status":"measured"}},
            {{"id":"goose","name":"Goose","status":"not_runnable_offline"}}],
            "metrics":[{metrics}],"results":[{results}]}}"#
        );
        serde_json::from_str(&j).unwrap()
    }

    pub fn m(id: &str, better: &str) -> String {
        format!(r#"{{"id":"{id}","axis":"resources","label":"{id}","better":"{better}"}}"#)
    }

    pub fn r(h: &str, metric: &str, n: u32, hold: Option<f64>, median: f64, extra: &str) -> String {
        let hold = hold.map(|x| format!(r#","hold_s":{x}"#)).unwrap_or_default();
        format!(r#"{{"harness":"{h}","metric":"{metric}","n_agents":{n},"runs":1,"median":{median},"min":{median},"max":{median}{hold}{extra}}}"#)
    }

    #[test]
    fn longest_hold_wins_over_short_cell() {
        // bug 07/11: the N=5/300 s cell turned 5 min of codex upload into 3407 KiB/h; the 1 h hold measured 31.6
        let e = ed(&m("idle_net_tx_kib_per_agent_hour", "lower"), &[r("codex", "idle_net_tx_kib_per_agent_hour", 5, Some(300.0), 3407.0, r#","comparable":true"#), r("codex", "idle_net_tx_kib_per_agent_hour", 50, Some(3600.0), 31.6, "")].join(","));
        assert_eq!(e.result("codex", "idle_net_tx_kib_per_agent_hour").unwrap().median, 31.6);
    }

    #[test]
    fn equal_hold_takes_largest_fleet() {
        let e = ed(&m("x", "lower"), &[r("pi", "x", 50, Some(3600.0), 94.8, ""), r("pi", "x", 10, Some(3600.0), 90.0, ""), r("pi", "x", 25, Some(900.0), 80.0, "")].join(","));
        assert_eq!(e.result("pi", "x").unwrap().n_agents, 50);
    }

    #[test]
    fn burst_compares_at_largest_fleet_size_shared_by_all() {
        // review 07/11: claude N=1 was shown next to codex N=50; decision 08/11: compare at the largest N every harness ran
        let rs = [r("claude", "burst_cpu_s_per_agent", 1, None, 0.5, ""), r("claude", "burst_cpu_s_per_agent", 10, None, 0.94, ""), r("codex", "burst_cpu_s_per_agent", 10, None, 0.54, ""), r("codex", "burst_cpu_s_per_agent", 50, None, 0.73, "")];
        let e = ed(&m("burst_cpu_s_per_agent", "lower"), &rs.join(","));
        assert_eq!(e.common_n("burst_cpu_s_per_agent"), Some(10));
        assert_eq!(e.result("claude", "burst_cpu_s_per_agent").unwrap().median, 0.94);
        assert_eq!(e.result("codex", "burst_cpu_s_per_agent").unwrap().median, 0.54);
    }

    #[test]
    fn largest_shared_size_ignores_other_metrics_and_harnesses_missing_it() {
        // hand-worked: codex and pi reached 300 steps, claude only 60 → no step count shared by all
        let id = "resumed_anon_mib_per_agent";
        let rs = [r("codex", id, 1, None, 73.2, r#","steps":300"#), r("pi", id, 1, None, 64.9, r#","steps":300"#), r("claude", id, 1, None, 163.0, r#","steps":60"#), r("opencode", "resumed_peak_anon_mib_per_agent", 1, None, 786.0, r#","steps":5"#)];
        let e = ed(&m(id, "lower"), &rs.join(","));
        assert_eq!(e.common_steps(id), None);
        let rs = [r("codex", id, 1, None, 79.4, r#","steps":60"#), r("pi", id, 1, None, 58.1, r#","steps":60"#), r("opencode", "resumed_peak_anon_mib_per_agent", 1, None, 786.0, r#","steps":5"#)];
        assert_eq!(ed(&m(id, "lower"), &rs.join(",")).common_steps(id), Some(60));
        // burst: claude solo only, codex 10+50, pi 10 → 10 (claude not counted); codex 50 + pi 25 + opencode 10 → none
        let b = "burst_cpu_s_per_agent";
        let rs = [r("claude", b, 1, None, 0.5, ""), r("codex", b, 10, None, 0.54, ""), r("codex", b, 50, None, 0.73, ""), r("pi", b, 10, None, 0.22, ""), r("opencode", "burst_turn_s_p50", 2, None, 7.0, "")];
        assert_eq!(ed(&m(b, "lower"), &rs.join(",")).common_n(b), Some(10));
        let rs = [r("codex", b, 50, None, 0.73, ""), r("pi", b, 25, None, 0.2, ""), r("opencode", b, 10, None, 1.0, "")];
        assert_eq!(ed(&m(b, "lower"), &rs.join(",")).common_n(b), None);
        // codex and pi ran 10, opencode only 25: 10 is not shared by opencode even though two others have it
        let rs = [r("codex", b, 10, None, 0.54, ""), r("pi", b, 10, None, 0.22, ""), r("opencode", b, 25, None, 1.0, "")];
        assert_eq!(ed(&m(b, "lower"), &rs.join(",")).common_n(b), None);
    }

    #[test]
    fn burst_with_only_solo_runs_shows_nothing() {
        let e = ed(&m("burst_turn_s_p50", "lower"), &r("claude", "burst_turn_s_p50", 1, None, 7.0, ""));
        assert!(e.result("claude", "burst_turn_s_p50").is_none());
    }

    #[test]
    fn edition_status_and_data_presence() {
        let mut e = ed(&m("a", "lower"), &r("pi", "a", 1, None, 1.0, ""));
        assert!(e.preliminary(), "no status = preliminary");
        e.status = Some("preliminary".into());
        assert!(e.preliminary());
        e.status = Some("final".into());
        assert!(!e.preliminary());
        assert!(e.has_data("a"));
        assert!(!e.has_data("b"));
    }

    #[test]
    fn burst_solo_run_is_not_a_fleet_result() {
        let rs = [r("claude", "burst_turn_s_p50", 1, None, 7.0, ""), r("codex", "burst_turn_s_p50", 50, None, 7.6, "")];
        let e = ed(&m("burst_turn_s_p50", "lower"), &rs.join(","));
        assert!(e.result("claude", "burst_turn_s_p50").is_none());
        assert_eq!(e.result("codex", "burst_turn_s_p50").unwrap().n_agents, 50);
    }

    #[test]
    fn resumed_sessions_compare_at_largest_step_count_shared_by_all() {
        // 07/11 08:30: codex had 300 steps, others only 60 → compare at 60; 08/11 all have 300
        let id = "resumed_anon_mib_per_agent";
        let mut rs = vec![r("codex", id, 1, None, 79.4, r#","steps":60"#), r("codex", id, 1, None, 73.2, r#","steps":300"#), r("pi", id, 1, None, 58.1, r#","steps":60"#)];
        let e = ed(&m(id, "lower"), &rs.join(","));
        assert_eq!(e.common_steps(id), Some(60));
        assert_eq!(e.result("codex", id).unwrap().median, 79.4);
        rs.push(r("pi", id, 1, None, 64.9, r#","steps":300"#));
        let e = ed(&m(id, "lower"), &rs.join(","));
        assert_eq!(e.common_steps(id), Some(300));
        assert_eq!(e.result("codex", id).unwrap().median, 73.2);
        assert_eq!(e.result("pi", id).unwrap().median, 64.9);
    }

    #[test]
    fn ranking_order_direction_ties_and_unmeasured() {
        let rs = [r("claude", "a", 50, None, 145.0, ""), r("codex", "a", 50, None, 28.4, ""), r("pi", "a", 50, None, 28.4, ""), r("goose", "a", 50, None, 1.0, "")];
        let e = ed(&m("a", "lower"), &rs.join(","));
        let k: Vec<(u32, &str)> = e.ranking("a").iter().map(|x| (x.0, x.1.id.as_str())).collect();
        assert_eq!(k.len(), 3, "not_run harness excluded");
        assert_eq!(k[0].0, 1);
        assert_eq!(k[1].0, 1, "tie shares rank");
        assert_eq!(k[2], (3, "claude"));
        let e = ed(&m("a", "higher"), &rs.join(","));
        assert_eq!(e.ranking("a")[0].1.id, "claude");
        assert!(e.ranking("missing").is_empty());
    }

    #[test]
    fn fault_totals_skip_metrics_flagged_out_of_totals() {
        // harness-bench 07/11 13:05: loop-3min in_totals=false, totals /28
        let ms = [
            m("fault_429-60s_done_alone", "higher"),
            r#"{"id":"fault_loop-3min_done_alone","axis":"failures","label":"l","better":"higher","in_totals":false}"#.to_string(),
        ];
        let rs = [
            r("pi", "fault_429-60s_done_alone", 1, None, 1.0, r#","values":[1,0]"#),
            r("pi", "fault_loop-3min_done_alone", 1, None, 0.0, r#","values":[0,0]"#),
            r("pi", "crash_kill9_recovered_alone", 1, None, 0.0, r#","values":[1,1,1]"#),
        ];
        let e = ed(&ms.join(","), &rs.join(","));
        assert_eq!(e.fault_tally("pi"), Some((1.0, 2)));
        assert_eq!(e.fault_tally("codex"), None);
    }

    #[test]
    fn load_rejects_other_schema() {
        let d = std::env::temp_dir().join(format!("arb-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("2026-11.json");
        std::fs::write(&p, r#"{"schema":"harnessdyno/1","edition":"t","harnesses":[],"metrics":[],"results":[]}"#).unwrap();
        assert!(Edition::load(&p).err().unwrap().contains("unsupported schema"));
        std::fs::write(&p, r#"{"schema":"agentrambench/1","edition":"t","harnesses":[],"metrics":[],"results":[]}"#).unwrap();
        assert_eq!(Edition::load(&p).unwrap().id, "2026-11");
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn measured_span_from_run_sources() {
        // 08/11 export: runs from 2026-11-06 and 2026-11-07, edition field said 2026-11-07
        let e = ed(&m("a", "lower"), &[r("pi", "a", 50, None, 1.0, r#","source":"idle-pi-50-hbm-tmux-20261107-110746""#), r("codex", "a", 5, None, 1.0, r#","source":"task-codex-20261106-1530""#), r("claude", "a", 5, None, 1.0, "")].join(","));
        assert_eq!(e.measured_span(), "2026-11-06 to 2026-11-07");
        let e = ed(&m("a", "lower"), &r("pi", "a", 1, None, 1.0, r#","source":"x-20261107-1""#));
        assert_eq!(e.measured_span(), "2026-11-07");
        let e = ed(&m("a", "lower"), &r("pi", "a", 1, None, 1.0, ""));
        assert_eq!(e.measured_span(), e.edition);
    }

    #[test]
    fn governance_ranks_on_worst_run() {
        // review 07/11 #17: codex task upload runs 0,0,0,0,613 KiB had median 0, tied with pi (0-0) and got "Best"
        let gm = r#"{"id":"task_net_out_kib","axis":"governance","label":"l","better":"lower"}"#;
        let rs = [r("codex", "task_net_out_kib", 1, None, 0.0, r#","max":613,"values":[0,0,0,0,613]"#).replace(r#""max":0,"#, ""), r("pi", "task_net_out_kib", 1, None, 0.0, "")];
        let e = ed(gm, &rs.join(","));
        let k: Vec<(u32, &str)> = e.ranking("task_net_out_kib").iter().map(|x| (x.0, x.1.id.as_str())).collect();
        assert_eq!(k, vec![(1, "pi"), (2, "codex")]);
        let e = ed(&m("task_net_out_kib", "lower"), &rs.join(","));
        assert_eq!(e.ranking("task_net_out_kib")[1].0, 1, "non-governance metrics still rank on median");
    }

    #[test]
    fn errata_withdraw_rows_and_fault_totals() {
        // harness-bench 17/11: gemini truncate faults never delivered in 2026-11, scored 1.0
        let d = std::env::temp_dir().join(format!("arb-err-{}", std::process::id()));
        std::fs::create_dir_all(d.join("editions")).unwrap();
        std::fs::create_dir_all(d.join("errata")).unwrap();
        let rs = [r("gemini", "fault_truncate_done_alone", 1, None, 1.0, r#","values":[1,1]"#), r("gemini", "fault_429x3_done_alone", 1, None, 1.0, r#","values":[1,0]"#)];
        let j = format!(r#"{{"schema":"agentrambench/1","edition":"2026-11","harnesses":[{{"id":"gemini","name":"Gemini CLI","status":"measured"}}],"metrics":[{},{}],"results":[{}]}}"#, m("fault_truncate_done_alone", "higher"), m("fault_429x3_done_alone", "higher"), rs.join(","));
        std::fs::write(d.join("editions/2026-11.json"), &j).unwrap();
        let e = Edition::load(&d.join("editions/2026-11.json")).unwrap();
        assert_eq!(e.fault_tally("gemini"), Some((3.0, 4)));
        std::fs::write(d.join("errata/2026-11.json"), r#"[{"harness":"gemini","metrics":["fault_truncate_done_alone"],"date":"2026-11-17","note":"n"}]"#).unwrap();
        let e = Edition::load(&d.join("editions/2026-11.json")).unwrap();
        assert_eq!(e.fault_tally("gemini"), Some((1.0, 2)));
        assert!(e.result("gemini", "fault_truncate_done_alone").is_none());
        assert!(e.erratum("gemini", "fault_truncate_done_alone").is_some());
        assert!(e.erratum("gemini", "fault_429x3_done_alone").is_none());
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn withheld_and_not_measured_give_reasons() {
        // 2026-12 export: codex role split withheld, opencode marathon stopped at turn 234
        let mut e = ed(&m("marathon_idle_anon_mib", "lower"), &r("pi", "marathon_idle_anon_mib", 1, None, 134.0, ""));
        e.withheld.insert("codex".into(), Gap { metrics: vec!["tool_children_anon_mib".into()], reason: "role split withdrawn".into() });
        e.not_measured.insert("opencode".into(), [("marathon".to_string(), Gap { metrics: vec!["marathon_idle_anon_mib".into()], reason: "stopped after auto-compaction at turn 234".into() })].into_iter().collect());
        assert_eq!(e.gap("codex", "tool_children_anon_mib"), Some("role split withdrawn"));
        assert_eq!(e.gap("opencode", "marathon_idle_anon_mib"), Some("stopped after auto-compaction at turn 234"));
        assert_eq!(e.gap("opencode", "marathon_peak_anon_mib"), None);
        assert_eq!(e.gap("pi", "marathon_idle_anon_mib"), None);
    }

    #[test]
    fn binary_means_all_zero_or_one() {
        let e = ed(&m("a", "higher"), &[r("pi", "a", 1, None, 1.0, r#","values":[1,0,1]"#), r("codex", "a", 1, None, 0.7, r#","values":[1,0.7]"#)].join(","));
        assert!(is_binary(e.result("pi", "a").unwrap()));
        assert!(!is_binary(e.result("codex", "a").unwrap()));
        assert_eq!(slug("Codex CLI (0.1)"), "codex-cli-0-1");
    }
}
