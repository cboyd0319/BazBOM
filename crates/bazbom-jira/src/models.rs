use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Jira issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    /// Issue key (e.g., "SEC-123")
    pub key: String,

    /// Issue ID (internal Jira ID)
    pub id: String,

    /// Fields
    pub fields: JiraIssueFields,
}

/// Jira issue fields (full response from Jira API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueFields {
    /// Project
    pub project: JiraProject,

    /// Issue type
    #[serde(rename = "issuetype")]
    pub issue_type: JiraIssueType,

    /// Summary (title)
    pub summary: String,

    /// Description (body)
    pub description: JiraDescription,

    /// Priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<JiraPriority>,

    /// Assignee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<JiraUser>,

    /// Status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<JiraStatus>,

    /// Labels
    #[serde(default)]
    pub labels: Vec<String>,

    /// Components
    #[serde(default)]
    pub components: Vec<JiraComponent>,

    /// Custom fields
    #[serde(flatten)]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Jira project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraProject {
    /// Project key (e.g., "SEC")
    pub key: String,

    /// Project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Jira issue type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueType {
    /// Type name (e.g., "Bug", "Task")
    pub name: String,

    /// Type ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Jira description (Atlassian Document Format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraDescription {
    #[serde(rename = "type")]
    pub doc_type: String, // "doc"

    pub version: i32, // 1

    pub content: Vec<JiraContent>,
}

/// Jira content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JiraContent {
    #[serde(rename = "paragraph")]
    Paragraph { content: Vec<JiraTextNode> },

    #[serde(rename = "heading")]
    Heading {
        attrs: HeadingAttrs,
        content: Vec<JiraTextNode>,
    },

    #[serde(rename = "codeBlock")]
    CodeBlock {
        attrs: CodeBlockAttrs,
        content: Vec<JiraTextNode>,
    },

    #[serde(rename = "bulletList")]
    BulletList { content: Vec<ListItem> },
}

/// Jira text node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTextNode {
    #[serde(rename = "type")]
    pub node_type: String, // "text"

    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub marks: Option<Vec<TextMark>>,
}

/// Text mark (bold, italic, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMark {
    #[serde(rename = "type")]
    pub mark_type: String, // "strong", "em", "code", "link"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<HashMap<String, serde_json::Value>>,
}

/// Heading attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingAttrs {
    pub level: i32, // 1-6
}

/// Code block attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlockAttrs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>, // "bash", "rust", etc.
}

/// List item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    #[serde(rename = "type")]
    pub item_type: String, // "listItem"

    pub content: Vec<JiraContent>,
}

/// Jira priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraPriority {
    pub name: String, // "Highest", "High", "Medium", "Low", "Lowest"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Jira user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraUser {
    #[serde(rename = "accountId")]
    pub account_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Jira status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatus {
    pub name: String, // "To Do", "In Progress", "Done"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Jira component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraComponent {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// BazBOM-specific Jira metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BazBomJiraMetadata {
    /// CVE ID
    pub cve_id: String,

    /// CVSS score
    pub cvss_score: f32,

    /// EPSS score
    pub epss_score: Option<f32>,

    /// CISA KEV status
    pub kev_status: bool,

    /// Reachability
    pub reachability: Reachability,

    /// Package PURL
    pub package_purl: String,

    /// Current version
    pub current_version: String,

    /// Fix version
    pub fix_version: Option<String>,

    /// Remediation effort estimate
    pub remediation_effort: RemediationEffort,

    /// BazBOM scan URL
    pub bazbom_link: Option<String>,
}

/// Reachability status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Reachability {
    Reachable,
    Unreachable,
    Unknown,
}

/// Remediation effort estimate
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RemediationEffort {
    #[serde(rename = "<1h")]
    LessThanOneHour,

    #[serde(rename = "1-4h")]
    OneToFourHours,

    #[serde(rename = "1d")]
    OneDay,

    #[serde(rename = "1w")]
    OneWeek,

    #[serde(rename = ">1w")]
    MoreThanOneWeek,
}

/// Simple project reference for creating issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRef {
    /// Project key (e.g., "SEC")
    pub key: String,
}

/// Simple issue type reference for creating issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTypeRef {
    /// Type name (e.g., "Bug", "Task")
    pub name: String,
}

/// Issue fields for creating issues (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueFields {
    /// Project reference
    pub project: ProjectRef,

    /// Summary (title)
    pub summary: String,

    /// Description (optional for creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<JiraDescription>,

    /// Issue type
    #[serde(rename = "issuetype")]
    pub issuetype: IssueTypeRef,

    /// Labels (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,

    /// Priority (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<JiraPriority>,

    /// Assignee (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<JiraUser>,

    /// Custom fields
    #[serde(flatten)]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Issue creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub fields: IssueFields,
}

/// Issue creation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueResponse {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_url: String,
}

/// Issue update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<HashMap<String, Vec<UpdateOperation>>>,
}

/// Update operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOperation {
    pub operation: String, // "add", "set", "remove"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// Transition request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRequest {
    pub transition: Transition,
}

/// Transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
}

/// Comment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCommentRequest {
    pub body: JiraDescription,
}

/// Search request (JQL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub jql: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,

    #[serde(rename = "startAt", skip_serializing_if = "Option::is_none")]
    pub start_at: Option<i32>,

    #[serde(rename = "maxResults", skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total: i32,

    #[serde(rename = "startAt")]
    pub start_at: i32,

    #[serde(rename = "maxResults")]
    pub max_results: i32,

    pub issues: Vec<JiraIssue>,
}
