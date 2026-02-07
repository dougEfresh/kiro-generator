use {
    super::TreeArgs,
    crate::{
        Manifest,
        Result,
        generator::{Generator, discover::ResolvedAgents},
        source::KdlSources,
    },
    ptree::TreeBuilder,
};

pub(super) async fn execute_tree(generator: &Generator, args: &TreeArgs) -> Result<()> {
    let resolved = &generator.resolved;

    let agents: Vec<(&String, &Manifest)> = resolved
        .agents
        .iter()
        .filter(|(name, _)| {
            if let Some(ref target) = args.agent {
                name == &target
            } else {
                true
            }
        })
        .collect();

    if agents.is_empty() {
        if let Some(ref name) = args.agent {
            eprintln!("Agent '{}' not found", name);
            return Ok(());
        }
        eprintln!("No agents found");
        return Ok(());
    }

    if args.json {
        return print_json(&agents, &resolved.sources);
    }

    for (name, manifest) in agents {
        print_agent_tree(name, manifest, resolved)?;
        println!();
    }

    Ok(())
}

fn print_json(agents: &[(&String, &Manifest)], sources: &KdlSources) -> Result<()> {
    let mut obj = facet_value::VObject::new();
    for (name, manifest) in agents {
        let mut agent = facet_value::VObject::new();
        agent.insert("template", facet_value::Value::from(manifest.template));
        if let Some(ref desc) = manifest.description {
            agent.insert("description", facet_value::Value::from(desc.as_str()));
        }
        let src_arr: facet_value::VArray = match sources.get(name.as_str()) {
            Some(s) => s
                .iter()
                .map(|s| {
                    let mut o = facet_value::VObject::new();
                    o.insert("type", facet_value::Value::from(s.source_type()));
                    o.insert(
                        "path",
                        facet_value::Value::from(s.path().to_string_lossy().as_ref()),
                    );
                    facet_value::Value::from(o)
                })
                .collect(),
            None => facet_value::VArray::new(),
        };
        agent.insert("sources", facet_value::Value::from(src_arr));
        let inherits: facet_value::VArray = manifest
            .inherits
            .iter()
            .map(|s| facet_value::Value::from(s.as_str()))
            .collect();
        agent.insert("inherits", facet_value::Value::from(inherits));
        obj.insert(name.as_str(), facet_value::Value::from(agent));
    }
    let root = facet_value::Value::from(obj);
    println!("{}", facet_json::to_string_pretty(&root)?);
    Ok(())
}

fn print_agent_tree(name: &str, manifest: &Manifest, resolved: &ResolvedAgents) -> Result<()> {
    let mut tree = TreeBuilder::new(format_agent_node(name, manifest, &resolved.sources));

    for parent_name in &manifest.inherits {
        if let Some(parent) = resolved.agents.get(parent_name) {
            add_parent_node(&mut tree, parent_name, parent, resolved);
        }
    }

    let tree = tree.build();
    ptree::print_tree(&tree)?;

    Ok(())
}

fn format_agent_node(name: &str, manifest: &Manifest, sources: &KdlSources) -> String {
    let template_marker = if manifest.template { " (template)" } else { "" };
    let sources_str = match sources.get(name) {
        Some(s) if !s.is_empty() => s
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        _ => "unknown".to_string(),
    };
    let inherits = if manifest.inherits.is_empty() {
        "(none)".to_string()
    } else {
        manifest
            .inherits
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!(
        "{}{}\n  sources: {}\n  inherits: {}",
        name, template_marker, sources_str, inherits
    )
}

fn add_parent_node(
    tree: &mut TreeBuilder,
    name: &str,
    manifest: &Manifest,
    resolved: &ResolvedAgents,
) {
    tree.begin_child(format_agent_node(name, manifest, &resolved.sources));

    for parent_name in &manifest.inherits {
        if let Some(parent) = resolved.agents.get(parent_name) {
            add_parent_node(tree, parent_name, parent, resolved);
        }
    }

    tree.end_child();
}
