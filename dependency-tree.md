# Dependency Tree Command - Status

## Completed

- [x] `kg tree` command with ptree visualization
- [x] `kg tree --json` for structured agent output
- [x] `kg tree --global` / `--local` flags (declared, filtering TBD)
- [x] Agent filtering: `kg tree rust`
- [x] Source tracking with actual file paths (not opaque "inline" labels)
- [x] `KdlAgentSource` refactored: `GlobalManifest(PathBuf)`, `LocalManifest(PathBuf)`, `GlobalFile(PathBuf)`, `LocalFile(PathBuf)`
- [x] JSON sources emit `{type, path}` objects
- [x] `load_manifests()` returns agent→manifest path mapping
- [x] SKILL.md updated with discovery as Step 1
- [x] Reverted wrong approach (sources field on Manifest struct)

## Remaining

- [ ] Wire up `--global` / `--local` filtering in tree.rs (flags exist but aren't used yet)
- [ ] Consider adding more metadata to JSON output (MCP servers, tools, resources count)
- [ ] Evaluate ptree vs hand-rolled indentation (get feedback from other models)
- [ ] Add color coding for templates vs real agents in ptree output

## Files Modified

- `Cargo.toml` — Added ptree dependency
- `src/commands/mod.rs` — TreeArgs struct, Tree command variant
- `src/commands/execute.rs` — Tree command handler
- `src/commands/tree.rs` — Tree implementation (ptree + JSON)
- `src/generator/mod.rs` — Made resolved/discover pub(crate)
- `src/generator/discover.rs` — load_manifests returns path mapping, manifest paths threaded through
- `src/source.rs` — KdlAgentSource refactored with PathBuf on all variants
- `resources/SKILL.md` — Discovery section added as Step 1
