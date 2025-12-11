import { useState } from 'react';
import { FileCode, Search, Loader2, ChevronRight } from 'lucide-react';
import { toast } from '../lib/toast';

interface Template {
  id: string;
  name: string;
  description: string;
  category: 'devops' | 'development' | 'support' | 'automation';
  yaml: string;
  tags: string[];
}

const TEMPLATES: Template[] = [
  {
    id: 'k8s-helper',
    name: 'Kubernetes Helper',
    description: 'Expert at kubectl commands, pod troubleshooting, and K8s concepts',
    category: 'devops',
    tags: ['kubernetes', 'kubectl', 'troubleshooting'],
    yaml: `apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: k8s-helper
spec:
  model: gemini-2.0-flash
  instructions: |
    You are a helpful Kubernetes expert. Help users with kubectl commands,
    troubleshoot pod issues, and explain K8s concepts clearly.
  tools:
    - type: Shell
      config:
        allowed_commands: ["kubectl"]
  temperature: 0.7
  max_iterations: 10`,
  },
  {
    id: 'code-reviewer',
    name: 'Code Reviewer',
    description: 'Reviews pull requests and provides constructive feedback',
    category: 'development',
    tags: ['code-review', 'github', 'quality'],
    yaml: `apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: code-reviewer
spec:
  model: gemini-2.0-flash
  instructions: |
    You are an experienced code reviewer. Analyze code for:
    - Code quality and best practices
    - Security vulnerabilities
    - Performance issues
    - Readability and maintainability
    Provide constructive, actionable feedback.
  tools:
    - type: GitHub
      config:
        permissions: ["read:repo", "write:pr"]
  temperature: 0.3
  max_iterations: 5`,
  },
  {
    id: 'slack-bot',
    name: 'Slack Support Bot',
    description: 'Auto-responds to common questions in Slack channels',
    category: 'support',
    tags: ['slack', 'support', 'automation'],
    yaml: `apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: slack-support-bot
spec:
  model: gemini-2.0-flash
  instructions: |
    You are a helpful support bot. Answer common questions and provide
    guidance. If you cannot help, suggest contacting a human.
  tools:
    - type: HTTP
      config:
        allowed_domains: ["api.slack.com"]
  trigger:
    type: Slack
    config:
      channel: "#support"
  temperature: 0.7
  max_iterations: 3`,
  },
  {
    id: 'incident-responder',
    name: 'Incident Responder',
    description: 'Diagnoses issues and suggests remediation steps',
    category: 'devops',
    tags: ['incident', 'sre', 'troubleshooting'],
    yaml: `apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: incident-responder
spec:
  model: gemini-2.0-flash
  instructions: |
    You are an SRE expert. When an incident occurs:
    1. Diagnose the root cause
    2. Suggest remediation steps
    3. Provide commands to execute
    Ask for human approval before executing destructive operations.
  tools:
    - type: Shell
      config:
        allowed_commands: ["kubectl", "docker", "systemctl"]
    - type: HTTP
      config:
        allowed_domains: ["*"]
  memory:
    enabled: true
    ttl: 3600
  temperature: 0.2
  max_iterations: 15`,
  },
  {
    id: 'doc-writer',
    name: 'Documentation Writer',
    description: 'Generates comprehensive documentation from code',
    category: 'development',
    tags: ['documentation', 'markdown', 'api-docs'],
    yaml: `apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: doc-writer
spec:
  model: gemini-2.0-flash
  instructions: |
    You are a technical writer. Generate clear, comprehensive documentation
    including:
    - API reference
    - Usage examples
    - Best practices
    - Troubleshooting guides
  temperature: 0.5
  max_iterations: 8`,
  },
  {
    id: 'log-analyzer',
    name: 'Log Analyzer',
    description: 'Analyzes logs to identify patterns and issues',
    category: 'devops',
    tags: ['logs', 'debugging', 'analysis'],
    yaml: `apiVersion: aof.dev/v1
kind: Agent
metadata:
  name: log-analyzer
spec:
  model: gemini-2.0-flash
  instructions: |
    You are a log analysis expert. Analyze logs to:
    - Identify error patterns
    - Detect anomalies
    - Suggest fixes
    - Provide root cause analysis
  tools:
    - type: Shell
      config:
        allowed_commands: ["grep", "awk", "jq"]
  temperature: 0.3
  max_iterations: 10`,
  },
];

const CATEGORIES = [
  { id: 'all', label: 'All Templates' },
  { id: 'devops', label: 'DevOps & SRE' },
  { id: 'development', label: 'Development' },
  { id: 'support', label: 'Support & Help' },
  { id: 'automation', label: 'Automation' },
];

interface AgentTemplatesProps {
  onLoadTemplate: (yaml: string) => void;
}

export function AgentTemplates({ onLoadTemplate }: AgentTemplatesProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [selectedTemplate, setSelectedTemplate] = useState<Template | null>(null);

  const filteredTemplates = TEMPLATES.filter(template => {
    const matchesSearch = template.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      template.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      template.tags.some(tag => tag.toLowerCase().includes(searchQuery.toLowerCase()));

    const matchesCategory = selectedCategory === 'all' || template.category === selectedCategory;

    return matchesSearch && matchesCategory;
  });

  const handleLoadTemplate = (template: Template) => {
    onLoadTemplate(template.yaml);
    toast.success(`Loaded template: ${template.name}`, 'Template configuration loaded into editor');
    setSelectedTemplate(null);
  };

  return (
    <div className="flex h-full">
      {/* Sidebar - Categories */}
      <div className="w-64 border-r border-zinc-700 bg-zinc-900/50 p-4">
        <h3 className="text-sm font-semibold text-zinc-400 uppercase tracking-wide mb-3">
          Categories
        </h3>
        <div className="space-y-1">
          {CATEGORIES.map(category => (
            <button
              key={category.id}
              onClick={() => setSelectedCategory(category.id)}
              className={`w-full text-left px-3 py-2 rounded-lg transition-colors ${
                selectedCategory === category.id
                  ? 'bg-sky-400/60 text-white'
                  : 'text-zinc-400 hover:bg-zinc-800'
              }`}
            >
              {category.label}
            </button>
          ))}
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col">
        {/* Search Bar */}
        <div className="p-6 border-b border-zinc-700">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-zinc-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search templates..."
              className="w-full pl-10 pr-4 py-3 bg-zinc-800 border border-zinc-700 rounded-lg text-white placeholder-zinc-500 focus:outline-none focus:ring-2 focus:ring-sky-400/60"
            />
          </div>
        </div>

        {/* Templates Grid */}
        <div className="flex-1 overflow-y-auto p-6">
          {filteredTemplates.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-zinc-500">
              <FileCode className="w-16 h-16 mb-4" />
              <p className="text-lg">No templates found</p>
              <p className="text-sm">Try adjusting your search or filters</p>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredTemplates.map(template => (
                <div
                  key={template.id}
                  className="bg-zinc-800/50 border border-zinc-700 rounded-lg p-5 hover:border-sky-400/60 transition-all cursor-pointer group"
                  onClick={() => setSelectedTemplate(template)}
                >
                  <div className="flex items-start justify-between mb-3">
                    <h3 className="text-lg font-semibold text-white group-hover:text-sky-400 transition-colors">
                      {template.name}
                    </h3>
                    <ChevronRight className="w-5 h-5 text-zinc-500 group-hover:text-sky-400 transition-colors" />
                  </div>

                  <p className="text-sm text-zinc-400 mb-4 line-clamp-2">
                    {template.description}
                  </p>

                  <div className="flex flex-wrap gap-2">
                    {template.tags.map(tag => (
                      <span
                        key={tag}
                        className="px-2 py-1 text-xs bg-zinc-700 text-zinc-300 rounded"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Template Preview Modal */}
      {selectedTemplate && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-6">
          <div className="bg-zinc-900 border border-zinc-700 rounded-lg max-w-4xl w-full max-h-[90vh] flex flex-col">
            {/* Modal Header */}
            <div className="p-6 border-b border-zinc-700">
              <h2 className="text-2xl font-bold text-white mb-2">{selectedTemplate.name}</h2>
              <p className="text-zinc-400">{selectedTemplate.description}</p>
              <div className="flex flex-wrap gap-2 mt-3">
                {selectedTemplate.tags.map(tag => (
                  <span
                    key={tag}
                    className="px-2 py-1 text-xs bg-zinc-800 text-zinc-300 rounded"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>

            {/* Modal Body - YAML Preview */}
            <div className="flex-1 overflow-y-auto p-6">
              <h3 className="text-sm font-semibold text-zinc-400 uppercase tracking-wide mb-3">
                Configuration
              </h3>
              <pre className="bg-zinc-950 border border-zinc-800 rounded-lg p-4 text-sm text-zinc-300 overflow-x-auto">
                {selectedTemplate.yaml}
              </pre>
            </div>

            {/* Modal Footer */}
            <div className="p-6 border-t border-zinc-700 flex justify-end space-x-3">
              <button
                onClick={() => setSelectedTemplate(null)}
                className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => handleLoadTemplate(selectedTemplate)}
                className="px-4 py-2 bg-sky-400/60 hover:bg-sky-400/80 text-white rounded-lg transition-colors"
              >
                Load Template
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
