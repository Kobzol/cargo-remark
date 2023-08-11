use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

pub fn cargo_remark(dir: &Path, args: &[&str]) -> anyhow::Result<Output> {
    let mut command = Command::new("cargo");
    command.arg("remark");
    for arg in args {
        command.arg(arg);
    }
    command.current_dir(dir);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());

    let path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{}", get_target_dir().display(), path);

    command.env("PATH", path);

    let child = command.spawn()?;
    Ok(child.wait_with_output()?)
}

pub fn analyze_remarks(dir: &Path, args: &[&str]) -> anyhow::Result<Output> {
    let mut command = Command::new("analyze-remarks");
    for arg in args {
        command.arg(arg);
    }
    command.current_dir(dir);
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());

    let path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{}", get_target_dir().display(), path);

    command.env("PATH", path);

    let child = command.spawn()?;
    Ok(child.wait_with_output()?)
}

pub trait OutputExt {
    fn assert_ok(self) -> Self;
    fn assert_error(self) -> Self;

    fn stdout(&self) -> String;
    fn stderr(&self) -> String;
}

impl OutputExt for Output {
    fn assert_ok(self) -> Self {
        if !self.status.success() {
            eprintln!("Stdout: {}", self.stdout());
            eprintln!("Stderr: {}", self.stderr());
            panic!("Output was not successful");
        }
        self
    }

    fn assert_error(self) -> Self {
        if self.status.success() {
            eprintln!("Stdout: {}", self.stdout());
            eprintln!("Stderr: {}", self.stderr());
            panic!("Output was successful");
        }
        self
    }

    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }

    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.stderr).to_string()
    }
}

fn get_target_dir() -> PathBuf {
    let mut target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    target_dir.push("target");
    target_dir.push("debug");
    target_dir
}

pub fn get_test_data_path<P: AsRef<Path>>(path: P) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(path.as_ref())
}

pub struct HTMLDir {
    dir: PathBuf,
}

impl HTMLDir {
    pub fn new(dir: &Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }

    pub fn check_index(&self) {
        self.check_exists("index.html");
    }

    pub fn check_source(&self, file: &str) {
        self.check_exists(Path::new("src").join(file));
    }

    fn check_exists<P: AsRef<Path>>(&self, path: P) {
        let path = self.dir.join(path.as_ref());
        assert!(path.is_file());
        assert!(path.metadata().unwrap().len() > 0);
    }
}

pub struct CargoProject {
    pub dir: PathBuf,
    _tempdir: TempDir,
}

impl CargoProject {
    pub fn path<P: Into<PathBuf>>(&self, path: P) -> PathBuf {
        let path = path.into();
        self.dir.join(path)
    }

    pub fn file<P: AsRef<Path>>(&mut self, path: P, code: &str) -> &mut Self {
        let path = self.path(path.as_ref());
        std::fs::write(path, code).expect("Could not write project file");
        self
    }

    pub fn remark_dir(&self) -> PathBuf {
        self.path("target/remarks/yaml")
    }

    pub fn default_out_dir(&self) -> HTMLDir {
        self.out_dir(&self.path("target/remarks/web"))
    }

    fn out_dir(&self, path: &Path) -> HTMLDir {
        HTMLDir::new(path)
    }
}

impl Drop for CargoProject {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Do not delete the directory if an error has occurred
            let path = std::mem::replace(&mut self._tempdir, TempDir::new().unwrap()).into_path();
            eprintln!("Directory of failed test located at: {}", path.display());
        }
    }
}

pub fn init_cargo_project() -> anyhow::Result<CargoProject> {
    let dir = tempfile::tempdir()?;

    let name = "foo";
    let status = Command::new("cargo")
        .args(["new", "--bin", name])
        .current_dir(dir.path())
        .status()?;
    assert!(status.success());

    let path = dir.path().join(name);

    println!("Created Cargo project {} at {}", name, path.display());

    Ok(CargoProject {
        dir: path,
        _tempdir: dir,
    })
}
