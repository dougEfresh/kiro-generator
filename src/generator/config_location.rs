use {
    super::*,
    crate::config::ConfigResult,
    std::{fmt::Display, path::Path},
};

/// Recursively search for an agent TOML file in a directory tree
fn find_agent_file(
    fs: &Fs,
    dir: &Path,
    agent_name: &str,
    current_depth: usize,
    max_depth: usize,
) -> Option<PathBuf> {
    if current_depth > max_depth || !fs.exists(dir) {
        return None;
    }

    let entries = fs.read_dir_sync(dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        let metadata = entry.metadata().ok()?;

        if metadata.is_dir() {
            // Recurse into subdirectory
            if let Some(found) =
                find_agent_file(fs, &path, agent_name, current_depth + 1, max_depth)
            {
                return Some(found);
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("toml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && stem == agent_name
        {
            return Some(path);
        }
    }

    None
}

/// Represents where configuration files are located
pub enum ConfigLocation {
    /// Only global ~/.kiro/generators
    Global(PathBuf),
    /// Only local ./.kiro/generators
    Local,
    /// Both global and local configs (local overrides global)
    Both(PathBuf),
}

impl ConfigLocation {
    pub fn global_path(&self) -> PathBuf {
        match self {
            ConfigLocation::Both(p) | Self::Global(p) => p.clone(),
            #[cfg(not(test))]
            ConfigLocation::Local => PathBuf::default(),
            #[cfg(test)]
            ConfigLocation::Local => PathBuf::from("dev").join("null"),
        }
    }

    /// Validate that there are no duplicate agent names in the agent
    /// directories
    pub fn validate(&self, fs: &Fs) -> ConfigResult<()> {
        fn scan_for_duplicates(
            fs: &Fs,
            dir: &Path,
            current_depth: usize,
            max_depth: usize,
            seen: &mut HashMap<String, PathBuf>,
            scope: &str,
        ) -> ConfigResult<()> {
            if current_depth > max_depth || !fs.exists(dir) {
                return Ok(());
            }

            let entries = fs.read_dir_sync(dir).map_err(|e| {
                crate::Error::Report(format!("Failed to read directory {}: {}", dir.display(), e))
            })?;

            for entry in entries.flatten() {
                let path = entry.path();
                let metadata = entry
                    .metadata()
                    .map_err(|e| crate::Error::Report(format!("Failed to read metadata: {}", e)))?;

                if metadata.is_dir() {
                    scan_for_duplicates(fs, &path, current_depth + 1, max_depth, seen, scope)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("toml")
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                {
                    let agent_name = stem.to_string();
                    if let Some(existing_path) = seen.get(&agent_name) {
                        return Err(crate::Error::DuplicateAgent {
                            name: agent_name,
                            scope: scope.to_string(),
                            first: existing_path.display().to_string(),
                            second: path.display().to_string(),
                        });
                    }
                    seen.insert(agent_name, path);
                }
            }

            Ok(())
        }

        // Validate global agents if applicable
        if !matches!(self, ConfigLocation::Local) {
            let global_agents_dir = match self {
                ConfigLocation::Global(path) | ConfigLocation::Both(path) => path.join("agents"),
                ConfigLocation::Local => unreachable!(),
            };
            let mut seen = HashMap::new();
            scan_for_duplicates(
                fs,
                &global_agents_dir,
                0,
                super::MAX_AGENT_DIR_DEPTH,
                &mut seen,
                "global agents",
            )?;
        }

        // Validate local agents if applicable
        if !matches!(self, ConfigLocation::Global(_)) {
            let local_agents_dir = PathBuf::from(".kiro/generators/agents");
            let mut seen = HashMap::new();
            scan_for_duplicates(
                fs,
                &local_agents_dir,
                0,
                super::MAX_AGENT_DIR_DEPTH,
                &mut seen,
                "local agents",
            )?;
        }

        Ok(())
    }

    /// Get path to agent definition file in agents/ directory (searches
    /// recursively)
    pub fn global_agent(&self, fs: &Fs, name: impl AsRef<str>) -> Option<PathBuf> {
        let agents_dir = match self {
            ConfigLocation::Global(path) | ConfigLocation::Both(path) => path.join("agents"),
            ConfigLocation::Local => return None,
        };

        find_agent_file(
            fs,
            &agents_dir,
            name.as_ref(),
            0,
            super::MAX_AGENT_DIR_DEPTH,
        )
    }

    /// Get path to agent definition file in agents/ directory (searches
    /// recursively)
    pub fn local_agent(&self, fs: &Fs, name: impl AsRef<str>) -> Option<PathBuf> {
        let agents_dir = match self {
            Self::Local | Self::Both(_) => PathBuf::from(".kiro/generators/agents"),
            Self::Global(_) => return None,
        };

        find_agent_file(
            fs,
            &agents_dir,
            name.as_ref(),
            0,
            super::MAX_AGENT_DIR_DEPTH,
        )
    }

    /// Get path to global manifests directory
    pub fn global_manifests_dir(&self) -> PathBuf {
        match self {
            ConfigLocation::Global(path) | ConfigLocation::Both(path) => path.join("manifests"),
            #[cfg(not(test))]
            ConfigLocation::Local => PathBuf::default(),
            #[cfg(test)]
            ConfigLocation::Local => PathBuf::from("dev").join("null"),
        }
    }

    /// Get path to local manifests directory
    pub fn local_manifests_dir(&self) -> PathBuf {
        match self {
            Self::Local | Self::Both(_) => {
                PathBuf::from(".kiro").join("generators").join("manifests")
            }
            #[cfg(not(test))]
            Self::Global(_) => PathBuf::default(),
            #[cfg(test)]
            Self::Global(_) => PathBuf::from("dev").join("null"),
        }
    }
}

impl Debug for ConfigLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLocation::Global(_) => write!(f, "[global]"),
            ConfigLocation::Local => write!(f, "[local]"),
            ConfigLocation::Both(_) => {
                write!(f, "[global,local]")
            }
        }
    }
}

impl Display for ConfigLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLocation::Global(p) => write!(f, "global={}", p.display()),
            ConfigLocation::Local => write!(f, "local"),
            ConfigLocation::Both(p) => {
                write!(f, "global={},local", p.display())
            }
        }
    }
}
