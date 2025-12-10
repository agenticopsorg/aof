import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: ReactNode;
  icon: string;
};

const FeatureList: FeatureItem[] = [
  {
    title: '🚀 Quick to Deploy',
    icon: '🚀',
    description: (
      <>
        Install with a single command and deploy your first AI agent in minutes.
        YAML-based configuration makes it easy to version control and share.
      </>
    ),
  },
  {
    title: '🔧 Flexible Tools',
    icon: '🔧',
    description: (
      <>
        Integrate with kubectl, shell commands, HTTP APIs, Slack, GitHub,
        PagerDuty, and custom MCP tools. Build agents that work with your
        existing infrastructure.
      </>
    ),
  },
  {
    title: '🤖 Multi-Provider AI',
    icon: '🤖',
    description: (
      <>
        Use OpenAI, Anthropic, Ollama, or Groq models. Switch providers easily
        and run locally or in the cloud.
      </>
    ),
  },
  {
    title: '🔒 Safe & Controlled',
    icon: '🔒',
    description: (
      <>
        Human-in-the-loop approvals, allowed command lists, and audit logging
        keep your infrastructure safe while enabling automation.
      </>
    ),
  },
  {
    title: '📊 Memory & Context',
    icon: '📊',
    description: (
      <>
        Persistent memory, RAG integration, and conversation history help agents
        learn and maintain context across interactions.
      </>
    ),
  },
  {
    title: '⚡ Production Ready',
    icon: '⚡',
    description: (
      <>
        Built with Rust for performance and reliability. Supports fleets, workflows,
        scheduling, webhooks, and complex orchestration patterns.
      </>
    ),
  },
];

function Feature({title, description, icon}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md">
        <div className={styles.featureIcon}>{icon}</div>
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
