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
    git(
        &source_repo,
        &["push", "-u", "origin", "HEAD:refs/heads/main"],
    );

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
    assert_eq!(
        count, 0,
        "unsigned push should NOT create an ownership record"
    );
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
    git(
        &source_repo,
        &["push", "-u", "origin", "HEAD:refs/heads/main"],
    );

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

#[test]
fn subsequent_pushes_work_after_ownership_established() {
    let temp = TempDir::new("repobox-server-multi-push-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let expected_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let source_repo = init_working_repo(temp.path().join("source"));
    let repobox_home = setup_repobox_key(temp.path(), private_key, expected_address);

    // First push: signed commit establishes ownership
    write_file(&source_repo.join("README.md"), "# v1\n");
    git(&source_repo, &["add", "README.md"]);
    let commit1 = create_signed_commit(
        &source_repo,
        &repobox_home,
        expected_address,
        "first commit",
    );
    git(&source_repo, &["update-ref", "HEAD", &commit1]);

    let namespace = "0xmulti";
    let repo_name = "incremental";
    let remote = format!("http://{bind}/{namespace}/{repo_name}.git");
    git(&source_repo, &["remote", "add", "origin", &remote]);
    git(
        &source_repo,
        &["push", "-u", "origin", "HEAD:refs/heads/main"],
    );

    // Second push: regular unsigned commit (just a normal git workflow)
    write_file(&source_repo.join("README.md"), "# v2 — updated\n");
    git(&source_repo, &["add", "README.md"]);
    git(&source_repo, &["commit", "-m", "second commit (unsigned)"]);
    git(&source_repo, &["push", "origin", "HEAD:refs/heads/main"]);

    // Clone and verify we get the latest content
    let clone_dir = temp.path().join("clone");
    let clone_str = clone_dir.to_string_lossy().to_string();
    git_in(temp.path(), &["clone", &remote, &clone_str]);
    let readme = std::fs::read_to_string(clone_dir.join("README.md")).unwrap();
    assert_eq!(
        readme, "# v2 — updated\n",
        "clone should have the latest pushed content"
    );

    // Verify git log has both commits
    let log = git_output(&clone_dir, &["log", "--oneline"]);
    assert!(
        log.lines().count() >= 2,
        "should have at least 2 commits, got: {log}"
    );
}

#[test]
fn addressless_push_with_signed_commit_creates_repo() {
    let temp = TempDir::new("repobox-server-addressless-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let expected_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let source_repo = init_working_repo(temp.path().join("source"));
    let repobox_home = setup_repobox_key(temp.path(), private_key, expected_address);

    write_file(&source_repo.join("README.md"), "# address-less push\n");
    git(&source_repo, &["add", "README.md"]);
    let commit_hash = create_signed_commit(
        &source_repo,
        &repobox_home,
        expected_address,
        "address-less initial commit",
    );
    git(&source_repo, &["update-ref", "HEAD", &commit_hash]);

    let repo_name = "addressless-repo";
    let remote = format!("http://{bind}/{repo_name}.git"); // No address prefix
    git(&source_repo, &["remote", "add", "origin", &remote]);
    git(
        &source_repo,
        &["push", "-u", "origin", "HEAD:refs/heads/main"],
    );

    // Repo should be created under the signer's address
    let final_repo = data_dir
        .join(expected_address)
        .join(format!("{repo_name}.git"));
    assert!(
        final_repo.exists(),
        "repo should be created under signer's address"
    );

    // Staging area should be cleaned up
    let staging_repo = data_dir.join("_staging").join(format!("{repo_name}.git"));
    assert!(!staging_repo.exists(), "staging repo should be cleaned up");

    // Check ownership in database
    let db = Connection::open(data_dir.join("repobox.db")).unwrap();
    let owner: String = db
        .query_row(
            "SELECT owner_address FROM repos WHERE address = ?1 AND name = ?2",
            [expected_address, repo_name],
            |row| row.get(0),
        )
        .expect("ownership record should exist");

    assert_eq!(
        owner.to_lowercase(),
        expected_address.to_lowercase(),
        "owner should be the EVM address that signed the first commit"
    );

    // Should be able to clone from the full path
    let clone_dir = temp.path().join("clone");
    let clone_str = clone_dir.to_string_lossy().to_string();
    let full_remote = format!("http://{bind}/{expected_address}/{repo_name}.git");
    git_in(temp.path(), &["clone", &full_remote, &clone_str]);
    let readme = std::fs::read_to_string(clone_dir.join("README.md")).unwrap();
    assert_eq!(readme, "# address-less push\n");
}

#[test]
fn addressless_push_unsigned_is_rejected() {
    let temp = TempDir::new("repobox-server-addressless-unsigned-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let source_repo = init_working_repo(temp.path().join("source"));
    write_file(&source_repo.join("file.txt"), "unsigned content\n");
    git(&source_repo, &["add", "file.txt"]);
    git(&source_repo, &["commit", "-m", "unsigned commit"]);

    let repo_name = "unsigned-addressless";
    let remote = format!("http://{bind}/{repo_name}.git"); // No address prefix
    git(&source_repo, &["remote", "add", "origin", &remote]);

    // Push will complete from git's perspective, but server should reject it
    let output = Command::new("git")
        .current_dir(&source_repo)
        .args(["push", "-u", "origin", "HEAD:refs/heads/main"])
        .output()
        .unwrap();
    let _ = output;

    // No repo should exist anywhere
    let staging_repo = data_dir.join("_staging").join(format!("{repo_name}.git"));
    assert!(!staging_repo.exists(), "staging repo should be cleaned up");

    // No ownership record should exist
    let db = Connection::open(data_dir.join("repobox.db")).unwrap();
    let count: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM repos WHERE name = ?1",
            [repo_name],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 0,
        "unsigned address-less push should NOT create an ownership record"
    );
}

#[test]
fn addressless_clone_returns_404() {
    let temp = TempDir::new("repobox-server-addressless-clone-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    // First create a repo the normal way
    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let expected_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let source_repo = init_working_repo(temp.path().join("source"));
    let repobox_home = setup_repobox_key(temp.path(), private_key, expected_address);

    write_file(&source_repo.join("README.md"), "# test repo\n");
    git(&source_repo, &["add", "README.md"]);
    let commit_hash =
        create_signed_commit(&source_repo, &repobox_home, expected_address, "initial");
    git(&source_repo, &["update-ref", "HEAD", &commit_hash]);

    let repo_name = "clone-test";
    let full_remote = format!("http://{bind}/{expected_address}/{repo_name}.git");
    git(&source_repo, &["remote", "add", "origin", &full_remote]);
    git(
        &source_repo,
        &["push", "-u", "origin", "HEAD:refs/heads/main"],
    );

    // Try to clone using address-less URL - should fail
    let addressless_remote = format!("http://{bind}/{repo_name}.git");
    let clone_dir = temp.path().join("clone");
    let clone_str = clone_dir.to_string_lossy().to_string();
    let output = Command::new("git")
        .current_dir(temp.path())
        .args(["clone", &addressless_remote, &clone_str])
        .output()
        .unwrap();

    assert!(!output.status.success(), "address-less clone should fail");
    assert!(
        !clone_dir.exists(),
        "clone directory should not exist after failed clone"
    );
}

#[test]
fn addressless_subsequent_push_to_existing_repo() {
    let temp = TempDir::new("repobox-server-subsequent-addressless-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let expected_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let source_repo = init_working_repo(temp.path().join("source"));
    let repobox_home = setup_repobox_key(temp.path(), private_key, expected_address);

    // First push: address-less signed commit
    write_file(&source_repo.join("README.md"), "# v1\n");
    git(&source_repo, &["add", "README.md"]);
    let commit1 = create_signed_commit(
        &source_repo,
        &repobox_home,
        expected_address,
        "first commit",
    );
    git(&source_repo, &["update-ref", "HEAD", &commit1]);

    let repo_name = "subsequent-test";
    let addressless_remote = format!("http://{bind}/{repo_name}.git");
    git(
        &source_repo,
        &["remote", "add", "origin", &addressless_remote],
    );
    git(
        &source_repo,
        &["push", "-u", "origin", "HEAD:refs/heads/main"],
    );

    // Verify repo was created under signer's address
    let final_repo = data_dir
        .join(expected_address)
        .join(format!("{repo_name}.git"));
    assert!(
        final_repo.exists(),
        "repo should exist under signer's address"
    );

    // Second push: address-less to the same repo name (should route to existing repo)
    write_file(&source_repo.join("README.md"), "# v2 updated\n");
    git(&source_repo, &["add", "README.md"]);
    git(&source_repo, &["commit", "-m", "second commit"]);
    git(&source_repo, &["push", "origin", "HEAD:refs/heads/main"]);

    // Clone from full path and verify content
    let clone_dir = temp.path().join("clone");
    let clone_str = clone_dir.to_string_lossy().to_string();
    let full_remote = format!("http://{bind}/{expected_address}/{repo_name}.git");
    git_in(temp.path(), &["clone", &full_remote, &clone_str]);
    let readme = std::fs::read_to_string(clone_dir.join("README.md")).unwrap();
    assert_eq!(
        readme, "# v2 updated\n",
        "subsequent push should update the existing repo"
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

fn create_signed_commit(repo: &Path, repobox_home: &Path, address: &str, message: &str) -> String {
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

    git_output_stdin(
        repo,
        &["hash-object", "-t", "commit", "-w", "--stdin"],
        &signed_commit,
    )
}

fn start_server(bind: SocketAddr, data_dir: &Path) -> ServerGuard {
    std::fs::create_dir_all(data_dir).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_repobox-server"))
        .args([
            "--bind",
            &bind.to_string(),
            "--data-dir",
            data_dir.to_string_lossy().as_ref(),
        ])
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
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
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

#[test]
fn x402_payment_required_response() {
    let temp = TempDir::new("repobox-server-x402-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let owner_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let source_repo = init_working_repo(temp.path().join("source"));
    let repobox_home = setup_repobox_key(temp.path(), private_key, owner_address);

    // Create repo with separate x402 config
    let config_content = r#"
groups:
  paid-readers: []

permissions:
  default: deny
  rules:
    - paid-readers read >*
"#;

    let x402_content = r#"
read_price: "1.50"
recipient: "0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048"
network: "base"
"#;

    std::fs::create_dir_all(&source_repo.join(".repobox")).unwrap();
    write_file(
        &source_repo.join(".repobox").join("config.yml"),
        config_content,
    );
    write_file(&source_repo.join(".repobox").join("x402.yml"), x402_content);
    write_file(&source_repo.join("README.md"), "# paid repo\n");

    git(&source_repo, &["add", "."]);
    let commit_hash = create_signed_commit(
        &source_repo,
        &repobox_home,
        owner_address,
        "initial commit with x402",
    );
    git(&source_repo, &["update-ref", "HEAD", &commit_hash]);

    let namespace = owner_address;
    let repo_name = "paid-repo";
    let remote = format!("http://{bind}/{namespace}/{repo_name}.git");
    git(&source_repo, &["remote", "add", "origin", &remote]);
    git(
        &source_repo,
        &["push", "-u", "origin", "HEAD:refs/heads/main"],
    );

    // Try to clone without payment - should get 402
    let clone_dir = temp.path().join("clone");
    let clone_str = clone_dir.to_string_lossy().to_string();
    let output = Command::new("git")
        .current_dir(temp.path())
        .args(["clone", &remote, &clone_str])
        .output()
        .unwrap();

    assert!(!output.status.success(), "clone should fail with 402");
    assert!(!clone_dir.exists(), "clone directory should not exist");

    // Check that git received a 402 response (this will be in stderr)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("402") || stderr.contains("Payment Required"),
        "should receive 402 Payment Required, got: {}",
        stderr
    );

    // Bad/malformed auth header should still surface the same discoverability UX, not an auth prompt
    let clone_dir_bad = temp.path().join("clone-bad-header");
    let clone_bad_str = clone_dir_bad.to_string_lossy().to_string();
    let bad_output = Command::new("git")
        .current_dir(temp.path())
        .env("GIT_TERMINAL_PROMPT", "0")
        .args([
            "-c",
            "http.extraheader=Authorization: Basic !!bad!!",
            "clone",
            &remote,
            &clone_bad_str,
        ])
        .output()
        .unwrap();

    assert!(!bad_output.status.success(), "bad auth header clone should fail");
    assert!(!clone_dir_bad.exists(), "clone should not be created");

    let bad_stderr = String::from_utf8_lossy(&bad_output.stderr);
    assert!(
        bad_stderr.contains("payment required for read access") ||
            bad_stderr.contains("402") ||
            bad_stderr.contains("Payment Required"),
        "bad auth header should keep payment-required guidance, got: {}",
        bad_stderr
    );
    assert!(
        !bad_stderr.contains("could not read Username"),
        "malformed auth must not degrade into username prompt, got: {}",
        bad_stderr
    );
}

#[test]
fn x402_grant_access_endpoint() {
    let temp = TempDir::new("repobox-server-x402-grant-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let owner_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    let payer_address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"; // Different address

    let source_repo = init_working_repo(temp.path().join("source"));
    let repobox_home = setup_repobox_key(temp.path(), private_key, owner_address);

    // Create repo with separate x402 config
    let config_content = r#"
groups:
  paid-readers: []

permissions:
  default: deny
  rules:
    - paid-readers read >*
"#;

    let x402_content = r#"
read_price: "2.00"
recipient: "0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048"
network: "base"
"#;

    std::fs::create_dir_all(&source_repo.join(".repobox")).unwrap();
    write_file(
        &source_repo.join(".repobox").join("config.yml"),
        config_content,
    );
    write_file(&source_repo.join(".repobox").join("x402.yml"), x402_content);
    write_file(
        &source_repo.join("README.md"),
        "# paid repo for grant test\n",
    );

    git(&source_repo, &["add", "."]);
    let commit_hash = create_signed_commit(
        &source_repo,
        &repobox_home,
        owner_address,
        "initial commit with x402",
    );
    git(&source_repo, &["update-ref", "HEAD", &commit_hash]);

    let namespace = owner_address;
    let repo_name = "grant-test-repo";
    let remote = format!("http://{bind}/{namespace}/{repo_name}.git");
    git(&source_repo, &["remote", "add", "origin", &remote]);
    git(
        &source_repo,
        &["push", "-u", "origin", "HEAD:refs/heads/main"],
    );

    // Verify repo was created successfully
    let bare_repo = data_dir.join(namespace).join(format!("{repo_name}.git"));
    assert!(bare_repo.exists(), "repo should exist after push");

    // Call grant-access endpoint
    let grant_url = format!("http://{bind}/{namespace}/{repo_name}.git/x402/grant-access");
    let client = reqwest::blocking::Client::new();
    let payload = serde_json::json!({
        "address": payer_address,
        "tx_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
    });

    let response = client.post(&grant_url).json(&payload).send().unwrap();

    let status = response.status();
    let text = response.text().unwrap();
    assert_eq!(
        status, 200,
        "grant-access should succeed, got {} with body: {}",
        status, text
    );

    // TODO: In a full implementation, we would verify that the payer_address
    // was added to the paid-readers group and can now access the repo
    // For MVP, we're just testing the endpoint responds correctly
}

#[test]
fn x402_info_endpoint() {
    let temp = TempDir::new("repobox-server-x402-info-test").unwrap();
    let data_dir = temp.path().join("data");
    let bind = free_addr();
    let _server = start_server(bind, &data_dir);

    let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let owner_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let source_repo = init_working_repo(temp.path().join("source"));
    let repobox_home = setup_repobox_key(temp.path(), private_key, owner_address);

    let config_content = r#"
groups:
  paid-readers: []

permissions:
  default: deny
  rules:
    - paid-readers read >*
"#;

    let x402_content = r#"
read_price: "2.50"
recipient: "0xDbbAfc2a00175D0cDDFDF130EFc9FA0fb61d2048"
network: "base"
"#;

    std::fs::create_dir_all(&source_repo.join(".repobox")).unwrap();
    write_file(
        &source_repo.join(".repobox").join("config.yml"),
        config_content,
    );
    write_file(&source_repo.join(".repobox").join("x402.yml"), x402_content);
    write_file(
        &source_repo.join("README.md"),
        "# paid repo for info test\n",
    );

    git(&source_repo, &["add", "."]);
    let commit_hash = create_signed_commit(
        &source_repo,
        &repobox_home,
        owner_address,
        "initial commit with x402 info",
    );
    git(&source_repo, &["update-ref", "HEAD", &commit_hash]);

    let namespace = owner_address;
    let repo_name = "info-test-repo";
    let remote = format!("http://{bind}/{namespace}/{repo_name}.git");
    git(&source_repo, &["remote", "add", "origin", &remote]);
    git(
        &source_repo,
        &["push", "-u", "origin", "HEAD:refs/heads/main"],
    );

    // Public preview endpoint should expose pricing metadata without auth
    let info_url = format!("http://{bind}/{namespace}/{repo_name}.git/x402/info");
    let client = reqwest::blocking::Client::new();
    let response = client.get(&info_url).send().unwrap();
    assert_eq!(
        response.status(),
        200,
        "x402 info endpoint should be reachable"
    );

    let body: serde_json::Value = response.json().unwrap();
    assert_eq!(
        body["repository"],
        serde_json::json!(format!("{namespace}/{repo_name}"))
    );
    assert_eq!(body["read_price"], serde_json::json!("2.50"));
    assert_eq!(body["for_sale"], serde_json::json!(true));
}
