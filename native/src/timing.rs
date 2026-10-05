use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Default)]
struct Row {
    ready: Option<u64>,
    launch: Option<u64>,
    visible: Option<u64>,
    closed: Option<u64>,
    finished: Option<u64>,
}
fn cell(v: Option<u64>) -> String {
    v.map(|n| n.to_string()).unwrap_or_default()
}
pub fn report(text: &str) -> String {
    let mut rows: BTreeMap<String, Row> = BTreeMap::new();
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(id) = v["session"].as_str() else {
            continue;
        };
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            continue;
        }
        let r = rows.entry(id.into()).or_default();
        let ms = v["elapsed_ms"].as_u64();
        match v["event"].as_str().unwrap_or("") {
            "launch_requested" => {
                r.ready = v["browser_ready_ms"].as_u64();
                r.launch = ms;
            }
            "capture_window_observed" => r.visible = ms,
            "capture_window_closed" => r.closed = ms,
            "finished" => r.finished = ms,
            _ => {}
        }
    }
    let mut csv=String::from("session,browser_ready_ms,native_launch_ms,window_observed_ms,estimated_click_to_bar_ms,window_closed_ms,finished_ms,close_to_finish_ms\n");
    for (id, r) in rows {
        let total = r.ready.zip(r.visible).map(|(a, b)| a.saturating_add(b));
        let cancel = r.closed.zip(r.finished).map(|(a, b)| b.saturating_sub(a));
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            id,
            cell(r.ready),
            cell(r.launch),
            cell(r.visible),
            cell(total),
            cell(r.closed),
            cell(r.finished),
            cell(cancel)
        ));
    }
    csv
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn joins_timing_events_and_leaves_missing_values_blank() {
        let data = r#"{"session":"a","event":"launch_requested","elapsed_ms":20,"browser_ready_ms":100}
{"session":"a","event":"capture_window_observed","elapsed_ms":300}
{"session":"a","event":"capture_window_closed","elapsed_ms":1000}
{"session":"a","event":"finished","elapsed_ms":1750}
{"session":"b","event":"finished"}
invalid"#;
        let result = report(data);
        assert!(result.contains("a,100,20,300,400,1000,1750,750"));
        assert!(result.contains("b,,,,,,,"));
    }
    #[test]
    fn rejects_csv_formula_session_ids() {
        assert_eq!(
            report(r#"{"session":"=bad","event":"finished"}"#)
                .lines()
                .count(),
            1
        );
    }
}
