#![cfg(feature = "desktop")]

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[test]
fn smoke_errors_exit_without_waiting_for_a_dialog() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_economic-management"))
        .args(["--smoke-test", "--invalid-option"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("Smoke mode waited for an interactive error dialog");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Opção desconhecida"));
}
