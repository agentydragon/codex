use crate::protocol::ReviewDecision;
use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecHistoryEntry {
    pub id: String,
    pub session_id: Uuid,
    pub timestamp: SystemTime,
    pub command: Vec<String>,
    pub working_dir: String,
    pub approval_requested: bool,
    pub approval_decision: Option<ReviewDecision>,
    #[serde(default)]
    pub auto_approved: bool,
    /// Automatically denied by auto-allow predicates or safety rejection.
    #[serde(default)]
    pub auto_denied: bool,
    /// Reason for auto-denial (e.g. predicate name or rejection reason).
    #[serde(default)]
    pub auto_denied_reason: Option<String>,
    /// User approved via prompt.
    #[serde(default)]
    pub user_approved: bool,
    /// User denied via prompt.
    #[serde(default)]
    pub user_denied: bool,
    #[serde(default)]
    pub execution_started: bool,
    pub execution_result: Option<ExecResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

pub struct ExecHistory {
    path: PathBuf,
}

impl ExecHistory {
    pub fn new(home_dir: &Path) -> Self {
        let path = home_dir.join("exec_history.jsonl");
        Self { path }
    }

    pub fn append_entry(&self, entry: &ExecHistoryEntry) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let json = serde_json::to_string(entry)?;
        // Ensure whole record is written atomically
        use fs2::FileExt;
        file.lock_exclusive()?;
        write!(file, "{}\n", json)?;
        file.sync_all()?;
        // Unlock when file is dropped

        Ok(())
    }

    pub fn read_all(&self) -> anyhow::Result<Vec<ExecHistoryEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<ExecHistoryEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("Failed to parse exec history entry: {}", e),
            }
        }

        Ok(entries)
    }

    pub fn query(&self, filter: ExecHistoryFilter) -> anyhow::Result<Vec<ExecHistoryEntry>> {
        let entries = self.read_all()?;

        Ok(entries
            .into_iter()
            .filter(|entry| {
                // Apply filters
                if let Some(approved_only) = filter.approved_only {
                    let is_approved = entry.auto_approved
                        || matches!(
                            entry.approval_decision,
                            Some(ReviewDecision::Approved)
                                | Some(ReviewDecision::ApprovedForSession)
                        );
                    if approved_only != is_approved {
                        return false;
                    }
                }

                if let Some(denied_only) = filter.denied_only {
                    let is_denied = matches!(
                        entry.approval_decision,
                        Some(ReviewDecision::Denied) | Some(ReviewDecision::Abort)
                    );
                    if denied_only != is_denied {
                        return false;
                    }
                }

                if let Some(session_id) = &filter.session_id {
                    if &entry.session_id != session_id {
                        return false;
                    }
                }

                if let Some(contains) = &filter.contains {
                    let command_str = entry.command.join(" ").to_lowercase();
                    if !command_str.contains(&contains.to_lowercase()) {
                        return false;
                    }
                }

                true
            })
            .collect())
    }

    pub fn get_last_n(&self, n: usize) -> anyhow::Result<Vec<ExecHistoryEntry>> {
        let entries = self.read_all()?;
        let start = entries.len().saturating_sub(n);
        Ok(entries[start..].to_vec())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ExecHistoryFilter {
    pub approved_only: Option<bool>,
    pub denied_only: Option<bool>,
    pub session_id: Option<Uuid>,
    pub contains: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_exec_history() {
        let temp_dir = TempDir::new().unwrap();
        let history = ExecHistory::new(temp_dir.path());

        // Test empty history
        assert_eq!(history.read_all().unwrap().len(), 0);

        // Add an entry
        let entry = ExecHistoryEntry {
            id: "test-1".to_string(),
            session_id: Uuid::new_v4(),
            timestamp: SystemTime::now(),
            command: vec!["ls".to_string(), "-la".to_string()],
            working_dir: "/tmp".to_string(),
            approval_requested: true,
            approval_decision: Some(ReviewDecision::Approved),
            auto_approved: false,
            auto_denied: false,
            auto_denied_reason: None,
            user_approved: false,
            user_denied: false,
            execution_started: true,
            execution_result: Some(ExecResult {
                exit_code: Some(0),
                duration_ms: 100,
                error: None,
            }),
        };

        history.append_entry(&entry).unwrap();

        // Read it back
        let entries = history.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command, vec!["ls", "-la"]);
    }
}
