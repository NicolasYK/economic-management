use std::process::Command;
fn main() {
    let sha = std::env::var("BUILD_COMMIT")
        .ok()
        .or_else(|| {
            Command::new("git")
                .args(["rev-parse", "--short=12", "HEAD"])
                .output()
                .ok()
                .filter(|o| o.status.success())
                .and_then(|o| String::from_utf8(o.stdout).ok())
        })
        .unwrap_or_else(|| "source".into());
    println!("cargo:rustc-env=BUILD_COMMIT={}", sha.trim());
    println!("cargo:rerun-if-changed=.git/HEAD");
    if let Ok(reference) = Command::new("git")
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()
        && let Ok(reference) = String::from_utf8(reference.stdout)
        && let Ok(path) = Command::new("git")
            .args(["rev-parse", "--git-path", reference.trim()])
            .output()
        && let Ok(path) = String::from_utf8(path.stdout)
    {
        println!("cargo:rerun-if-changed={}", path.trim());
    }
    println!("cargo:rerun-if-env-changed=BUILD_COMMIT");
}
