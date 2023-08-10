use crate::utils::{cargo_remark, init_cargo_project, OutputExt};
use cargo_remark::remark::{load_remarks_from_dir, Location, Remark, RemarkLoadOptions};
use std::path::Path;

const INLINE_NEVER_SOURCE: &str = r#"
#[inline(never)]
fn foo() {}

fn main() {
    foo();
}
"#;

#[test]
fn test_build_filter() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;
    project.file("src/main.rs", INLINE_NEVER_SOURCE);
    cargo_remark(&project.dir, &["build", "--filter", "NeverInline"])?.assert_ok();
    assert!(load_remarks(&project.remark_dir(), vec![]).is_empty());
    Ok(())
}

#[test]
fn test_generate_remarks() -> anyhow::Result<()> {
    let mut project = init_cargo_project()?;
    project.file("src/main.rs", INLINE_NEVER_SOURCE);
    cargo_remark(&project.dir, &["build", "--filter", ""])?.assert_ok();

    project.default_out_dir().check_index();

    let remark_dir = project.remark_dir();
    assert!(remark_dir.is_dir());
    let remarks = load_remarks_from_dir(
        &remark_dir,
        RemarkLoadOptions {
            external: false,
            source_dir: project.dir.clone(),
            filter_kind: vec![],
            rustc_source_root: None,
        },
        None,
    )?;

    let remark = remarks
        .iter()
        .find(|remark| remark.name == "NeverInline")
        .unwrap();
    assert_eq!(remark.pass, "inline");
    assert_eq!(remark.function.name, "foo::main");

    // Windows doesn't seem to load the debug location correctly
    #[cfg(unix)]
    {
        assert_eq!(
            normalize_location(remark.function.location.as_ref()),
            Some(Location {
                file: "src/main.rs".to_string(),
                line: 6,
                column: 5
            })
        );
    }

    Ok(())
}

fn normalize_location(location: Option<&Location>) -> Option<Location> {
    location.map(|l| Location {
        file: l.file.replace('\\', "/"),
        line: l.line,
        column: l.column,
    })
}

fn load_remarks(path: &Path, filter: Vec<String>) -> Vec<Remark> {
    load_remarks_from_dir(
        path,
        RemarkLoadOptions {
            external: false,
            source_dir: path.to_path_buf(),
            filter_kind: filter,
            rustc_source_root: None,
        },
        None,
    )
    .expect("Cannot load remarks")
}
