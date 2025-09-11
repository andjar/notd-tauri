// Task Parser
// Phase 1: Extract and validate task information from block content

use regex::Regex;
use serde::{Serialize, Deserialize};
use chrono::Utc;

/// Task status enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    Doing,
    Done,
    Waiting,
    Cancelled,
}

impl TaskStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "TODO" => Some(TaskStatus::Todo),
            "DOING" => Some(TaskStatus::Doing),
            "DONE" => Some(TaskStatus::Done),
            "WAITING" => Some(TaskStatus::Waiting),
            "CANCELLED" => Some(TaskStatus::Cancelled),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "TODO",
            TaskStatus::Doing => "DOING", 
            TaskStatus::Done => "DONE",
            TaskStatus::Waiting => "WAITING",
            TaskStatus::Cancelled => "CANCELLED",
        }
    }
    
    pub fn is_completed(&self) -> bool {
        matches!(self, TaskStatus::Done | TaskStatus::Cancelled)
    }
    
    pub fn is_active(&self) -> bool {
        matches!(self, TaskStatus::Todo | TaskStatus::Doing | TaskStatus::Waiting)
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Extract task status from content (if it's a task block)
pub fn extract_task_status(content: &str) -> Option<TaskStatus> {
    let task_regex = Regex::new(r"^(TODO|DOING|DONE|WAITING|CANCELLED)\s+").unwrap();
    
    if let Some(cap) = task_regex.captures(content.trim()) {
        TaskStatus::from_str(&cap[1])
    } else {
        None
    }
}

/// Check if content has task markers
pub fn has_task_markers(content: &str) -> bool {
    let task_regex = Regex::new(r"^(TODO|DOING|DONE|WAITING|CANCELLED)\s+").unwrap();
    task_regex.is_match(content.trim())
}

/// Extract task content without the status prefix
pub fn extract_task_content(content: &str) -> String {
    let task_regex = Regex::new(r"^(TODO|DOING|DONE|WAITING|CANCELLED)\s+(.*)").unwrap();
    
    if let Some(cap) = task_regex.captures(content.trim()) {
        cap[2].to_string()
    } else {
        content.to_string()
    }
}

/// Convert task status to different status
pub fn change_task_status(content: &str, new_status: TaskStatus) -> String {
    let task_content = extract_task_content(content);
    format!("{} {}", new_status.as_str(), task_content)
}

/// Extract task priority from content (looks for priority:: property or #priority tags)
pub fn extract_task_priority(content: &str) -> Option<TaskPriority> {
    // First check for priority:: property
    if let Some(priority_str) = crate::parser::properties::get_property_value(content, "priority") {
        return TaskPriority::from_str(&priority_str);
    }
    
    // Then check for priority tags
    let priority_regex = Regex::new(r"#(high|medium|low)\b").unwrap();
    if let Some(cap) = priority_regex.captures(content) {
        return TaskPriority::from_str(&cap[1]);
    }
    
    None
}

/// Extract due date from task content
pub fn extract_due_date(content: &str) -> Option<String> {
    // Check for due:: property
    if let Some(due_str) = crate::parser::properties::get_property_value(content, "due") {
        return Some(due_str);
    }
    
    // Check for due date patterns in content
    let due_patterns = [
        r"due:?\s+(\d{4}-\d{2}-\d{2})",           // due: 2024-01-15
        r"@(\d{4}-\d{2}-\d{2})",                  // @2024-01-15
        r"due\s+(today|tomorrow|yesterday)",       // due tomorrow
        r"@(today|tomorrow|yesterday)",           // @tomorrow
    ];
    
    for pattern in &due_patterns {
        let regex = Regex::new(pattern).unwrap();
        if let Some(cap) = regex.captures(content) {
            return Some(cap[1].to_string());
        }
    }
    
    None
}

/// Extract tags from task content
pub fn extract_task_tags(content: &str) -> Vec<String> {
    let tag_regex = Regex::new(r"#([a-zA-Z0-9_]+)").unwrap();
    
    tag_regex.captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Parse task metadata from content
pub fn parse_task_metadata(content: &str) -> Option<TaskMetadata> {
    if let Some(status) = extract_task_status(content) {
        Some(TaskMetadata {
            status,
            content: extract_task_content(content),
            priority: extract_task_priority(content),
            due_date: extract_due_date(content),
            tags: extract_task_tags(content),
        })
    } else {
        None
    }
}

/// Check if task is overdue (simplified check)
pub fn is_task_overdue(due_date: &str) -> bool {
    // Simple date comparison - in a real implementation, you'd use proper date parsing
    let today = Utc::now().naive_utc().date().to_string();
    
    match due_date {
        "overdue" => true, // Assuming this is pre-calculated
        date_str if date_str.len() == 10 => {
            // Basic check for YYYY-MM-DD format
            date_str < today.as_str()
        }
        _ => false,
    }
}

/// Generate task summary
pub fn generate_task_summary(tasks: &[TaskMetadata]) -> TaskSummary {
    let total = tasks.len();
    let todo = tasks.iter().filter(|t| t.status == TaskStatus::Todo).count();
    let doing = tasks.iter().filter(|t| t.status == TaskStatus::Doing).count();
    let done = tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
    let waiting = tasks.iter().filter(|t| t.status == TaskStatus::Waiting).count();
    let cancelled = tasks.iter().filter(|t| t.status == TaskStatus::Cancelled).count();
    
    let overdue = tasks.iter()
        .filter(|t| t.due_date.as_ref().map_or(false, |d| is_task_overdue(d)))
        .count();
    
    TaskSummary {
        total,
        todo,
        doing,
        done,
        waiting,
        cancelled,
        overdue,
        completion_rate: if total > 0 { 
            (done as f32 / total as f32) * 100.0 
        } else { 
            0.0 
        },
    }
}

/// Validate task format
pub fn validate_task_format(content: &str) -> TaskValidation {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    
    if let Some(status) = extract_task_status(content) {
        let task_content = extract_task_content(content);
        
        // Check for empty task content
        if task_content.trim().is_empty() {
            warnings.push("Task has no description".to_string());
        }
        
        // Check for very long task content
        if task_content.len() > 500 {
            warnings.push("Task description is very long".to_string());
        }
        
        // Check for malformed due dates
        if let Some(due_date) = extract_due_date(content) {
            if !is_valid_due_date(&due_date) {
                warnings.push(format!("Potentially invalid due date: {}", due_date));
            }
        }
        
        // Check for duplicate status markers
        let status_count = content.matches(&format!("{} ", status.as_str())).count();
        if status_count > 1 {
            errors.push("Multiple status markers found".to_string());
        }
        
    } else if content.trim().to_uppercase().starts_with("TODO") || 
              content.trim().to_uppercase().starts_with("DONE") ||
              content.trim().to_uppercase().starts_with("DOING") {
        // Looks like a task but doesn't match pattern
        errors.push("Malformed task status - missing space after status?".to_string());
    }
    
    TaskValidation {
        is_valid: errors.is_empty(),
        warnings,
        errors,
    }
}

/// Simple due date validation
fn is_valid_due_date(due_date: &str) -> bool {
    match due_date {
        "today" | "tomorrow" | "yesterday" => true,
        _ => {
            // Check YYYY-MM-DD format
            let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
            date_regex.is_match(due_date)
        }
    }
}

// ================================
// Data Structures
// ================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskPriority {
    High,
    Medium,
    Low,
}

impl TaskPriority {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "high" | "urgent" | "critical" => Some(TaskPriority::High),
            "medium" | "normal" => Some(TaskPriority::Medium),
            "low" | "minor" => Some(TaskPriority::Low),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::High => "high",
            TaskPriority::Medium => "medium",
            TaskPriority::Low => "low",
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub status: TaskStatus,
    pub content: String,
    pub priority: Option<TaskPriority>,
    pub due_date: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub total: usize,
    pub todo: usize,
    pub doing: usize,
    pub done: usize,
    pub waiting: usize,
    pub cancelled: usize,
    pub overdue: usize,
    pub completion_rate: f32,
}

#[derive(Debug, Clone)]
pub struct TaskValidation {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_task_status() {
        assert_eq!(extract_task_status("TODO Buy groceries"), Some(TaskStatus::Todo));
        assert_eq!(extract_task_status("DONE Complete project"), Some(TaskStatus::Done));
        assert_eq!(extract_task_status("DOING Work on task"), Some(TaskStatus::Doing));
        assert_eq!(extract_task_status("Regular text"), None);
    }
    
    #[test]
    fn test_has_task_markers() {
        assert!(has_task_markers("TODO Something"));
        assert!(has_task_markers("DONE Something else"));
        assert!(!has_task_markers("Regular content"));
        assert!(!has_task_markers("TODO")); // Missing space
    }
    
    #[test]
    fn test_extract_task_content() {
        assert_eq!(extract_task_content("TODO Buy milk and bread"), "Buy milk and bread");
        assert_eq!(extract_task_content("DONE Finished the report"), "Finished the report");
        assert_eq!(extract_task_content("Regular content"), "Regular content");
    }
    
    #[test]
    fn test_change_task_status() {
        let content = "TODO Write documentation";
        let updated = change_task_status(content, TaskStatus::Done);
        assert_eq!(updated, "DONE Write documentation");
    }
    
    #[test]
    fn test_extract_task_priority() {
        let content_with_property = "TODO Important task\npriority:: high";
        assert_eq!(extract_task_priority(content_with_property), Some(TaskPriority::High));
        
        let content_with_tag = "TODO Task with #high priority";
        assert_eq!(extract_task_priority(content_with_tag), Some(TaskPriority::High));
        
        let content_without_priority = "TODO Regular task";
        assert_eq!(extract_task_priority(content_without_priority), None);
    }
    
    #[test]
    fn test_extract_due_date() {
        let content_with_property = "TODO Task\ndue:: 2024-01-15";
        assert_eq!(extract_due_date(content_with_property), Some("2024-01-15".to_string()));
        
        let content_with_at_symbol = "TODO Task @2024-01-15";
        assert_eq!(extract_due_date(content_with_at_symbol), Some("2024-01-15".to_string()));
        
        let content_with_relative = "TODO Task due tomorrow";
        assert_eq!(extract_due_date(content_with_relative), Some("tomorrow".to_string()));
        
        let content_without_due = "TODO Regular task";
        assert_eq!(extract_due_date(content_without_due), None);
    }
    
    #[test]
    fn test_extract_task_tags() {
        let content = "TODO Task with #work and #urgent tags";
        let tags = extract_task_tags(content);
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"work".to_string()));
        assert!(tags.contains(&"urgent".to_string()));
    }
    
    #[test]
    fn test_parse_task_metadata() {
        let content = "TODO Important task #work #urgent\npriority:: high\ndue:: tomorrow";
        let metadata = parse_task_metadata(content).unwrap();
        
        assert_eq!(metadata.status, TaskStatus::Todo);
        assert!(metadata.content.contains("Important task"));
        assert_eq!(metadata.priority, Some(TaskPriority::High));
        assert_eq!(metadata.due_date, Some("tomorrow".to_string()));
        assert_eq!(metadata.tags.len(), 2);
    }
    
    #[test]
    fn test_task_status_methods() {
        assert!(TaskStatus::Done.is_completed());
        assert!(TaskStatus::Cancelled.is_completed());
        assert!(!TaskStatus::Todo.is_completed());
        
        assert!(TaskStatus::Todo.is_active());
        assert!(TaskStatus::Doing.is_active());
        assert!(!TaskStatus::Done.is_active());
    }
    
    #[test]
    fn test_priority_parsing() {
        assert_eq!(TaskPriority::from_str("high"), Some(TaskPriority::High));
        assert_eq!(TaskPriority::from_str("urgent"), Some(TaskPriority::High));
        assert_eq!(TaskPriority::from_str("medium"), Some(TaskPriority::Medium));
        assert_eq!(TaskPriority::from_str("low"), Some(TaskPriority::Low));
        assert_eq!(TaskPriority::from_str("invalid"), None);
    }
    
    #[test]
    fn test_is_task_overdue() {
        assert!(is_task_overdue("yesterday"));
        assert!(!is_task_overdue("today"));
        assert!(!is_task_overdue("tomorrow"));
        
        // This test would need to be updated based on current date for real scenarios
        // For now, we'll just test the function exists and handles basic cases
    }
    
    #[test]
    fn test_generate_task_summary() {
        let tasks = vec![
            TaskMetadata {
                status: TaskStatus::Todo,
                content: "Task 1".to_string(),
                priority: None,
                due_date: None,
                tags: vec![],
            },
            TaskMetadata {
                status: TaskStatus::Done,
                content: "Task 2".to_string(),
                priority: None,
                due_date: None,
                tags: vec![],
            },
            TaskMetadata {
                status: TaskStatus::Doing,
                content: "Task 3".to_string(),
                priority: None,
                due_date: None,
                tags: vec![],
            },
        ];
        
        let summary = generate_task_summary(&tasks);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.todo, 1);
        assert_eq!(summary.done, 1);
        assert_eq!(summary.doing, 1);
        assert!((summary.completion_rate - 33.333333).abs() < 0.01);
    }
    
    #[test]
    fn test_validate_task_format() {
        let valid_task = "TODO Write documentation";
        let validation = validate_task_format(valid_task);
        assert!(validation.is_valid);
        assert!(validation.errors.is_empty());
        
        let invalid_task = "TODO";
        let validation = validate_task_format(invalid_task);
        assert!(!validation.warnings.is_empty());
    }
}
