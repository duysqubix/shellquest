//! Regression test for shellqeuest-x3p.1: several shells ticking at once must neither
//! corrupt the save nor lose each other's updates. Drives the real binary against a
//! throwaway HOME; it never touches the developer's save.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const SHELLS: usize = 8;
const ROUNDS: usize = 8;

fn sq(home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_sq"));
    // SQ_DEBUG makes a tick that gives up on a busy lock exit non-zero with a
    // message, so the count below stays exact even on a slow machine.
    cmd.env("HOME", home)
        .env("SQ_NO_PACING", "1")
        .env("SQ_DEBUG", "1");
    cmd
}

fn fresh_home() -> PathBuf {
    let home = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("concurrent-saves-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    home
}

#[test]
fn concurrent_ticks_keep_every_update() {
    let home = fresh_home();
    let mut init = sq(&home)
        .arg("init")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    init.stdin
        .take()
        .unwrap()
        .write_all(b"Racer\n2\n1\nn\n")
        .unwrap();
    assert!(init.wait().unwrap().success());

    // Mark the crates.io check as fresh so no tick touches the network.
    let save = home.join(".shellquest").join("save.json");
    let mut state: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&save).unwrap()).unwrap();
    state["last_version_check"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
    fs::write(&save, serde_json::to_string_pretty(&state).unwrap()).unwrap();

    let mut counted = 0;
    for _ in 0..ROUNDS {
        let ticks: Vec<_> = (0..SHELLS)
            .map(|_| {
                sq(&home)
                    .args(["tick", "--cmd=ls", "--exit-code=0"])
                    .arg(format!("--cwd={}", home.display()))
                    .stdout(Stdio::null())
                    .stderr(Stdio::piped())
                    .spawn()
                    .unwrap()
            })
            .collect();
        for tick in ticks {
            let out = tick.wait_with_output().unwrap();
            let stderr = String::from_utf8_lossy(&out.stderr);
            if out.status.success() {
                counted += 1;
            } else {
                // The only acceptable failure is a tick that gave up waiting for the lock.
                assert!(
                    stderr.contains("Tick lock failure"),
                    "tick failed: {}",
                    stderr
                );
            }
            assert!(!stderr.contains("Failed to"), "a tick reported: {}", stderr);
        }
    }

    let state: serde_json::Value = serde_json::from_str(&fs::read_to_string(&save).unwrap())
        .expect("the save must stay parseable");
    assert!(
        counted >= SHELLS * ROUNDS / 2,
        "too many ticks gave up: {}",
        counted
    );
    assert_eq!(
        state["character"]["commands_run"],
        serde_json::json!(counted),
        "every tick that succeeded must be counted exactly once"
    );
    let _ = fs::remove_dir_all(&home);
}
