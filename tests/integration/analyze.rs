use crate::utils::{analyze_remarks, get_test_data_path, HTMLDir, OutputExt};

#[test]
fn analyze_directory() -> anyhow::Result<()> {
    let dir = tempfile::TempDir::new()?;
    analyze_remarks(
        dir.path(),
        &[
            "--source-dir",
            dir.path().to_str().unwrap(),
            "--output-dir",
            "foo",
            get_test_data_path("remarks-1").to_str().unwrap(),
        ],
    )?
    .assert_ok();
    let dir = HTMLDir::new(&dir.path().join("foo"));
    dir.check_index();

    Ok(())
}

#[test]
fn create_source_file() -> anyhow::Result<()> {
    let data_dir = get_test_data_path("remarks-similarity-join");
    let dir = tempfile::TempDir::new()?;
    let output_dir = "output";

    analyze_remarks(
        dir.path(),
        &[
            "--source-dir",
            data_dir.to_str().unwrap(),
            "--output-dir",
            output_dir,
            data_dir.join("yaml").to_str().unwrap(),
        ],
    )?
    .assert_ok();
    let dir = HTMLDir::new(&dir.path().join(output_dir));
    dir.check_source("src_main.rs.html");
    dir.check_source("src_record.rs.html");

    Ok(())
}
