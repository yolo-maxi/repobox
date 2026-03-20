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
fn signed_push_and_clone_roundtrip() {
    let temp = TempDir::new("repobox-server-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    // Known test key (Hardhat account #0)
    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let expected_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let source_repo = init_working_repo(temp.path().join("source"));

    // Set up repobox key
    let repobox_home = temp.path().join("repobox-home");
    let keys_dir = repobox_home.join(".repobox").join("keys");
    std::fs::create_dir_all(&keys_dir).unwrap();
    write_file(
        &keys_dir.join(format!("{expected_address}.key")),
        &format!("0x{private_key}"),
    );

    // Create signed commit
    write_file(&source_repo.join("README.md"), "# repo.box\n");
    git(&source_repo, &["add", "README.md"]);
    let signed_commit = create_signed_commit(
        &source_repo,
        &repobox_home,
        expected_address,
        "initial commit",
    );
    git(&source_repo, &["update-ref", "HEAD", &signed_commit]);

    let address = "0xdemo";
    let repo_name = "roundtrip";
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
fn unsigned_push_is_rejected_and_cleaned_up() {
    let temp = TempDir::new("repobox-server-unsigned-test").unwrap();
    let data_dir = temp.path().join("data");
    let address = "0xfeedbeef";
    let repo_name = "unsigned";
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let source_repo = init_working_repo(temp.path().join("source"));
    write_file(&source_repo.join("file.txt"), "unsigned\n");
    git(&source_repo, &["add", "file.txt"]);
    git(&source_repo, &["commit", "-m", "unsigned commit"]);

    let remote = format!("http://{bind}/{address}/{repo_name}.git");
    git(&source_repo, &["remote", "add", "origin", &remote]);

    // Push will "succeed" from git's perspective (objects transferred),
    // but the server should clean up the repo afterwards
    let output = Command::new("git")
        .current_dir(&source_repo)
        .args(["push", "-u", "origin", "HEAD:refs/heads/main"])
        .output()
        .unwrap();
    let _ = output;

    // Bare repo should NOT exist — server cleaned it up
    let bare_repo = data_dir.join(address).join(format!("{repo_name}.git"));
    assert!(!bare_repo.exists(), "unsigned repo should be deleted");

    // No ownership record either
    let db = Connection::open(data_dir.join("repobox.db")).unwrap();
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM repos WHERE address = ?1 AND name = ?2",
            [address, repo_name],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "unsigned push should NOT create an ownership record");
}

#[test]
fn signed_push_establishes_ownership() {
    let temp = TempDir::new("repobox-server-signed-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let expected_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let source_repo = init_working_repo(temp.path().join("source"));
    let repobox_home = setup_repobox_key(temp.path(), private_key, expected_address);

    write_file(&source_repo.join("README.md"), "# signed repo\n");
    git(&source_repo, &["add", "README.md"]);
    let commit_hash = create_signed_commit(
        &source_repo,
        &repobox_home,
        expected_address,
        "signed initial commit",
    );
    git(&source_repo, &["update-ref", "HEAD", &commit_hash]);

    let namespace = expected_address;
    let repo_name = "signed-repo";
    let remote = format!("http://{bind}/{namespace}/{repo_name}.git");
    git(&source_repo, &["remote", "add", "origin", &remote]);
    git(&source_repo, &["push", "-u", "origin", "HEAD:refs/heads/main"]);

    // Check ownership in SQLite
    let db = Connection::open(data_dir.join("repobox.db")).unwrap();
    let owner: String = db
        .query_row(
            "SELECT owner_address FROM repos WHERE address = ?1 AND name = ?2",
            [namespace, repo_name],
            |row| row.get(0),
        )
        .expect("ownership record should exist");

    assert_eq!(
        owner.to_lowercase(),
        expected_address.to_lowercase(),
        "owner should be the EVM address that signed the first commit"
    );
}

fn setup_repobox_key(temp_dir: &Path, private_key: &str, address: &str) -> PathBuf {
    let repobox_home = temp_dir.join("repobox-home");
    let keys_dir = repobox_home.join(".repobox").join("keys");
    std::fs::create_dir_all(&keys_dir).unwrap();
    write_file(
        &keys_dir.join(format!("{address}.key")),
        &format!("0x{private_key}"),
    );
    repobox_home
}

fn create_signed_commit(
    repo: &Path,
    repobox_home: &Path,
    address: &str,
    message: &str,
) -> String {
    let tree_hash = git_output(repo, &["write-tree"]);

    let commit_content = format!(
        "tree {tree_hash}\n\
         author Test <test@test.com> 1234567890 +0000\n\
         committer Test <test@test.com> 1234567890 +0000\n\
         \n\
         {message}\n"
    );

    let sig = repobox::signing::sign(repobox_home, address, commit_content.as_bytes())
        .expect("signing should succeed");
    let sig_hex = hex::encode(&sig);

    let signed_commit = format!(
        "tree {tree_hash}\n\
         author Test <test@test.com> 1234567890 +0000\n\
         committer Test <test@test.com> 1234567890 +0000\n\
         gpgsig {sig_hex}\n\
         \n\
         {message}\n"
    );

    git_output_stdin(repo, &["hash-object", "-t", "commit", "-w", "--stdin"], &signed_commit)
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

fn git_output(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git").current_dir(repo).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn git_output_stdin(repo: &Path, args: &[&str], stdin_data: &str) -> String {
    let mut child = Command::new("git")
        .current_dir(repo)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(stdin_data.as_bytes()).unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
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
