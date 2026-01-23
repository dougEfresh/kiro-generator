use {
    super::{Agent, Knowledge, tools::*},
    facet::Facet,
    std::collections::HashSet,
};

/// Normalized agent for stable diffing with concrete tool types
#[derive(Facet, Debug, Clone, Default)]
pub struct NormalizedAgent {
    pub name: String,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub description: Option<String>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub prompt: Option<String>,
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub tools: HashSet<String>,
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub allowed_tools: HashSet<String>,
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub resources: Vec<String>,
    #[facet(default, skip_serializing_if = Vec::is_empty)]
    pub knowledge: Vec<Knowledge>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub shell: Option<ExecuteShellTool>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub aws: Option<AwsTool>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub read: Option<ReadTool>,
    #[facet(default, skip_serializing_if = Option::is_none)]
    pub write: Option<WriteTool>,
    #[facet(default, skip_serializing_if = HashSet::is_empty)]
    pub other_tools: HashSet<String>,
}

impl Agent {
    pub fn normalize(self) -> NormalizedAgent {
        let mut shell = None;
        let mut aws = None;
        let mut read = None;
        let mut write = None;
        let mut other_tools = HashSet::new();

        for (tool_name, value) in self.tools_settings {
            let json = facet_json::to_string(&value).unwrap_or_default();
            match tool_name.as_str() {
                "shell" => shell = facet_json::from_str(&json).ok(),
                "aws" => aws = facet_json::from_str(&json).ok(),
                "read" => read = facet_json::from_str(&json).ok(),
                "write" => write = facet_json::from_str(&json).ok(),
                _ => {
                    other_tools.insert(tool_name);
                }
            }
        }

        let mut resources = HashSet::new();
        let mut knowledge: Vec<Knowledge> = Vec::new();

        for resource in self.resources {
            if let Some(s) = resource.as_string() {
                resources.insert(s.to_string());
            } else {
                // Try to parse as Knowledge object
                let json = facet_json::to_string(&resource).unwrap_or_default();
                match facet_json::from_str::<Knowledge>(&json) {
                    Ok(k) => {
                        knowledge.push(k);
                    }
                    Err(e) => tracing::warn!("unable to decode knowledge '{json}''\n{e}"),
                };
            }
        }

        let mut resources: Vec<_> = resources.into_iter().collect();
        resources.sort();

        knowledge.sort_by(|a, b| a.name.cmp(&b.name));

        NormalizedAgent {
            name: self.name,
            description: self.description,
            prompt: self.prompt,
            tools: self.tools,
            allowed_tools: self.allowed_tools,
            resources,
            knowledge,
            shell,
            aws,
            read,
            write,
            other_tools,
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::config::Manifest, facet_diff::FacetDiff};

    #[test]
    fn test_default_agent() -> crate::Result<()> {
        let agent = Agent {
            name: "test".to_string(),
            ..Default::default()
        };
        assert_eq!("test", format!("{agent}"));

        let kg_agent = Manifest::default();
        let agent = Agent::try_from(&kg_agent)?;
        assert_eq!(agent.tools, Agent::default().tools);

        Ok(())
    }

    #[test]
    fn test_normalized_agent_diff_identical() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            description: Some("Test agent".to_string()),
            prompt: Some("You are a test agent".to_string()),
            ..Default::default()
        };
        let agent2 = agent1.clone();

        let diff = agent1.diff(&agent2);
        assert!(diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_prompt_changed() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            prompt: Some("Original prompt".to_string()),
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            prompt: Some("Changed prompt".to_string()),
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_stability() {
        // Create agents with resources in different order
        let mut agent1 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://b.md".to_string(), "file://a.md".to_string()],
            ..Default::default()
        };
        let mut agent2 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://a.md".to_string(), "file://b.md".to_string()],
            ..Default::default()
        };

        // Sort both to normalize
        agent1.resources.sort();
        agent2.resources.sort();

        // After sorting, should be equal
        let diff = agent1.diff(&agent2);
        assert!(diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_resources_added() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://a.md".to_string()],
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://a.md".to_string(), "file://b.md".to_string()],
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_knowledge_changed() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            knowledge: vec![Knowledge {
                name: "kb1".to_string(),
                knowledge_type: "best".to_string(),
                source: Some("file://docs".to_string()),
                description: Some("Original".to_string()),
                index_type: None,
                auto_update: None,
            }],
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            knowledge: vec![Knowledge {
                name: "kb1".to_string(),
                knowledge_type: "best".to_string(),
                source: Some("file://docs".to_string()),
                description: Some("Changed".to_string()),
                index_type: None,
                auto_update: None,
            }],
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_shell_tool_changed() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            shell: Some(ExecuteShellTool {
                allowed_commands: HashSet::from(["git status".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            shell: Some(ExecuteShellTool {
                allowed_commands: HashSet::from([
                    "git status".to_string(),
                    "git fetch".to_string(),
                ]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_allowed_tools_changed() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            allowed_tools: HashSet::from(["read".to_string()]),
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            allowed_tools: HashSet::from(["read".to_string(), "write".to_string()]),
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalized_agent_diff_empty_to_populated() {
        let agent1 = NormalizedAgent {
            name: "test".to_string(),
            ..Default::default()
        };
        let agent2 = NormalizedAgent {
            name: "test".to_string(),
            resources: vec!["file://a.md".to_string()],
            shell: Some(ExecuteShellTool::default()),
            ..Default::default()
        };

        let diff = agent1.diff(&agent2);
        assert!(!diff.is_equal());
    }

    #[test]
    fn test_normalize_malformed_knowledge() {
        use facet_value::Value;

        let agent = Agent {
            name: "test".to_string(),
            resources: vec![
                Value::from("file://valid.md"),
                Value::from(42), // Not a string or valid Knowledge
            ],
            ..Default::default()
        };

        let normalized = agent.normalize();
        assert_eq!(normalized.resources.len(), 1);
        assert_eq!(normalized.resources[0], "file://valid.md");
        assert!(normalized.knowledge.is_empty());
    }
}
