use std::process::Command;

/// Returns true if the currently used rustc supports `-Zremark-dir`.
pub fn check_remark_dir_support() -> anyhow::Result<bool> {
    let output = Command::new("rustc").arg("-Z").arg("help").output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to execute rustc -Z help"));
    }

    let options = String::from_utf8_lossy(&output.stdout);
    for line in options.lines() {
        let items: Vec<&str> = line.split_whitespace().take(2).map(|v| v.trim()).collect();
        if items.len() != 2 || items[0] != "-Z" {
            continue;
        }
        if items[1] == "remark-dir=val" {
            return Ok(true);
        }
    }

    Ok(false)
}
