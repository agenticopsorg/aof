# aofctl Command Mapping: Current → Target (kubectl-style)

## Quick Reference

### Command Structure Transformation

```
BEFORE: aofctl <noun> <verb> [options]
AFTER:  aofctl <verb> <noun> [name] [flags]
```

---

## 1. Core Commands

### Get/List Resources

**Before:**
```bash
# Not clearly defined in current implementation
aofctl get agent my-agent
aofctl get workflow
```

**After (kubectl-style):**
```bash
aofctl get agents                    # list all agents
aofctl get agent my-agent            # get specific agent
aofctl get agent my-agent -o yaml    # with output format
aofctl get workflows -n my-namespace # with namespace
aofctl get wf                        # using shorthand
```

### Run/Execute

**Before:**
```bash
aofctl run --config agent.yaml --input "query"
```

**After:**
```bash
aofctl run agent my-agent --input "query"
aofctl run workflow data-pipeline --params params.yaml
aofctl run wf data-pipeline          # using shorthand
```

### Describe (New)

**After:**
```bash
aofctl describe agent my-agent
aofctl describe workflow data-pipeline
aofctl describe tool google-search
aofctl desc wf data-pipeline          # using alias
```

### Apply Configuration

**Before:**
```bash
aofctl apply --file agent.yaml
```

**After (already correct!):**
```bash
aofctl apply -f agent.yaml
aofctl apply -f config-directory/
aofctl apply -f https://example.com/agent.yaml
```

### Delete Resources

**Before:**
```bash
aofctl delete agent my-agent
```

**After (already correct!):**
```bash
aofctl delete agent my-agent
aofctl delete workflow data-pipeline
aofctl delete agents --all
aofctl del agent my-agent             # using alias
```

### Create Resources

**After (new):**
```bash
aofctl create agent my-agent --config agent.yaml
aofctl create workflow --from-file wf.yaml
```

---

## 2. Runtime Operations

### Logs

**After (new):**
```bash
aofctl logs agent/my-agent
aofctl logs workflow/data-pipeline
aofctl logs agent/my-agent --follow
aofctl logs agent/my-agent --tail 100
aofctl logs agent/my-agent --since 1h
```

### Exec

**After (new):**
```bash
aofctl exec agent/my-agent -- status
aofctl exec agent/my-agent -- list-tools
aofctl exec sess/session-123 -- inspect
```

---

## 3. Discovery & Introspection

### API Resources

**After (new):**
```bash
aofctl api-resources                 # list all resource types
aofctl api-resources -o wide         # with more details
aofctl api-resources --sort-by=name
```

### Version

**Before/After (same):**
```bash
aofctl version
aofctl version --client
aofctl version -o json
```

---

## 4. Configuration Management

### Tools (MCP)

**Before:**
```bash
aofctl tools --server "npx server" --args arg1 arg2
```

**After:**
```bash
aofctl get tools                     # list available tools
aofctl get tool google-search        # get specific tool
aofctl describe tool google-search   # detailed info
```

### Config

**After (new):**
```bash
aofctl config view                   # view current config
aofctl config get-contexts           # list contexts
aofctl config use-context prod       # switch context
aofctl config set runtime.timeout 30s
```

### Validate

**Before:**
```bash
aofctl validate --file agent.yaml
```

**After:**
```bash
aofctl apply -f agent.yaml --dry-run  # kubectl-style validation
aofctl validate agent.yaml            # or keep as standalone
```

---

## 5. Resource Type Reference

### Supported Resource Types

| Type | Shorthand | Plural | Singular |
|------|-----------|--------|----------|
| agents | agent | agents | agent |
| workflows | wf | workflows | workflow |
| tools | tool | tools | tool |
| configs | config | configs | config |
| sessions | sess | sessions | session |
| executions | exec | executions | execution |

### Type/Name Syntax

All commands support `<type>/<name>` notation:

```bash
aofctl logs agent/researcher
aofctl describe workflow/data-pipeline
aofctl exec tool/google-search -- test
```

---

## 6. Common Flags (kubectl-compatible)

| Flag | Description | Example |
|------|-------------|---------|
| `-o, --output` | Output format (json, yaml, wide) | `aofctl get agents -o yaml` |
| `-f, --filename` | Filename or URL | `aofctl apply -f agent.yaml` |
| `-n, --namespace` | Namespace scope | `aofctl get agents -n prod` |
| `-l, --selector` | Label selector | `aofctl get agents -l env=prod` |
| `--all` | Select all resources | `aofctl delete agents --all` |
| `--dry-run` | Preview changes | `aofctl apply -f x --dry-run` |
| `--follow` | Stream logs | `aofctl logs agent/x --follow` |
| `--tail` | Lines to show | `aofctl logs agent/x --tail 50` |
| `--since` | Time range | `aofctl logs agent/x --since 1h` |

---

## 7. Implementation Phases

### Phase 1: Core Restructure
- [x] `get` - Already implemented, verify verb-first
- [x] `apply` - Already implemented correctly
- [x] `delete` - Already implemented correctly
- [ ] `run` - Restructure to accept resource type
- [ ] `describe` - New command for detailed info

### Phase 2: Runtime Operations
- [ ] `logs` - View agent execution logs
- [ ] `exec` - Execute commands in agents
- [ ] `create` - Imperative resource creation
- [ ] `edit` - Interactive editing

### Phase 3: Discovery & Config
- [ ] `api-resources` - Resource type discovery
- [ ] `config` - Configuration management
- [ ] Shorthand support (agent, wf, etc.)
- [ ] Type/name syntax (agent/name)

### Phase 4: Advanced Features
- [ ] `scale` - Scale agent replicas
- [ ] `rollout` - Workflow deployments
- [ ] `port-forward` - Agent port forwarding
- [ ] `top` - Resource usage metrics

---

## 8. Migration Guide for Users

### Breaking Changes

| Old Command | New Command | Migration Path |
|-------------|-------------|----------------|
| `aofctl run --config x` | `aofctl run agent x` | Update scripts to specify resource type |
| `aofctl tools --server x` | `aofctl get tools` | Use get/describe for tool discovery |
| `aofctl validate -f x` | `aofctl apply -f x --dry-run` | Use kubectl-style validation |

### Backwards Compatibility

**Option 1: Deprecation Warnings**
```bash
$ aofctl run --config agent.yaml
⚠️  Warning: This syntax is deprecated. Use: aofctl run agent <name>
```

**Option 2: Auto-detect & Translate**
```bash
# Internally translate old syntax to new
aofctl run --config agent.yaml → aofctl run agent $(parse_name_from_config)
```

**Option 3: Dual Support (v1/v2)**
```bash
aofctl v1 run --config agent.yaml  # old syntax
aofctl run agent my-agent          # new syntax (default)
```

---

## 9. Testing Matrix

### Command Parsing Tests

```rust
#[test]
fn test_get_command_parsing() {
    assert_eq!(parse("get agents"), GetCommand { resource: "agents", name: None });
    assert_eq!(parse("get agent my-agent"), GetCommand { resource: "agent", name: Some("my-agent") });
    assert_eq!(parse("get ag"), GetCommand { resource: "agent", name: None }); // shorthand
}

#[test]
fn test_run_command_parsing() {
    assert_eq!(parse("run agent my-agent --input 'query'"),
               RunCommand { resource: "agent", name: "my-agent", input: "query" });
}

#[test]
fn test_type_name_syntax() {
    assert_eq!(parse("logs agent/my-agent"),
               LogsCommand { target: Target::Agent("my-agent") });
}
```

---

## 10. Code Structure Changes

### Before (current cli.rs)

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    Run { config: String, input: String, output: String },
    Get { resource: String, name: Option<String> },
    Apply { file: String },
    Delete { resource: String, name: String },
    Tools { server: String, args: Vec<String> },
    Validate { file: String },
    Version,
}
```

### After (proposed cli.rs)

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // Resource operations
    Get(GetCommand),
    Describe(DescribeCommand),
    Create(CreateCommand),
    Delete(DeleteCommand),

    // Runtime operations
    Run(RunCommand),
    Apply(ApplyCommand),
    Exec(ExecCommand),
    Logs(LogsCommand),

    // Discovery
    ApiResources(ApiResourcesCommand),

    // Configuration
    Config(ConfigCommand),
    Version(VersionCommand),
}

#[derive(Args, Debug)]
pub struct GetCommand {
    /// Resource type (agent, workflow, tool, etc.)
    pub resource: ResourceType,

    /// Resource name (optional for list)
    pub name: Option<String>,

    #[arg(short = 'o', long)]
    pub output: Option<OutputFormat>,

    #[arg(short = 'n', long)]
    pub namespace: Option<String>,

    #[arg(short = 'l', long)]
    pub selector: Option<String>,
}

#[derive(Args, Debug)]
pub struct RunCommand {
    /// Resource type (agent, workflow)
    pub resource: ResourceType,

    /// Resource name
    pub name: String,

    #[arg(short, long)]
    pub input: String,

    #[arg(short = 'o', long)]
    pub output: Option<OutputFormat>,
}
```

---

**Document Version:** 1.0
**Date:** 2025-12-11
**Companion to:** kubectl-patterns-analysis.md
