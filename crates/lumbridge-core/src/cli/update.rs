//! `lbc update --from FILE`: swap this lbc for another build, keep the terminals running.

use super::*;

/// What a binary says it is (`lbc 0.2.0`), if it is a working lbc.
pub(super) fn version_of(path: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new(path).arg("--version").stdin(std::process::Stdio::null()).output().ok()?;
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (out.status.success() && v.starts_with("lbc ")).then_some(v)
}

/// `lbc 0.1.9` is older than `lbc 0.2.0`; anything unparsed is not older.
pub(super) fn older(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Option<Vec<u64>> {
        v.strip_prefix("lbc ")?.split(['.', '-']).take(3).map(|p| p.parse().ok()).collect()
    };
    matches!((parts(a), parts(b)), (Some(a), Some(b)) if a < b)
}

/// Swaps the binary this lbc runs from for `from`, then restarts the core
/// the planned way: every terminal keeps running. One swap path, for people
/// and for apps that bundle lbc.
pub(super) fn update(paths: &Paths, from: &std::path::Path, force: bool) -> Result<Out, RpcError> {
    let source = crate::platform::update_source(from);
    let from = source.as_path();
    let refused = |why: String, hint: &str| RpcError::new("refused", why, hint);
    let dest = std::env::current_exe().and_then(|e| e.canonicalize()).map_err(|e| RpcError::internal(e.to_string()))?;
    let shown = dest.display().to_string();
    if let Some(manager) = client::installed_by() {
        let how = match manager {
            "homebrew" => "brew upgrade lumbridge-core",
            "nix" => "update it in your Nix configuration",
            "mise" => "mise upgrade lumbridge-core",
            _ => {
                return Err(refused(
                    format!("{} isn't writable by you", dest.parent().unwrap().display()),
                    "update lumbridge-core the way it was installed (package manager, or sudo)",
                ));
            }
        };
        return Err(refused(format!("{shown} belongs to {manager}"), how));
    }
    let was = format!("lbc {}", env!("CARGO_PKG_VERSION"));
    let now = version_of(from).ok_or_else(|| refused(format!("{} is not a working lbc", from.display()), ""))?;
    if older(&now, &was) && !force {
        return Err(refused(format!("{now} is older than this {was}"), "--force to go back anyway"));
    }
    // Staged beside it, checked, then swapped in: never a half-written lbc.
    crate::platform::swap_in(from, &dest, |staged| version_of(staged).as_deref() == Some(now.as_str())).map_err(
        |e| match (e.kind(), crate::platform::bundle_update_hint()) {
            (std::io::ErrorKind::InvalidData, _) => refused(format!("the copy of {} does not run", from.display()), ""),
            (std::io::ErrorKind::InvalidInput, Some(hint)) => refused(e.to_string(), hint),
            _ => RpcError::internal(format!("cannot replace {shown}: {e}")),
        },
    )?;
    let restarted = match client::connect_existing(paths) {
        Some(stream) => {
            client::call(&stream, "server.restart", json!({}))?;
            wait_closed(&stream);
            true
        }
        None => false,
    };
    let status = call(paths, "server.status", json!({}))?;
    let human = format!(
        "{shown}: {was} -> {now}; core running again (pid {}), terminals kept{}",
        status["pid"],
        crate::platform::privacy_upgrade_notice(&dest)
    );
    Ok(Out {
        json: json!({"path": shown, "was": was, "now": now, "restarted": restarted, "pid": status["pid"]}),
        human,
    })
}
