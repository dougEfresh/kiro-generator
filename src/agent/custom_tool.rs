use {
    facet::Facet,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Facet, Default, Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[facet(default, deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct CustomToolConfig {
    /// The URL for HTTP-based MCP server communication
    #[facet(default, skip_serializing_if = String::is_empty)]
    pub url: String,
    /// HTTP headers to include when communicating with HTTP-based MCP servers
    #[facet(default, skip_serializing_if = HashMap::is_empty)]
    pub headers: HashMap<String, String>,
    /// The command string used to initialize the mcp server
    #[facet(default)]
    pub command: String,
    /// A list of arguments to be used to run the command with
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub args: Vec<String>,
    /// A list of environment variables to run the command with
    #[facet(default, skip_serializing_if = HashMap::is_empty)]
    pub env: HashMap<String, String>,
    /// Timeout for each mcp request in ms
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub timeout: Option<u64>,
    /// A boolean flag to denote whether or not to load this mcp server
    #[facet(default)]
    pub disabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::config::{ConfigResult, toml_parse},
        indoc::indoc,
    };

    #[derive(Facet, Debug)]
    struct McpDoc {
        #[facet(default, rename = "mcpServers")]
        mcp_servers: HashMap<String, CustomToolConfig>,
    }

    #[test]
    fn parse_basic_mcp() -> ConfigResult<()> {
        let raw = indoc! {
            r#"
[mcpServers.rustdocs]
command = "rust-docs-mcp"
timeout  =1000
"#
        };

        let doc: McpDoc = toml_parse(raw)?;
        assert!(!doc.mcp_servers.is_empty());
        assert!(doc.mcp_servers.contains_key("rustdocs"));
        let mcp = doc.mcp_servers.get("rustdocs").unwrap();
        assert_eq!(mcp.command, "rust-docs-mcp");
        assert_eq!(mcp.timeout, Some(1000));
        Ok(())
    }

    #[test]
    fn parse_mcp_with_url() -> ConfigResult<()> {
        let raw = r#"
        [mcpServers.remote]
        url="http://localhost:8080"
        "#;
        let doc: McpDoc = toml_parse(raw)?;
        assert!(!doc.mcp_servers.is_empty());
        assert!(doc.mcp_servers.contains_key("remote"));
        let mcp = doc.mcp_servers.get("remote").unwrap();
        assert_eq!(mcp.url, "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn parse_mcp_with_env_and_headers() -> ConfigResult<()> {
        let raw = r#"
        [mcpServers.api]
        command = "api-server"
        [mcpServers.api.env]
        API_KEY= "secret123"
        DEBUG="true"
        [mcpServers.api.headers]
        Authorization= "Bearer token"
        "Content-Type"= "application/json"
        "#;
        let doc: McpDoc = toml_parse(raw)?;
        assert!(!doc.mcp_servers.is_empty());
        assert!(doc.mcp_servers.contains_key("api"));
        let mcp = doc.mcp_servers.get("api").unwrap();
        assert!(mcp.timeout.is_none());
        assert_eq!(mcp.env.len(), 2);
        assert_eq!(mcp.headers.len(), 2);
        let env = &mcp.env;
        assert_eq!(env.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(env.get("DEBUG"), Some(&"true".to_string()));
        let header = &mcp.headers;
        assert_eq!(
            header.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
        assert_eq!(
            header.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        Ok(())
    }

    #[test]
    fn parse_mcp_with_args() -> ConfigResult<()> {
        let raw = indoc! { r#"
        [mcpServers.tool]
        command = "my-tool"
        args = ["--verbose", "--output=json"]
        disabled = true
        "#
        };

        let doc: McpDoc = toml_parse(raw)?;
        assert!(!doc.mcp_servers.is_empty());
        assert!(doc.mcp_servers.contains_key("tool"));
        let mcp = doc.mcp_servers.get("tool").unwrap();
        assert_eq!(mcp.args, vec!["--verbose", "--output=json"]);
        assert!(mcp.disabled.unwrap_or_default());
        Ok(())
    }
}
