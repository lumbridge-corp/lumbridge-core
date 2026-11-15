#![cfg(unix)] // Windows: tests/windows.rs

mod common;

use common::*;
use serde_json::{Value, json};

#[test]
fn update_needs_a_local_build_and_never_downloads() {
    let s = Session::new();
    let out = s.lbc(&["update"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--from"), "{out:?}");
    // The old download options are gone.
    assert!(!s.lbc(&["update", "--version", "9.9.9"]).status.success());
    assert!(!s.connect_ok(), "a refused update must not start a core");
}

#[test]
fn an_install_in_a_read_only_folder_explains_how_to_update_it() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        return; // Root can replace files in read-only directories.
    }
    let mut s = Session::new();
    let dir = s.root.join("bin");
    std::fs::create_dir(&dir).unwrap();
    s.bin = dir.join("lumbridge-core");
    std::fs::copy(BIN, &s.bin).unwrap();
    s.json(&["server", "start"]);
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();
    let status = s.cmd().args(["server", "status"]).output().unwrap();
    let update = s.cmd().args(["update", "--from", BIN]).output().unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
    let status: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["installed_by"], "system package manager");
    let error: Value = serde_json::from_slice(&update.stderr).unwrap();
    assert!(!update.status.success());
    assert_eq!(error["error"]["kind"], "refused", "{error}");
    assert_eq!(error["error"]["message"], format!("{} isn't writable by you", dir.canonicalize().unwrap().display()));
    assert_eq!(error["error"]["hint"], "update lumbridge-core the way it was installed (package manager, or sudo)");
}

#[test]
fn update_from_a_file_swaps_lbc_and_keeps_everything_running() {
    let mut s = Session::new();
    s.env.push(("LBC_SESSION_VALUE".into(), "original".into()));
    let dir = s.root.join("bin");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(BIN, dir.join("lumbridge-core")).unwrap();
    std::os::unix::fs::symlink("lumbridge-core", dir.join("lbc")).unwrap();
    s.bin = dir.join("lbc"); // run as `lbc`, the way people do
    let t = run(&s, &["sh", "-c", "echo before; while read l; do echo got-$l; done"]);
    let pid = s.json(&["server", "status"])["pid"].clone();

    let out = s
        .cmd()
        .env("NO_COLOR", "1")
        .env("LBC_SESSION_VALUE", "updater")
        .env("LBC_UPDATER_ONLY", "leaked")
        .args(["update", "--from", BIN])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let r: Value = serde_json::from_slice(&out.stdout).unwrap();
    let real = std::fs::canonicalize(dir.join("lumbridge-core")).unwrap(); // macOS: /var is /private/var
    assert_eq!(r["path"], real.display().to_string(), "the file lbc runs from, not the link: {r}");
    assert!(r["was"].as_str().unwrap().starts_with("lbc ") && r["now"].as_str().unwrap().starts_with("lbc "), "{r}");
    assert_eq!(r["restarted"], true);
    assert_ne!(s.json(&["server", "status"])["pid"], pid, "a new core runs");
    assert!(dir.join("lbc").is_symlink(), "the link stays a link");
    s.json(&["send", &t, "after"]);
    wait_until("the terminal still answers", || text(&s, &t).contains("got-after"));
    let probe = "printf 'ENV:%s|%s|%s\\n' \"${NO_COLOR-unset}\" \"$LBC_SESSION_VALUE\" \"${LBC_UPDATER_ONLY-unset}\"";
    let out = s.lbc(&["run", "sh", "-c", probe]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("ENV:unset|original|unset"), "{out:?}");
    let stream = lumbridge_core::platform::connect(&s.socket()).unwrap();
    let terminal = lumbridge_core::client::call(
        &stream,
        "terminal.run",
        json!({
            "argv": ["sh", "-c", probe],
            "env": {"NO_COLOR": "1", "LBC_SESSION_VALUE": "override"},
        }),
    )
    .unwrap();
    wait_until("per-terminal environment override", || {
        text(&s, terminal["address"].as_str().unwrap()).contains("ENV:1|override|unset")
    });

    let older = s.root.join("older");
    std::fs::write(&older, "#!/bin/sh\necho 'lbc 0.0.1'\n").unwrap();
    std::fs::set_permissions(&older, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    let (_, e) = s.err(&["--json", "update", "--from", older.to_str().unwrap()]);
    assert_eq!(e["error"]["kind"], "refused", "never older by accident: {e}");
    let junk = s.root.join("junk");
    std::fs::write(&junk, "not a program").unwrap();
    assert_eq!(s.err(&["--json", "update", "--from", junk.to_str().unwrap()]).1["error"]["kind"], "refused");
    assert!(
        std::fs::read_dir(&dir).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().contains(".new")),
        "no leftovers"
    );
}
