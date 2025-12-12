# Kubernetes CLI Pattern Research & Analysis

## Executive Summary

This document analyzes the kubectl command hierarchy and patterns to inform the restructuring of aofctl from a noun-verb pattern (`aofctl agent run`) to a verb-noun pattern (`aofctl run agent`).

**Current Pattern:** `aofctl agent run xxx` (noun-verb)
**Target Pattern:** `aofctl run agent xxx` (verb-noun)
**Rationale:** Align with industry-standard kubectl conventions for better UX

---

## 1. kubectl Command Structure & Hierarchy

### 1.1 Core Pattern: Verb-Noun-[Name]-[Flags]

kubectl follows a natural language structure:

```
kubectl <VERB> <RESOURCE_TYPE> [RESOURCE_NAME] [FLAGS]
```

**Example:**
```bash
kubectl get pods nginx-pod -n production -o yaml
  │      │    │      │          │           └─ output format flag
  │      │    │      │          └─ namespace flag
  │      │    │      └─ resource name (optional)
  │      │    └─ resource type (noun)
  │      └─ operation (verb)
  └─ command
```

### 1.2 Command Categories

Based on [Kubernetes official documentation](https://kubernetes.io/docs/reference/generated/kubectl/kubectl-commands), kubectl commands are organized into:

#### **Getting Started** (Basic CRUD)
- `create` - Create resources from files or stdin
- `get` - Display one or many resources
- `run` - Run a particular image on the cluster
- `expose` - Expose a resource as a service
- `delete` - Delete resources

#### **App Management** (Configuration & Updates)
- `apply` - Apply configuration to resources
- `edit` - Edit resources on the server
- `patch` - Update field(s) of a resource
- `replace` - Replace a resource
- `scale` - Set new size for deployments
- `rollout` - Manage rollout of resources
- `set` - Set specific features on objects
- `label` - Update labels on resources
- `annotate` - Update annotations on resources

#### **Working with Apps** (Runtime Operations)
- `attach` - Attach to a running container
- `exec` - Execute a command in a container
- `logs` - Print logs from containers
- `cp` - Copy files/directories to/from containers
- `port-forward` - Forward ports to pods
- `proxy` - Run a proxy to the API server
- `describe` - Show detailed information
- `top` - Display resource usage

#### **Cluster Management** (Infrastructure)
- `api-versions` - Print supported API versions
- `api-resources` - Print supported resources
- `cluster-info` - Display cluster information
- `cordon` - Mark node as unschedulable
- `drain` - Drain node for maintenance
- `taint` - Update taints on nodes
- `certificate` - Modify certificate resources

#### **Settings & Utils**
- `config` - Modify kubeconfig files
- `version` - Print client/server version
- `explain` - Documentation of resources
- `completion` - Output shell completion code

---

## 2. Resource Types & Shortcuts

### 2.1 Common Resource Types

From [kubectl quick reference](https://kubernetes.io/docs/reference/kubectl/quick-reference/) and [community shortcuts](https://medium.com/@lingeshcbz/kubernetes-command-shortcuts-a-quick-reference-guide-ad23ce89117a):

| Full Name | Short Name | Category |
|-----------|------------|----------|
| pods | po | Workloads |
| deployments | deploy | Workloads |
| replicasets | rs | Workloads |
| statefulsets | sts | Workloads |
| daemonsets | ds | Workloads |
| jobs | job | Workloads |
| cronjobs | cj | Workloads |
| services | svc | Networking |
| endpoints | ep | Networking |
| ingresses | ing | Networking |
| configmaps | cm | Config |
| secrets | sec | Config |
| persistentvolumes | pv | Storage |
| persistentvolumeclaims | pvc | Storage |
| namespaces | ns | Cluster |
| nodes | no | Cluster |
| serviceaccounts | sa | RBAC |

**Key Insight:** Resource types support three forms:
- Singular: `pod`, `deployment`
- Plural: `pods`, `deployments`
- Abbreviated: `po`, `deploy`

All three forms are accepted interchangeably.

### 2.2 aofctl Resource Types (Proposed)

Based on current aofctl structure, we need to support:

| Full Name | Short Name | Category | Description |
|-----------|------------|----------|-------------|
| agents | agent | Workloads | AI agent instances |
| workflows | wf | Workloads | Multi-agent workflows |
| tools | tool | Config | MCP tool definitions |
| configs | config | Config | Agent configurations |
| sessions | sess | Runtime | Active sessions |
| executions | exec | Runtime | Execution history |

---

## 3. kubectl api-resources Command

### 3.1 Purpose & Functionality

From [kubectl api-resources docs](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_api-resources/):

`kubectl api-resources` displays all available resource types on the server, helping users discover what can be created and managed.

### 3.2 Output Format

```bash
$ kubectl api-resources
NAME                    SHORTNAMES   APIVERSION          NAMESPACED   KIND
bindings                             v1                  true         Binding
componentstatuses       cs           v1                  false        ComponentStatus
configmaps              cm           v1                  true         ConfigMap
endpoints               ep           v1                  true         Endpoints
events                  ev           v1                  true         Event
namespaces              ns           v1                  false        Namespace
nodes                   no           v1                  false        Node
persistentvolumeclaims  pvc          v1                  true         PersistentVolumeClaim
persistentvolumes       pv           v1                  false        PersistentVolume
pods                    po           v1                  true         Pod
services                svc          v1                  true         Service
```

### 3.3 Useful Flags

- `--namespaced=true/false` - Filter by namespace scope
- `--api-group=<group>` - Filter by API group
- `--verbs=<verb>` - Show resources supporting specific operations
- `-o wide` - Extended information
- `-o name` - Names only
- `--sort-by=name` - Sort output

### 3.4 aofctl Equivalent: `aofctl api-resources`

Proposed implementation for aofctl:

```bash
$ aofctl api-resources
NAME         SHORTNAMES   KIND        SCOPE       DESCRIPTION
agents       agent        Agent       cluster     AI agent instances
workflows    wf           Workflow    cluster     Multi-agent workflows
tools        tool         Tool        cluster     MCP tool definitions
configs      config       Config      cluster     Agent configurations
sessions     sess         Session     cluster     Active agent sessions
executions   exec         Execution   cluster     Execution history
```

---

## 4. Commands Without Resource Types

### 4.1 Patterns Observed

From [kubectl reference](https://kubernetes.io/docs/reference/generated/kubectl/kubectl-commands):

Some commands operate without explicit resource types:

#### **Apply/Delete with Files**
```bash
kubectl apply -f manifest.yaml
kubectl delete -f manifest.yaml
```

#### **Logs with Shortcuts**
```bash
kubectl logs pod/nginx
kubectl logs deploy/nginx    # targets first pod in deployment
kubectl logs svc/nginx       # targets pods behind service
```

#### **Exec with Shortcuts**
```bash
kubectl exec pod/nginx -- ls
kubectl exec deploy/nginx -- date
```

#### **Configuration Commands**
```bash
kubectl config view
kubectl config get-contexts
kubectl version
```

### 4.2 Shorthand Notation

kubectl supports `TYPE/NAME` syntax:
```bash
kubectl logs deploy/my-app
kubectl exec svc/my-service -- env
kubectl describe node/worker-1
```

### 4.3 aofctl Implementation

Commands that should work without explicit resource types:

```bash
# Apply configurations
aofctl apply -f agent-config.yaml

# View logs
aofctl logs agent/researcher
aofctl logs workflow/data-pipeline

# Execute commands
aofctl exec agent/coder -- status
aofctl exec sess/session-123 -- list-tools

# Configuration
aofctl config view
aofctl version
```

---

## 5. Naming Conventions & Shortcuts

### 5.1 kubectl Alias Patterns

From [kubectl-aliases GitHub](https://github.com/ahmetb/kubectl-aliases) and [community guides](https://ahmet.im/blog/kubectl-aliases/):

#### **Basic Pattern**
- `k` = `kubectl`
- `kg` = `kubectl get`
- `kgpo` = `kubectl get pods`
- `kdeploy` = `kubectl describe deployment`

#### **Complex Patterns**
```bash
# Namespace + resource + flags
ksysgpo = kubectl --namespace=kube-system get pods

# With output format
kgpoyaml = kubectl get pods -o yaml

# With labels
kgpol = kubectl get pods -l
```

### 5.2 Naming Rules

1. **Short > Long**: Prefer `get` over `retrieve`, `logs` over `show-logs`
2. **Consistent Abbreviations**: Always use same shortcuts (`po`, `deploy`, `svc`)
3. **Verb First**: Operation always comes first
4. **Case Insensitive**: `Pod`, `pod`, `PODS` all work
5. **Singular/Plural Agnostic**: `pod` and `pods` both accepted

### 5.3 aofctl Naming Standards

Proposed conventions:

| Full Command | Shorthand | Alias |
|--------------|-----------|-------|
| `aofctl get agents` | `aofctl get agent` | `aofctl get ag` |
| `aofctl run agent` | - | - |
| `aofctl logs agent/name` | - | - |
| `aofctl describe workflow` | `aofctl desc wf` | - |
| `aofctl delete agent` | `aofctl del agent` | - |

---

## 6. Current vs Target Pattern Comparison

### 6.1 Current aofctl Pattern (Noun-Verb)

```rust
// From cli.rs
pub enum Commands {
    Run { config: String, input: String },
    Get { resource: String, name: Option<String> },
    Apply { file: String },
    Delete { resource: String, name: String },
    Tools { server: String, args: Vec<String> },
    Validate { file: String },
    Version,
}
```

**Usage:**
```bash
aofctl run --config agent.yaml --input "query"
aofctl get agent my-agent
```

### 6.2 Target kubectl-Style Pattern (Verb-Noun)

**Proposed Structure:**
```rust
pub enum Commands {
    // Resource management
    Get { resource: String, name: Option<String>, flags: GetFlags },
    Describe { resource: String, name: String },
    Create { resource: String, file: Option<String> },
    Delete { resource: String, name: String },

    // Operations
    Run { resource: String, name: String, input: String },
    Apply { file: String },
    Exec { target: String, command: Vec<String> },
    Logs { target: String, flags: LogFlags },

    // Discovery
    ApiResources { flags: ApiResourceFlags },

    // Config
    Config { subcommand: ConfigCommands },
    Version,
}
```

**Target Usage:**
```bash
# Resource operations
aofctl get agents
aofctl get agent my-agent
aofctl describe agent my-agent
aofctl delete agent my-agent

# Runtime operations
aofctl run agent my-agent --input "query"
aofctl logs agent/my-agent
aofctl exec agent/my-agent -- status

# Configuration
aofctl apply -f agent.yaml
aofctl config view

# Discovery
aofctl api-resources
aofctl version
```

---

## 7. Implementation Recommendations

### 7.1 Command Priority (Phase 1)

Core commands to implement first:

1. **get** - Most frequently used, list/show resources
2. **describe** - Detailed resource information
3. **run** - Execute agents (already exists, needs restructure)
4. **apply** - Create/update from configs
5. **delete** - Remove resources
6. **logs** - View execution logs
7. **version** - Show version info

### 7.2 Command Priority (Phase 2)

Additional commands for full kubectl parity:

1. **api-resources** - Discover available resource types
2. **exec** - Execute commands in running agents
3. **config** - Manage aofctl configuration
4. **create** - Imperative resource creation
5. **edit** - Interactive resource editing
6. **scale** - Scale agent replicas
7. **rollout** - Manage workflow deployments

### 7.3 Resource Type Priority

Implementation order:

1. **agents** - Core primitive
2. **workflows** - Multi-agent coordination
3. **tools** - MCP tool management
4. **configs** - Configuration management
5. **sessions** - Runtime sessions
6. **executions** - Historical records

### 7.4 Key Design Principles

1. **Verb-First**: Always `verb noun` not `noun verb`
2. **Resource Shortcuts**: Support full, plural, and abbreviated forms
3. **Type/Name Syntax**: Support `agent/name` notation
4. **Consistent Flags**: Use standard flags (-n, -o, -f, etc.)
5. **Backwards Compatibility**: Maintain deprecation path for old syntax

---

## 8. Mapping: Current → Target Commands

| Current | Target | Notes |
|---------|--------|-------|
| `aofctl run --config x` | `aofctl run agent x` | Add resource type |
| `aofctl get agent x` | `aofctl get agent x` | Already correct! |
| `aofctl apply -f x` | `aofctl apply -f x` | Already correct! |
| `aofctl delete agent x` | `aofctl delete agent x` | Already correct! |
| N/A | `aofctl describe agent x` | New command |
| N/A | `aofctl logs agent/x` | New command |
| N/A | `aofctl exec agent/x -- cmd` | New command |
| N/A | `aofctl api-resources` | New command |
| `aofctl tools` | `aofctl get tools` | Restructure |
| `aofctl validate` | `aofctl apply --dry-run` | Align with kubectl |
| `aofctl version` | `aofctl version` | Already correct! |

---

## 9. References & Sources

### Official Kubernetes Documentation
- [kubectl Reference](https://kubernetes.io/docs/reference/generated/kubectl/kubectl-commands)
- [kubectl Quick Reference](https://kubernetes.io/docs/reference/kubectl/quick-reference/)
- [kubectl api-resources](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_api-resources/)
- [Managing Objects Imperatively](https://kubernetes.io/docs/tasks/manage-kubernetes-objects/imperative-command/)

### Community Resources
- [kubectl Aliases Repository](https://github.com/ahmetb/kubectl-aliases)
- [kubectl Alias Patterns](https://ahmet.im/blog/kubectl-aliases/)
- [Kubernetes Command Shortcuts Guide](https://medium.com/@lingeshcbz/kubernetes-command-shortcuts-a-quick-reference-guide-ad23ce89117a)
- [kubectl Cheat Sheet](https://spacelift.io/blog/kubernetes-cheat-sheet)

### Technical Guides
- [The guide to kubectl](https://glasskube.dev/products/package-manager/guides/kubectl/)
- [kubectl exec Best Practices](https://last9.io/blog/kubectl-exec-commands-examples-and-best-practices/)

---

## 10. Next Steps

1. **Create detailed design doc** for verb-noun restructure
2. **Define complete resource type hierarchy** for aofctl
3. **Implement api-resources command** first for discovery
4. **Refactor CLI structure** to support new patterns
5. **Create migration guide** for existing users
6. **Write comprehensive tests** for command parsing
7. **Update documentation** with new patterns

---

**Document Version:** 1.0
**Date:** 2025-12-11
**Researcher:** Hive Mind Research Agent
**Status:** Complete
