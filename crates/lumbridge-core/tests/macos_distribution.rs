//! macOS bundle metadata and signing script, without production identities or credentials.
#![cfg(unix)]

use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

#[test]
fn calendar_and_reminder_descriptions_cover_macos_13_and_14() {
    let plist = include_str!("../../../distribution/macos-Info.plist");
    for service in ["Calendars", "Reminders"] {
        for access in ["", "FullAccess"] {
            assert!(plist.contains(&format!("<key>NS{service}{access}UsageDescription</key>")));
        }
    }
}

struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let root = PathBuf::from(format!(
            "/var/tmp/lbc-macos-distribution-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        Self { root }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn stub(s: &Scratch, name: &str, body: &str) {
    let path = s.root.join("tools").join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, format!("#!/bin/sh\nprintf '%s\\n' \"{name} $*\" >> \"$CALLS\"\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn script(s: &Scratch, name: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Command {
    let mut cmd = Command::new("bash");
    cmd.arg(format!("{ROOT}/scripts/{name}"))
        .args(args)
        .env("PATH", format!("{}:{}", s.root.join("tools").display(), std::env::var("PATH").unwrap()))
        .env("CALLS", s.root.join("calls"))
        .env_remove("APPLE_CERTIFICATE_P12")
        .env_remove("APPLE_CERTIFICATE_PASSWORD");
    cmd
}

#[test]
fn mac_sign_uses_runtime_and_entitlements_even_for_ad_hoc_builds() {
    let s = Scratch::new();
    stub(&s, "codesign", "exit 0");
    std::fs::create_dir_all(s.root.join("lumbridge-core/Contents/MacOS")).unwrap();
    let out = script(&s, "mac-sign", [s.root.join("lumbridge-core")]).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    let calls = std::fs::read_to_string(s.root.join("calls")).unwrap();
    assert!(calls.contains("--options runtime"), "{calls}");
    assert!(calls.contains("--entitlements ") && calls.contains("macos-entitlements.plist"), "{calls}");
    assert!(calls.contains("--identifier com.lumbridgecorp.core"), "{calls}");
    assert!(calls.contains("--verify --strict"), "{calls}");
    let lines: Vec<_> = calls.lines().collect();
    assert!(lines[0].ends_with("/Contents/MacOS/lumbridge-core"), "nested executable first: {calls}");
    assert!(lines[1].ends_with("/lumbridge-core"), "then bundle: {calls}");
}

#[test]
fn advertised_personal_information_has_runtime_entitlements() {
    let entitlements = std::fs::read_to_string(format!("{ROOT}/distribution/macos-entitlements.plist")).unwrap();
    for key in ["addressbook", "calendars", "photos-library"] {
        assert!(entitlements.contains(&format!("com.apple.security.personal-information.{key}</key><true/>")));
    }
}
