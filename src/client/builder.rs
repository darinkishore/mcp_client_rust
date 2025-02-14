use crate::client::Client;
use crate::error::Error;
use crate::transport::stdio::StdioTransport;
use crate::types::{ClientCapabilities, Implementation};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;

/// A builder for creating and initializing an MCP `Client` with a subprocess using stdio transport.
/// This can be used to spawn a local MCP-compatible process and connect automatically.
pub struct ClientBuilder {
    /// The command/binary to invoke, e.g. "uvx".
    command: String,
    /// Arguments passed to the command.
    args: Vec<String>,
    /// Optional working directory for the subprocess.
    working_directory: Option<PathBuf>,
    /// Optional client implementation details (name, version).
    implementation: Option<Implementation>,
    /// Optional client capabilities to announce to the server upon initialization.
    capabilities: Option<ClientCapabilities>,
    /// Environment variables for the subprocess.
    env: HashMap<String, String>,
}

impl ClientBuilder {
    pub fn new(command: &str) -> Self {
        tracing::debug!(%command, "Creating new ClientBuilder");
        Self {
            command: command.to_string(),
            args: vec![],
            working_directory: None,
            implementation: None,
            capabilities: None,
            env: HashMap::new(),
        }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        tracing::trace!(%arg, "Adding argument to ClientBuilder");
        self.args.push(arg.to_string());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_string()).collect();
        tracing::trace!(?args, "Adding multiple arguments to ClientBuilder");
        self.args.extend(args);
        self
    }

    /// Sets the working directory for the subprocess.
    pub fn directory<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        let dir = dir.into();
        tracing::trace!(?dir, "Setting working directory for ClientBuilder");
        self.working_directory = Some(dir);
        self
    }

    pub fn implementation(mut self, name: &str, version: &str) -> Self {
        tracing::trace!(%name, %version, "Setting implementation for ClientBuilder");
        self.implementation = Some(Implementation {
            name: name.to_string(),
            version: version.to_string(),
        });
        self
    }

    pub fn capabilities(mut self, caps: ClientCapabilities) -> Self {
        tracing::trace!(?caps, "Setting capabilities for ClientBuilder");
        self.capabilities = Some(caps);
        self
    }

    /// Adds an environment variable to the subprocess's environment.
    pub fn env(mut self, key: &str, value: &str) -> Self {
        tracing::trace!(%key, %value, "Adding environment variable to ClientBuilder");
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// Spawns the subprocess using the stored command, arguments, etc.,
    /// creates a `StdioTransport` from the subprocess's stdin/stdout,
    /// then returns an initialized `Client`.
    ///
    /// # Errors
    ///
    /// Returns an error if the command cannot be spawned, or if initialization fails.
    pub async fn spawn_and_initialize(self) -> Result<Client, Error> {
        tracing::info!(
            command = %self.command,
            args = ?self.args,
            working_dir = ?self.working_directory,
            "Spawning MCP client process"
        );

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(dir) = &self.working_directory {
            tracing::debug!(?dir, "Setting working directory for process");
            cmd.current_dir(dir);
        }

        for (key, value) in &self.env {
            tracing::debug!(%key, %value, "Setting environment variable");
            cmd.env(key, value);
        }

        cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

        tracing::debug!("Spawning process");
        let mut child = cmd.spawn().map_err(|e| {
            tracing::error!(error = %e, "Failed to spawn process");
            Error::Io(e.to_string())
        })?;

        let child_stdout = child.stdout.take().ok_or_else(|| {
            let err = "No stdout available from spawned process";
            tracing::error!(err);
            Error::Io(err.into())
        })?;

        let child_stdin = child.stdin.take().ok_or_else(|| {
            let err = "No stdin available from spawned process";
            tracing::error!(err);
            Error::Io(err.into())
        })?;

        tracing::debug!("Creating StdioTransport");
        let transport = StdioTransport::with_streams(child_stdout, child_stdin)?;
        let client = Client::new(Arc::new(transport), Some(child));

        let implementation = self.implementation.unwrap_or_else(|| {
            let default_impl = Implementation {
                name: "mcp-client".to_string(),
                version: "0.1.2".to_string(),
            };
            tracing::debug!(?default_impl, "Using default implementation");
            default_impl
        });

        let capabilities = self.capabilities.unwrap_or_else(|| {
            tracing::debug!("Using default capabilities");
            ClientCapabilities::default()
        });

        tracing::debug!(?implementation, ?capabilities, "Initializing client");
        client.initialize(implementation, capabilities).await?;

        tracing::info!("MCP client successfully spawned and initialized");
        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_builder_spawn_failure() {
        // Test that spawning a non-existent command returns an error
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let builder = ClientBuilder::new("non_existent_command");
            let result = builder.spawn_and_initialize().await;
            assert!(
                result.is_err(),
                "Expected error when spawning non-existent command"
            );
        });
    }
}
