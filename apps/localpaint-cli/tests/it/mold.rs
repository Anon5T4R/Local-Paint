//! The suite's agent-native mould in Local Paint: `commands --search`, `run --script` with a JSON
//! array, and `mcp --connect` (the suite's name for `--bridge`) taking a bare loopback port while
//! refusing any other host.

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn bin() -> Command {
    let mut cmd = Command::new(crate::binary());
    for var in ["LOCALPAINT_CONTROL_PORT", "LOCALPAINT_CONTROL_TOKEN", "LOCALPAINT_CONTROL_TOKEN_FILE"] {
        cmd.env_remove(var);
    }
    cmd
}

fn run(args: &[&str]) -> Output {
    bin().args(args).output().unwrap()
}

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("pc-cli-mould-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn commands_search_is_the_filter_alias() {
    let all = run(&["commands", "--json"]);
    assert!(all.status.success(), "{}", String::from_utf8_lossy(&all.stderr));
    let all: Vec<serde_json::Value> = serde_json::from_slice(&all.stdout).unwrap();
    let filtered = run(&["commands", "--json", "--filter", "blur"]);
    let searched = run(&["commands", "--json", "--search", "blur"]);
    assert!(searched.status.success(), "{}", String::from_utf8_lossy(&searched.stderr));
    assert_eq!(searched.stdout, filtered.stdout, "--search must be --filter");
    let found: Vec<serde_json::Value> = serde_json::from_slice(&searched.stdout).unwrap();
    assert!(!found.is_empty() && found.len() < all.len(), "{} of {}", found.len(), all.len());
}

#[test]
fn a_script_may_be_a_json_array() {
    let d = dir("array");
    let script = d.join("steps.json");
    std::fs::write(&script, r#"[{"command":"layer.new.layer","params":{"name":"Ink"}}]"#).unwrap();
    let out = d.join("out.png");
    let o = run(&["run", "--new", "{}", "--script", script.to_str().unwrap(), "--out", out.to_str().unwrap()]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(out.exists(), "the run saved the result");
}

#[test]
fn a_bad_array_step_names_its_index() {
    let d = dir("bad");
    let script = d.join("steps.json");
    std::fs::write(&script, r#"["layer.new.layer", {"params": {}}]"#).unwrap();
    let o = run(&["run", "--new", "{}", "--script", script.to_str().unwrap()]);
    assert!(!o.status.success());
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert!(stderr.contains("[1]") && stderr.contains("needs `command` or `id`"), "{stderr}");
}

#[test]
fn connect_refuses_a_public_host() {
    let o = Command::new(crate::binary()).args(["mcp", "--connect", "192.168.1.5:7878"]).stdin(Stdio::null()).output().unwrap();
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("loopback only"));
}
