//! Validation CLI only: JSONL input/output for the actual-source oracle runner.
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let result = line
            .map_err(|_| "Invalid probe input".to_string())
            .and_then(|l| {
                let v: serde_json::Value =
                    serde_json::from_str(&l).map_err(|_| "Invalid probe input")?;
                tezbar_json2ts::convert(
                    v["input"].as_str().ok_or("Missing input")?,
                    v["rootName"].as_str().unwrap_or("rootName"),
                )
            });
        println!(
            "{}",
            match result {
                Ok(s) => serde_json::json!({"ok":true,"output":s}),
                Err(e) => serde_json::json!({"ok":false,"error":e}),
            }
        );
    }
}
