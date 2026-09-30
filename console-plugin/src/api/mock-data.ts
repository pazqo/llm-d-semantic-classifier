import { ClassDefinition, Prediction, TrainingRun } from './types';

export const MOCK_CLASSES: ClassDefinition[] = [
  {
    name: 'SIMPLE',
    anchors: [
      'What is the capital of France?',
      'Define photosynthesis in one sentence.',
      'How many planets are in the solar system?',
    ],
  },
  {
    name: 'MEDIUM',
    anchors: [
      'Write a Python function that reads a CSV file and returns the sum of a column.',
      'Explain the difference between TCP and UDP with examples of when to use each.',
      'Create a SQL query that joins two tables and groups results by category.',
    ],
  },
  {
    name: 'COMPLEX',
    anchors: [
      'Design a microservices architecture for an e-commerce platform with inventory, orders, and payments.',
      'Refactor this monolithic application into modules with dependency injection and backward compatibility.',
      'Build a CI/CD pipeline for a multi-service app with staging, production, and database migrations.',
    ],
  },
  {
    name: 'REASONING',
    anchors: [
      'Prove by mathematical induction that the sum of 1 to n equals n(n+1)/2.',
      'Analyze the time complexity of this recursive algorithm using the Master theorem.',
      'Prove that the halting problem is undecidable using proof by contradiction.',
    ],
  },
];

export const MOCK_PREDICTIONS: Prediction[] = [
  { id: '001', text: 'What is the boiling point of water?', expected: 'SIMPLE', predicted: 'SIMPLE', confidence: 0.97 },
  { id: '002', text: 'Who wrote Romeo and Juliet?', expected: 'SIMPLE', predicted: 'SIMPLE', confidence: 0.95 },
  { id: '003', text: 'Convert 100 degrees Fahrenheit to Celsius.', expected: 'SIMPLE', predicted: 'SIMPLE', confidence: 0.93 },
  { id: '004', text: 'Write unit tests for a function that validates email addresses.', expected: 'MEDIUM', predicted: 'MEDIUM', confidence: 0.91 },
  { id: '005', text: 'Create a Dockerfile for a Python Flask application with multi-stage build.', expected: 'MEDIUM', predicted: 'MEDIUM', confidence: 0.89 },
  { id: '006', text: 'Explain how garbage collection works in Java with an example.', expected: 'MEDIUM', predicted: 'MEDIUM', confidence: 0.87 },
  { id: '007', text: 'Design a database schema for a multi-tenant SaaS app with row-level security.', expected: 'COMPLEX', predicted: 'COMPLEX', confidence: 0.92 },
  { id: '008', text: 'Architect a real-time notification system handling millions of WebSocket connections.', expected: 'COMPLEX', predicted: 'COMPLEX', confidence: 0.88 },
  { id: '009', text: 'Derive the gradient descent update rule for a two-layer neural network.', expected: 'REASONING', predicted: 'REASONING', confidence: 0.90 },
  { id: '010', text: 'Prove the correctness of Dijkstra\'s algorithm using a loop invariant.', expected: 'REASONING', predicted: 'REASONING', confidence: 0.86 },
  { id: '011', text: 'Write a bash script that finds all files larger than 100MB sorted by size.', expected: 'MEDIUM', predicted: 'SIMPLE', confidence: 0.62, alternatives: [{ className: 'MEDIUM', score: 0.58 }] },
  { id: '012', text: 'Implement authentication middleware supporting OAuth2, JWT, and API keys with RBAC.', expected: 'COMPLEX', predicted: 'MEDIUM', confidence: 0.71, alternatives: [{ className: 'COMPLEX', score: 0.67 }] },
  { id: '013', text: 'Analyze whether this concurrent program has a potential deadlock.', expected: 'REASONING', predicted: 'COMPLEX', confidence: 0.65, alternatives: [{ className: 'REASONING', score: 0.61 }] },
  { id: '014', text: 'Create a Kubernetes operator that manages custom database clusters with automated failover.', expected: 'COMPLEX', predicted: 'MEDIUM', confidence: 0.59, alternatives: [{ className: 'COMPLEX', score: 0.56 }] },
];

export const MOCK_TRAINING_HISTORY: TrainingRun[] = [
  {
    id: 'run-001',
    startedAt: '2026-09-20T10:00:00Z',
    completedAt: '2026-09-20T10:05:32Z',
    status: 'completed',
    modelName: 'cnuland/llm-d-sc-complexity',
    anchorSetVersion: 'scr-default-anchors-v1',
    metrics: { accuracy: 0.667, macroF1: 0.632 },
  },
  {
    id: 'run-002',
    startedAt: '2026-09-23T14:30:00Z',
    completedAt: '2026-09-23T14:35:18Z',
    status: 'completed',
    modelName: 'cnuland/llm-d-sc-complexity',
    anchorSetVersion: 'scr-default-anchors-v2',
    metrics: { accuracy: 0.786, macroF1: 0.752 },
  },
];
