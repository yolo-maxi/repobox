use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use rusqlite::Connection;
use tempdir::TempDir;

struct ServerGuard {
    child: Child,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn repo_creation_on_first_push_and_clone_after_push() {
    let temp = TempDir::new("repobox-server-test").unwrap();
    let data_dir = temp.path().join("data");
    let address = "0xabc123";
    let repo_name = "demo";
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let source_repo = init_working_repo(temp.path().join("source"));
    write_file(&source_repo.join("README.md"), "# repo.box\n");
    git(&source_repo, &["add", "README.md"]);
    git(&source_repo, &["commit", "-m", "initial commit"]);

    let remote = format!("http://{bind}/{address}/{repo_name}.git");
    git(&source_repo, &["remote", "add", "origin", &remote]);
    git(&source_repo, &["push", "-u", "origin", "HEAD:refs/heads/main"]);

    let bare_repo = data_dir.join(address).join(format!("{repo_name}.git"));
    assert!(bare_repo.exists(), "expected bare repo to be created");
    assert!(bare_repo.join("hooks").join("pre-receive").exists());

    let clone_dir = temp.path().join("clone");
    let clone_str = clone_dir.to_string_lossy().to_string();
    git_in(temp.path(), &["clone", &remote, &clone_str]);
    assert!(clone_dir.exists(), "clone dir should exist after clone");
    let cloned_readme = std::fs::read_to_string(clone_dir.join("README.md")).unwrap();
    assert_eq!(cloned_readme, "# repo.box\n");
}

#[test]
fn sqlite_ownership_record_is_created() {
    let temp = TempDir::new("repobox-server-db-test").unwrap();
    let data_dir = temp.path().join("data");
    let address = "0xfeedbeef";
    let repo_name = "owners";
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let source_repo = init_working_repo(temp.path().join("source"));
    write_file(&source_repo.join("owned.txt"), "owned\n");
    git(&source_repo, &["add", "owned.txt"]);
    git(&source_repo, &["commit", "-m", "ownership"]);

    let remote = format!("http://{bind}/{address}/{repo_name}.git");
    git(&source_repo, &["remote", "add", "origin", &remote]);
    git(&source_repo, &["push", "-u", "origin", "HEAD:refs/heads/main"]);

    let db = Connection::open(data_dir.join("repobox.db")).unwrap();
    let row = db
        .query_row(
            "SELECT owner_address FROM repos WHERE address = ?1 AND name = ?2",
            [address, repo_name],
            |row| row.get::<_, String>(0),
        )
        .unwrap();
    assert_eq!(row, address);
}

fn start_server(bind: SocketAddr, data_dir: &Path) -> ServerGuard {
    std::fs::create_dir_all(data_dir).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_repobox-server"))
        .args(["--bind", &bind.to_string(), "--data-dir", data_dir.to_string_lossy().as_ref()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    wait_for_port(bind);
    ServerGuard { child }
}

fn init_working_repo(path: PathBuf) -> PathBuf {
    std::fs::create_dir_all(&path).unwrap();
    let path_str = path.to_string_lossy().to_string();
    git_in(path.parent().unwrap(), &["init", &path_str]);
    git(&path, &["config", "user.name", "repo.box"]);
    git(&path, &["config", "user.email", "repobox@example.com"]);
    path
}

fn write_file(path: &Path, contents: &str) {
    std::fs::write(path, contents).unwrap();
}

fn git(repo: &Path, args: &[&str]) {
    git_in(repo, args);
}

fn git_in(cwd: &Path, args: &[&str]) {
    let output = Command::new("git").current_dir(cwd).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn free_addr() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    addr
}

fn wait_for_port(bind: SocketAddr) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(10) {
        if std::net::TcpStream::connect(bind).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("server did not start on {bind}");
}
