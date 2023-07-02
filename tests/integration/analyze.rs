use crate::utils::{cargo_remark, get_test_data_path, OutputDir, OutputExt};

#[test]
fn analyze_directory() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    cargo_remark(
        dir.path(),
        &[
            "analyze",
            "--output-dir",
            "foo",
            get_test_data_path("remarks-1").to_str().unwrap(),
        ],
    )?
    .assert_ok();
    let dir = OutputDir::new(&dir.path().join("foo"));
    dir.check_index();

    Ok(())
}
