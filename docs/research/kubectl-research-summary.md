# kubectl Pattern Research - Executive Summary

## Mission Completed ✓

**Objective:** Research kubectl's verb-noun command pattern and document how it differs from aofctl's current noun-verb pattern.

**Status:** Complete | **Duration:** ~4 minutes | **Files Generated:** 3

---

## Key Findings

### 1. Command Hierarchy Pattern

**kubectl uses:** `<verb> <noun> [name] [flags]`

**Examples:**
- `kubectl get pods` (verb + noun)
- `kubectl describe deployment nginx` (verb + noun + name)
- `kubectl logs pod/nginx` (verb + type/name)

**Current aofctl:** Mixed patterns, needs restructure

---

### 2. Resource Type System

kubectl supports **three forms** for every resource:
1. **Singular:** `pod`, `deployment`, `service`
2. **Plural:** `pods`, `deployments`, `services`
3. **Abbreviated:** `po`, `deploy`, `svc`

**Proposed aofctl resources:**
- agents (agent, ag)
- workflows (wf)
- tools (tool)
- configs (config)
- sessions (sess)
- executions (exec)

---

### 3. Commands Without Resource Types

Some commands work **without explicit resource types**:

**Configuration:**
- `kubectl apply -f manifest.yaml`
- `kubectl version`
- `kubectl config view`

**Discovery:**
- `kubectl api-resources`
- `kubectl api-versions`

**Shorthand notation:**
- `kubectl logs deploy/nginx` (targets first pod in deployment)
- `kubectl exec svc/web -- env` (executes in pod behind service)

---

### 4. api-resources Command

**Purpose:** Discover all available resource types on the server

**Output:**
```
NAME          SHORTNAMES   APIVERSION   NAMESPACED   KIND
pods          po           v1           true         Pod
deployments   deploy       apps/v1      true         Deployment
services      svc          v1           true         Service
```

**Flags:**
- `--namespaced=true/false` - Filter by scope
- `--api-group=<group>` - Filter by API group
- `--verbs=<verb>` - Show resources supporting specific operations
- `-o wide` - Extended information

**aofctl equivalent needed:** `aofctl api-resources`

---

### 5. Naming Conventions

**Rules observed:**
1. Short > Long (`get` not `retrieve`)
2. Verb-first always
3. Case insensitive
4. Singular/plural agnostic
5. Consistent abbreviations

**Common kubectl aliases:**
- `k` = kubectl
- `kg` = kubectl get
- `kgpo` = kubectl get pods
- `kdeploy` = kubectl describe deployment

---

## Command Mapping

### Current → Target Transformation

| Current | Target | Status |
|---------|--------|--------|
| `aofctl run --config x` | `aofctl run agent x` | Needs refactor |
| `aofctl get agent x` | `aofctl get agent x` | ✓ Already correct |
| `aofctl apply -f x` | `aofctl apply -f x` | ✓ Already correct |
| `aofctl delete agent x` | `aofctl delete agent x` | ✓ Already correct |
| N/A | `aofctl describe agent x` | New command |
| N/A | `aofctl logs agent/x` | New command |
| N/A | `aofctl exec agent/x -- cmd` | New command |
| N/A | `aofctl api-resources` | New command |
| `aofctl tools --server x` | `aofctl get tools` | Restructure |

---

## Implementation Phases

### Phase 1: Core Restructure (Priority: HIGH)
1. Restructure `run` command to accept resource type
2. Add `describe` command for detailed information
3. Implement resource type shortcuts (ag, wf, etc.)
4. Add type/name syntax support (agent/name)

### Phase 2: Runtime Operations (Priority: MEDIUM)
1. Add `logs` command for execution logs
2. Add `exec` command for agent commands
3. Add `create` command for imperative creation
4. Add `edit` command for interactive editing

### Phase 3: Discovery (Priority: HIGH)
1. Implement `api-resources` command
2. Add `config` subcommand for configuration
3. Support all common output formats (-o json/yaml/wide)
4. Implement namespace filtering

### Phase 4: Advanced Features (Priority: LOW)
1. Add `scale` for agent replicas
2. Add `rollout` for workflow deployments
3. Add `top` for resource metrics
4. Add `port-forward` for debugging

---

## Files Generated

### 1. kubectl-patterns-analysis.md (10 sections, 350+ lines)
Comprehensive analysis covering:
- Command hierarchy and structure
- Resource types and shortcuts
- api-resources functionality
- Commands without resource types
- Naming conventions
- Current vs target comparison
- Implementation recommendations
- Complete reference sources

**Location:** `/Users/gshah/work/agentic/my-framework/docs/research/kubectl-patterns-analysis.md`

### 2. aofctl-command-mapping.md (10 sections, 300+ lines)
Detailed command mapping with:
- Before/after command examples
- Resource type reference
- Common flags (kubectl-compatible)
- Implementation phases
- Migration guide for users
- Code structure changes
- Testing matrix

**Location:** `/Users/gshah/work/agentic/my-framework/docs/research/aofctl-command-mapping.md`

### 3. kubectl-research-summary.md (this file)
Executive summary for quick reference

**Location:** `/Users/gshah/work/agentic/my-framework/docs/research/kubectl-research-summary.md`

---

## Memory Storage

All findings stored in coordination memory:
- **Key:** `hive/research/kubectl-patterns`
- **Key:** `hive/research/command-mapping`
- **Location:** `.swarm/memory.db`

---

## Sources Referenced

### Official Kubernetes Documentation
- [kubectl Reference](https://kubernetes.io/docs/reference/generated/kubectl/kubectl-commands)
- [kubectl Quick Reference](https://kubernetes.io/docs/reference/kubectl/quick-reference/)
- [kubectl api-resources](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_api-resources/)
- [Managing Objects Imperatively](https://kubernetes.io/docs/tasks/manage-kubernetes-objects/imperative-command/)

### Community Resources
- [kubectl-aliases GitHub](https://github.com/ahmetb/kubectl-aliases)
- [kubectl Alias Patterns](https://ahmet.im/blog/kubectl-aliases/)
- [Kubernetes Command Shortcuts Guide](https://medium.com/@lingeshcbz/kubernetes-command-shortcuts-a-quick-reference-guide-ad23ce89117a)
- [kubectl Cheat Sheet - Spacelift](https://spacelift.io/blog/kubernetes-cheat-sheet)
- [kubectl Cheat Sheet - ContainIQ](https://www.containiq.com/post/kubectl-cheat-sheet)

### Technical Guides
- [The guide to kubectl - Glasskube](https://glasskube.dev/products/package-manager/guides/kubectl/)
- [kubectl exec Best Practices - Last9](https://last9.io/blog/kubectl-exec-commands-examples-and-best-practices/)

---

## Recommendations for Hive

### Immediate Actions
1. **Architect:** Design new CLI structure based on findings
2. **Coder:** Begin Phase 1 implementation (core restructure)
3. **Tester:** Create test suite for command parsing
4. **Documenter:** Update user documentation with new patterns

### Critical Design Decisions
1. **Backwards Compatibility:** Decide on migration strategy
2. **Resource Types:** Finalize resource type names and shortcuts
3. **Shorthand Support:** Determine which abbreviations to support
4. **Type/Name Syntax:** Implement parser for `type/name` notation

### Success Metrics
- [ ] 100% kubectl command pattern compliance
- [ ] All existing functionality preserved
- [ ] Zero breaking changes (with deprecation warnings)
- [ ] Complete test coverage for new commands
- [ ] Documentation updated with examples

---

## Next Steps

**FOR ARCHITECT AGENT:**
- Review research findings
- Design new CLI structure (cli.rs)
- Define resource type enums
- Plan migration strategy

**FOR CODER AGENT:**
- Implement new command structure
- Add resource type parsing
- Implement shorthand support
- Add type/name syntax parser

**FOR TESTER AGENT:**
- Create command parsing tests
- Test all resource type variations
- Test backwards compatibility
- Test error handling

**FOR REVIEWER AGENT:**
- Review design decisions
- Validate kubectl compliance
- Check for edge cases
- Ensure consistency

---

**Research Completed:** 2025-12-11
**Research Duration:** ~4 minutes
**Agent:** Hive Mind Research Specialist
**Status:** ✓ Complete and stored in coordination memory
**Coordination Key:** `hive/research/kubectl-patterns`

🐝 **Ready for hive collaboration!**
