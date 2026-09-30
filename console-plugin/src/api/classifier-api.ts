import { ClassDefinition, ClassifyResult, ClassMetric, EvaluationResult, Prediction, TrainingRun } from './types';
import { MOCK_CLASSES, MOCK_PREDICTIONS, MOCK_TRAINING_HISTORY } from './mock-data';

function computeMetrics(classes: ClassDefinition[], predictions: Prediction[]): ClassMetric[] {
  return classes.map(({ name }) => {
    const tp = predictions.filter((p) => p.expected === name && p.predicted === name).length;
    const fp = predictions.filter((p) => p.expected !== name && p.predicted === name).length;
    const support = predictions.filter((p) => p.expected === name).length;
    const precision = tp + fp > 0 ? tp / (tp + fp) : 0;
    const recall = support > 0 ? tp / support : 0;
    const f1 = precision + recall > 0 ? (2 * precision * recall) / (precision + recall) : 0;
    return { name, precision, recall, f1, support };
  });
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export interface ClassifierApi {
  getEvaluation(): Promise<EvaluationResult>;
  getTrainingHistory(): Promise<TrainingRun[]>;
  startRetraining(classes: ClassDefinition[]): Promise<TrainingRun>;
  getTrainingStatus(runId: string): Promise<TrainingRun>;
  classify(text: string): Promise<ClassifyResult>;
}

const trainingHistory = [...MOCK_TRAINING_HISTORY];
let nextRunId = 3;
const activeRuns = new Map<string, TrainingRun>();

export const mockClassifierApi: ClassifierApi = {
  async getEvaluation(): Promise<EvaluationResult> {
    await delay(300);
    const classes = MOCK_CLASSES;
    const predictions = MOCK_PREDICTIONS;
    const metrics = computeMetrics(classes, predictions);
    const correct = predictions.filter((p) => p.expected === p.predicted).length;
    const macroF1 = metrics.reduce((sum, m) => sum + m.f1, 0) / metrics.length;
    return {
      modelName: 'cnuland/llm-d-sc-complexity',
      anchorSetVersion: 'scr-default-anchors-v1',
      datasetName: 'complexity-eval-14',
      predictions,
      classes,
      metrics,
      accuracy: correct / predictions.length,
      macroF1,
      totalExamples: predictions.length,
      errorCount: predictions.length - correct,
    };
  },

  async getTrainingHistory(): Promise<TrainingRun[]> {
    await delay(200);
    return [...trainingHistory].reverse();
  },

  async startRetraining(_classes: ClassDefinition[]): Promise<TrainingRun> {
    const runId = `run-${String(nextRunId++).padStart(3, '0')}`;
    const run: TrainingRun = {
      id: runId,
      startedAt: new Date().toISOString(),
      status: 'running',
      modelName: 'cnuland/llm-d-sc-complexity',
      anchorSetVersion: `scr-default-anchors-v${nextRunId + 1}`,
    };
    activeRuns.set(runId, run);
    trainingHistory.push(run);

    delay(3000).then(() => {
      const completed: TrainingRun = {
        ...run,
        status: 'completed',
        completedAt: new Date().toISOString(),
        metrics: {
          accuracy: 0.75 + Math.random() * 0.1,
          macroF1: 0.709 + Math.random() * 0.1,
        },
      };
      activeRuns.set(runId, completed);
      const idx = trainingHistory.findIndex((r) => r.id === runId);
      if (idx >= 0) trainingHistory[idx] = completed;
    });

    return run;
  },

  async getTrainingStatus(runId: string): Promise<TrainingRun> {
    await delay(100);
    const run = activeRuns.get(runId);
    if (!run) throw new Error(`Training run ${runId} not found`);
    return run;
  },

  async classify(text: string): Promise<ClassifyResult> {
    await delay(500);
    const labels = MOCK_CLASSES.map((c) => c.name);
    const scores = labels.map(() => Math.random());
    const total = scores.reduce((a, b) => a + b, 0);
    const normalized = scores.map((s) => s / total);
    const maxIdx = normalized.indexOf(Math.max(...normalized));
    return {
      text,
      predicted: labels[maxIdx],
      confidence: normalized[maxIdx],
      labelScores: labels.map((label, i) => ({ label, score: normalized[i] })),
      anchorDetails: [],
    };
  },
};
