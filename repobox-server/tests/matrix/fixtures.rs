use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;
use tempdir::TempDir;

/// Test fixture builder for repo.box server integration tests
pub struct TestFixture {
    pub temp_dir: TempDir,
    pub repo_path: PathBuf,
    pub config_path: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with a basic git repository
    pub fn new(fixture_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new(&format!("repobox-test-{}", fixture_name))?;
        let repo_path = temp_dir.path().join("repo");
        fs::create_dir_all(&repo_path)?;

        // Initialize git repository
        Self::run_git(&repo_path, &["init"])?;
        Self::run_git(&repo_path, &["config", "user.email", "test@repo.box"])?;
        Self::run_git(&repo_path, &["config", "user.name", "Test User"])?;

        // Create .repobox directory
        let repobox_dir = repo_path.join(".repobox");
        fs::create_dir_all(&repobox_dir)?;
        let config_path = repobox_dir.join("config.yml");

        Ok(Self {
            temp_dir,
            repo_path,
            config_path,
        })
    }

    /// Create fixture for ownership testing
    pub fn ownership_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let mut fixture = Self::new("ownership")?;
        
        // Create basic config with ownership rules
        let config = r#"
groups:
  founders:
    - evm:0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266

permissions:
  default: allow
  rules:
    founders:
      - edit docs/spec.md
      - edit src/core.rs
"#;
        fs::write(&fixture.config_path, config)?;
        
        // Create owned files
        Self::create_file(&fixture.repo_path.join("docs/spec.md"), "# Specification\n")?;
        Self::create_file(&fixture.repo_path.join("src/core.rs"), "// Core module\n")?;
        Self::create_file(&fixture.repo_path.join("README.md"), "# Project\n")?;

        // Initial commit
        Self::run_git(&fixture.repo_path, &["add", "."])?;
        Self::run_git(&fixture.repo_path, &["commit", "-m", "Initial commit"])?;

        Ok(fixture)
    }

    /// Create fixture for append-only testing
    pub fn append_only_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let mut fixture = Self::new("append-only")?;
        
        let config = r#"
permissions:
  default: allow
  rules:
    - append logs/events.log
"#;
        fs::write(&fixture.config_path, config)?;
        
        // Create append-only file
        Self::create_file(&fixture.repo_path.join("logs/events.log"), "2026-03-27 08:00:00 Starting\n")?;
        Self::create_file(&fixture.repo_path.join("README.md"), "# Project\n")?;

        // Initial commit
        Self::run_git(&fixture.repo_path, &["add", "."])?;
        Self::run_git(&fixture.repo_path, &["commit", "-m", "Initial commit"])?;

        Ok(fixture)
    }

    /// Create fixture for signature testing
    pub fn signature_repo() -> Result<Self, Box<dyn std::error::Error>> {
        let mut fixture = Self::new("signatures")?;
        
        let config = r#"
permissions:
  default: deny
  rules:
    - signed push >main
"#;
        fs::write(&fixture.config_path, config)?;
        
        // Create basic file
        Self::create_file(&fixture.repo_path.join("README.md"), "# Project\n")?;

        // Initial commit (unsigned for test setup)
        Self::run_git(&fixture.repo_path, &["add", "."])?;
        Self::run_git(&fixture.repo_path, &["commit", "-m", "Initial commit"])?;

        Ok(fixture)
    }

    /// Create fixture for empty repository
    pub fn empty_repo() -> Result<Self, Box<dyn std::error::Error>> {
        Self::new("empty")
    }

    /// Add a file with specific content
    pub fn add_file(&self, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.repo_path.join(path);
        Self::create_file(&file_path, content)?;
        Self::run_git(&self.repo_path, &["add", path])?;
        Ok(())
    }

    /// Modify an existing file
    pub fn modify_file(&self, path: &str, new_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.repo_path.join(path);
        fs::write(file_path, new_content)?;
        Self::run_git(&self.repo_path, &["add", path])?;
        Ok(())
    }

    /// Delete a file
    pub fn delete_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Self::run_git(&self.repo_path, &["rm", path])?;
        Ok(())
    }

    /// Rename a file
    pub fn rename_file(&self, old_path: &str, new_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        Self::run_git(&self.repo_path, &["mv", old_path, new_path])?;
        Ok(())
    }

    /// Append content to a file
    pub fn append_to_file(&self, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file_path = self.repo_path.join(path);
        let existing = fs::read_to_string(&file_path).unwrap_or_default();
        fs::write(&file_path, format!("{}{}", existing, content))?;
        Self::run_git(&self.repo_path, &["add", path])?;
        Ok(())
    }

    /// Create a commit with the current staged changes
    pub fn commit(&self, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        Self::run_git(&self.repo_path, &["commit", "-m", message])?;
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["rev-parse", "HEAD"])
            .output()?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Create a signed commit using a specific private key
    pub fn commit_signed(&self, message: &str, private_key: &str) -> Result<String, Box<dyn std::error::Error>> {
        // This would integrate with repobox signing logic
        // For now, just create a regular commit with a signature marker
        let signed_message = format!("{}\n\nSigned-off-by: test@repo.box", message);
        Self::run_git(&self.repo_path, &["commit", "-m", &signed_message])?;
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["rev-parse", "HEAD"])
            .output()?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Get the path to the repository
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    /// Get the path to the config file
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Helper to create a file with parent directories
    fn create_file(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    /// Helper to run git commands
    fn run_git(repo_path: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(args)
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Git command failed: git {}\nStderr: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            ).into());
        }

        Ok(())
    }
}

/// Actor configuration for testing different user contexts
pub struct TestActor {
    pub name: String,
    pub email: String,
    pub address: Option<String>,
    pub private_key: Option<String>,
}

impl TestActor {
    pub fn owner() -> Self {
        Self {
            name: "Owner User".to_string(),
            email: "owner@repo.box".to_string(),
            address: Some("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string()),
            private_key: Some("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()),
        }
    }

    pub fn non_owner() -> Self {
        Self {
            name: "Non-Owner User".to_string(),
            email: "user@repo.box".to_string(),
            address: Some("0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string()),
            private_key: Some("59c6995e998f97436b52e15e8b964b36c81bb25e39a6c999e1c40e5bbf456a12".to_string()),
        }
    }

    pub fn unsigned() -> Self {
        Self {
            name: "Unsigned User".to_string(),
            email: "unsigned@repo.box".to_string(),
            address: None,
            private_key: None,
        }
    }

    /// Configure git identity for this actor
    pub fn configure_git(&self, repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Command::new("git")
            .current_dir(repo_path)
            .args(&["config", "user.name", &self.name])
            .output()?;
        
        Command::new("git")
            .current_dir(repo_path)
            .args(&["config", "user.email", &self.email])
            .output()?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_fixture() {
        let fixture = TestFixture::ownership_repo().unwrap();
        
        // Verify files exist
        assert!(fixture.repo_path.join("docs/spec.md").exists());
        assert!(fixture.repo_path.join("src/core.rs").exists());
        assert!(fixture.repo_path.join("README.md").exists());
        assert!(fixture.config_path.exists());

        // Verify it's a git repository
        assert!(fixture.repo_path.join(".git").exists());
    }

    #[test]
    fn test_append_only_fixture() {
        let fixture = TestFixture::append_only_repo().unwrap();
        
        // Verify append-only file exists
        assert!(fixture.repo_path.join("logs/events.log").exists());
        assert!(fixture.config_path.exists());
    }

    #[test]
    fn test_actor_configuration() {
        let owner = TestActor::owner();
        assert!(owner.address.is_some());
        assert!(owner.private_key.is_some());
        
        let non_owner = TestActor::non_owner();
        assert!(non_owner.address.is_some());
        assert_ne!(owner.address, non_owner.address);
        
        let unsigned = TestActor::unsigned();
        assert!(unsigned.address.is_none());
        assert!(unsigned.private_key.is_none());
    }

    #[test]
    fn test_file_operations() {
        let fixture = TestFixture::new("file_ops").unwrap();
        
        // Add file
        fixture.add_file("test.txt", "content").unwrap();
        assert!(fixture.repo_path.join("test.txt").exists());
        
        // Modify file
        fixture.modify_file("test.txt", "new content").unwrap();
        let content = fs::read_to_string(fixture.repo_path.join("test.txt")).unwrap();
        assert_eq!(content, "new content");
        
        // Append to file
        fixture.append_to_file("test.txt", " appended").unwrap();
        let content = fs::read_to_string(fixture.repo_path.join("test.txt")).unwrap();
        assert_eq!(content, "new content appended");
    }
}