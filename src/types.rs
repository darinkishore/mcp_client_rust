use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;

/// A progress token, used to associate progress notifications with the original request
pub type ProgressToken = String;

/// An opaque token used to represent a cursor for pagination
pub type Cursor = String;

/// Base request metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_token: Option<ProgressToken>,
}

/// Base notification metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// The progress thus far
    pub progress: u64,
    /// Optional total progress
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    /// Optional message describing the progress
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Resource contents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResourceContents {
    Text {
        uri: String,
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
        text: String,
    },
    Blob {
        uri: String,
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
        blob: String,
    },
}
/// A resource in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Unique identifier for the resource
    pub uri: String,
    /// Human-readable title
    pub title: String,
    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Resource contents
    pub contents: ResourceContents,
}

/// Model preferences for completion requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPreferences {
    /// Optional model name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional temperature (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Optional maximum number of tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// A completion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    /// The completion text
    pub text: String,
    /// Optional score/confidence (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

/// Client implementation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    /// Name of the implementation
    pub name: String,
    /// Version of the implementation
    pub version: String,
}

/// Client capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Optional custom capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Server capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapability>,
}

/// Logging levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// A prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

/// A prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Content types for prompt messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        uri: String,
        alt_text: Option<String>,
    },
    #[serde(rename = "resource")]
    Resource { resource: Resource },
}

impl MessageContent {
    pub fn contains(&self, substring: &str) -> bool {
        match self {
            MessageContent::Text { text } => text.contains(substring),
            MessageContent::Image { alt_text, .. } => alt_text
                .as_ref()
                .map(|text| text.contains(substring))
                .unwrap_or(false),
            MessageContent::Resource { resource } => {
                resource.title.contains(substring)
                    || resource
                        .description
                        .as_ref()
                        .map(|d| d.contains(substring))
                        .unwrap_or(false)
            }
        }
    }
}

impl PartialEq for MessageContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MessageContent::Text { text: t1 }, MessageContent::Text { text: t2 }) => t1 == t2,
            (
                MessageContent::Image {
                    uri: u1,
                    alt_text: a1,
                },
                MessageContent::Image {
                    uri: u2,
                    alt_text: a2,
                },
            ) => u1 == u2 && a1 == a2,
            (
                MessageContent::Resource { resource: r1 },
                MessageContent::Resource { resource: r2 },
            ) => r1.uri == r2.uri,
            _ => false,
        }
    }
}

/// A prompt message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: Vec<MessageContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
}

/// A tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Root definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Initialize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeRequest {
    pub client_info: Implementation,
    pub capabilities: ClientCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
}

/// Initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    #[serde(rename = "serverInfo")]
    pub server_info: Implementation,
    #[serde(rename = "capabilities")]
    pub capabilities: ServerCapabilities,
}

/// List resources request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

/// List resources result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
    pub resources: Vec<Resource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
}

/// List prompts request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

/// List prompts result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResult {
    pub prompts: Vec<Prompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
}

/// Complete request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteRequest {
    pub prompt_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_preferences: Option<ModelPreferences>,
}

/// Complete result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteResult {
    pub completion: Completion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<MessageContent>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    pub contents: Vec<ResourceContents>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingCapability {
    // If the server supports logging, define fields as needed
    // If not returned by server, you can leave this empty
}
