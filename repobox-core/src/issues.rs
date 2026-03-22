use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::ConfigError;

/// Issue status in the tracking system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueStatus {
    Open,
    InProgress,
    PendingReview,
    Closed,
    Rejected,
}

/// Issue priority level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssuePriority {
    Critical,
    High,
    Medium,
    Low,
}

/// A bug bounty issue in the tracking system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub title: String,
    pub description: String,
    pub reproduction_steps: String,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub bounty_amount: String,
    pub labels: Vec<String>,
    pub assignee: Option<String>,
    pub claimed_by: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Comment on an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub id: String,
    pub issue_id: String,
    pub author: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Issue assignment to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueAssignment {
    pub issue_id: String,
    pub agent_address: String,
    pub branch_name: String,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    pub status: AssignmentStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssignmentStatus {
    Active,
    Completed,
    Abandoned,
}

/// Issue tracking storage interface.
pub trait IssueStorage {
    fn create_issue(&mut self, issue: Issue) -> Result<(), ConfigError>;
    fn get_issue(&self, id: &str) -> Result<Option<Issue>, ConfigError>;
    fn list_issues(&self, status: Option<IssueStatus>) -> Result<Vec<Issue>, ConfigError>;
    fn update_issue(&mut self, issue: Issue) -> Result<(), ConfigError>;
    fn delete_issue(&mut self, id: &str) -> Result<(), ConfigError>;

    fn add_comment(&mut self, comment: IssueComment) -> Result<(), ConfigError>;
    fn get_comments(&self, issue_id: &str) -> Result<Vec<IssueComment>, ConfigError>;

    fn assign_issue(&mut self, assignment: IssueAssignment) -> Result<(), ConfigError>;
    fn get_assignments(&self, agent_address: &str) -> Result<Vec<IssueAssignment>, ConfigError>;
    fn complete_assignment(&mut self, issue_id: &str, agent_address: &str) -> Result<(), ConfigError>;
}

/// In-memory implementation for testing.
#[derive(Debug, Default)]
pub struct MemoryIssueStorage {
    issues: HashMap<String, Issue>,
    comments: HashMap<String, Vec<IssueComment>>,
    assignments: Vec<IssueAssignment>,
    next_id: u64,
}

impl MemoryIssueStorage {
    pub fn new() -> Self {
        Self::default()
    }

    fn generate_id(&mut self) -> String {
        self.next_id += 1;
        format!("issue_{}", self.next_id)
    }

    /// Create sample issues for testing virtuals integration.
    pub fn create_sample_issues(&mut self) -> Result<Vec<Issue>, ConfigError> {
        let sample_issues = vec![
            Issue {
                id: "1".to_string(),
                title: "Memory leak in parser module".to_string(),
                description: "The parser module has a memory leak when processing large files. Memory usage grows unbounded during parsing.".to_string(),
                reproduction_steps: "1. Create a file larger than 10MB\n2. Run parser on the file\n3. Monitor memory usage\n4. Memory will not be freed after parsing".to_string(),
                status: IssueStatus::Open,
                priority: IssuePriority::High,
                bounty_amount: "50.00".to_string(),
                labels: vec!["bug".to_string(), "memory".to_string(), "parser".to_string()],
                assignee: None,
                claimed_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Issue {
                id: "2".to_string(),
                title: "Configuration validation errors not properly displayed".to_string(),
                description: "When configuration validation fails, error messages are not clearly displayed to the user, making debugging difficult.".to_string(),
                reproduction_steps: "1. Create an invalid config.yml file\n2. Run repobox-check on the file\n3. Observe unclear error message".to_string(),
                status: IssueStatus::Open,
                priority: IssuePriority::Medium,
                bounty_amount: "25.00".to_string(),
                labels: vec!["bug".to_string(), "config".to_string(), "ux".to_string()],
                assignee: None,
                claimed_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Issue {
                id: "3".to_string(),
                title: "Critical security vulnerability in signature verification".to_string(),
                description: "Potential bypass in EVM signature verification under specific conditions. Could allow unauthorized commits.".to_string(),
                reproduction_steps: "Details provided in private security disclosure.".to_string(),
                status: IssueStatus::Open,
                priority: IssuePriority::Critical,
                bounty_amount: "100.00".to_string(),
                labels: vec!["bug".to_string(), "security".to_string(), "critical".to_string()],
                assignee: None,
                claimed_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Issue {
                id: "4".to_string(),
                title: "Add support for custom commit message templates".to_string(),
                description: "Allow repositories to define custom commit message templates that agents must follow.".to_string(),
                reproduction_steps: "Feature request - no reproduction steps needed.".to_string(),
                status: IssueStatus::Open,
                priority: IssuePriority::Low,
                bounty_amount: "10.00".to_string(),
                labels: vec!["feature".to_string(), "enhancement".to_string(), "commit".to_string()],
                assignee: None,
                claimed_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ];

        let mut created_issues = Vec::new();
        for issue in sample_issues {
            self.create_issue(issue.clone())?;
            created_issues.push(issue);
        }

        Ok(created_issues)
    }
}

impl IssueStorage for MemoryIssueStorage {
    fn create_issue(&mut self, issue: Issue) -> Result<(), ConfigError> {
        self.issues.insert(issue.id.clone(), issue);
        Ok(())
    }

    fn get_issue(&self, id: &str) -> Result<Option<Issue>, ConfigError> {
        Ok(self.issues.get(id).cloned())
    }

    fn list_issues(&self, status: Option<IssueStatus>) -> Result<Vec<Issue>, ConfigError> {
        let issues: Vec<Issue> = if let Some(filter_status) = status {
            self.issues
                .values()
                .filter(|issue| issue.status == filter_status)
                .cloned()
                .collect()
        } else {
            self.issues.values().cloned().collect()
        };
        Ok(issues)
    }

    fn update_issue(&mut self, issue: Issue) -> Result<(), ConfigError> {
        if self.issues.contains_key(&issue.id) {
            self.issues.insert(issue.id.clone(), issue);
            Ok(())
        } else {
            Err(ConfigError::InvalidRule(format!("Issue not found: {}", issue.id)))
        }
    }

    fn delete_issue(&mut self, id: &str) -> Result<(), ConfigError> {
        if self.issues.remove(id).is_some() {
            // Also remove comments and assignments
            self.comments.remove(id);
            self.assignments.retain(|a| a.issue_id != id);
            Ok(())
        } else {
            Err(ConfigError::InvalidRule(format!("Issue not found: {}", id)))
        }
    }

    fn add_comment(&mut self, comment: IssueComment) -> Result<(), ConfigError> {
        self.comments
            .entry(comment.issue_id.clone())
            .or_insert_with(Vec::new)
            .push(comment);
        Ok(())
    }

    fn get_comments(&self, issue_id: &str) -> Result<Vec<IssueComment>, ConfigError> {
        Ok(self.comments
            .get(issue_id)
            .cloned()
            .unwrap_or_default())
    }

    fn assign_issue(&mut self, assignment: IssueAssignment) -> Result<(), ConfigError> {
        // Check if issue exists
        if !self.issues.contains_key(&assignment.issue_id) {
            return Err(ConfigError::InvalidRule(format!("Issue not found: {}", assignment.issue_id)));
        }

        // Update issue assignee
        if let Some(issue) = self.issues.get_mut(&assignment.issue_id) {
            issue.assignee = Some(assignment.agent_address.clone());
            issue.claimed_by = Some(assignment.agent_address.clone());
            issue.status = IssueStatus::InProgress;
            issue.updated_at = chrono::Utc::now();
        }

        self.assignments.push(assignment);
        Ok(())
    }

    fn get_assignments(&self, agent_address: &str) -> Result<Vec<IssueAssignment>, ConfigError> {
        let assignments: Vec<IssueAssignment> = self.assignments
            .iter()
            .filter(|a| a.agent_address == agent_address)
            .cloned()
            .collect();
        Ok(assignments)
    }

    fn complete_assignment(&mut self, issue_id: &str, agent_address: &str) -> Result<(), ConfigError> {
        // Update assignment status
        for assignment in &mut self.assignments {
            if assignment.issue_id == issue_id && assignment.agent_address == agent_address {
                assignment.status = AssignmentStatus::Completed;
            }
        }

        // Update issue status
        if let Some(issue) = self.issues.get_mut(issue_id) {
            issue.status = IssueStatus::Closed;
            issue.updated_at = chrono::Utc::now();
        }

        Ok(())
    }
}

/// Create an issue comment from an agent.
pub fn create_agent_comment(
    issue_id: &str,
    agent_address: &str,
    content: &str,
) -> IssueComment {
    IssueComment {
        id: format!("comment_{}_{}", issue_id, chrono::Utc::now().timestamp()),
        issue_id: issue_id.to_string(),
        author: agent_address.to_string(),
        content: content.to_string(),
        created_at: chrono::Utc::now(),
    }
}

/// Create an issue assignment when an agent claims an issue.
pub fn create_issue_assignment(
    issue_id: &str,
    agent_address: &str,
    branch_name: &str,
) -> IssueAssignment {
    IssueAssignment {
        issue_id: issue_id.to_string(),
        agent_address: agent_address.to_string(),
        branch_name: branch_name.to_string(),
        assigned_at: chrono::Utc::now(),
        status: AssignmentStatus::Active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_retrieve_issue() {
        let mut storage = MemoryIssueStorage::new();
        
        let issue = Issue {
            id: "test-1".to_string(),
            title: "Test issue".to_string(),
            description: "A test issue".to_string(),
            reproduction_steps: "Steps to reproduce".to_string(),
            status: IssueStatus::Open,
            priority: IssuePriority::Medium,
            bounty_amount: "25.00".to_string(),
            labels: vec!["test".to_string()],
            assignee: None,
            claimed_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        storage.create_issue(issue.clone()).unwrap();
        let retrieved = storage.get_issue("test-1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test issue");
    }

    #[test]
    fn test_issue_assignment() {
        let mut storage = MemoryIssueStorage::new();
        
        let issue = Issue {
            id: "assign-test".to_string(),
            title: "Assignment test".to_string(),
            description: "Test issue assignment".to_string(),
            reproduction_steps: "Steps".to_string(),
            status: IssueStatus::Open,
            priority: IssuePriority::High,
            bounty_amount: "50.00".to_string(),
            labels: vec!["test".to_string()],
            assignee: None,
            claimed_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        storage.create_issue(issue).unwrap();

        let assignment = create_issue_assignment(
            "assign-test",
            "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00",
            "agent/AAc050Ca4FB723bE066E7C12290EE965C84a4a00/fix-assign-test",
        );

        storage.assign_issue(assignment).unwrap();

        // Check issue was updated
        let updated_issue = storage.get_issue("assign-test").unwrap().unwrap();
        assert_eq!(updated_issue.status, IssueStatus::InProgress);
        assert_eq!(updated_issue.assignee, Some("0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00".to_string()));

        // Check assignment was created
        let assignments = storage.get_assignments("0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00").unwrap();
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].status, AssignmentStatus::Active);
    }

    #[test]
    fn test_complete_assignment() {
        let mut storage = MemoryIssueStorage::new();
        
        let issue = Issue {
            id: "complete-test".to_string(),
            title: "Completion test".to_string(),
            description: "Test completing assignment".to_string(),
            reproduction_steps: "Steps".to_string(),
            status: IssueStatus::InProgress,
            priority: IssuePriority::Medium,
            bounty_amount: "25.00".to_string(),
            labels: vec!["test".to_string()],
            assignee: Some("0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00".to_string()),
            claimed_by: Some("0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        storage.create_issue(issue).unwrap();

        let assignment = create_issue_assignment(
            "complete-test",
            "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00",
            "agent/AAc050Ca4FB723bE066E7C12290EE965C84a4a00/fix-complete-test",
        );

        storage.assign_issue(assignment).unwrap();
        storage.complete_assignment("complete-test", "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00").unwrap();

        // Check issue status
        let completed_issue = storage.get_issue("complete-test").unwrap().unwrap();
        assert_eq!(completed_issue.status, IssueStatus::Closed);

        // Check assignment status
        let assignments = storage.get_assignments("0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00").unwrap();
        assert_eq!(assignments[0].status, AssignmentStatus::Completed);
    }

    #[test]
    fn test_create_sample_issues() {
        let mut storage = MemoryIssueStorage::new();
        let issues = storage.create_sample_issues().unwrap();
        
        assert_eq!(issues.len(), 4);
        
        // Verify all issues were stored
        for issue in &issues {
            let retrieved = storage.get_issue(&issue.id).unwrap();
            assert!(retrieved.is_some());
        }
        
        // Test filtering by status
        let open_issues = storage.list_issues(Some(IssueStatus::Open)).unwrap();
        assert_eq!(open_issues.len(), 4);
        
        let closed_issues = storage.list_issues(Some(IssueStatus::Closed)).unwrap();
        assert_eq!(closed_issues.len(), 0);
    }

    #[test]
    fn test_issue_comments() {
        let mut storage = MemoryIssueStorage::new();
        
        let comment = create_agent_comment(
            "test-issue",
            "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00",
            "Working on this issue",
        );

        storage.add_comment(comment).unwrap();
        
        let comments = storage.get_comments("test-issue").unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].content, "Working on this issue");
        assert_eq!(comments[0].author, "0xAAc050Ca4FB723bE066E7C12290EE965C84a4a00");
    }
}