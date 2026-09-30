import { ClassifierApi } from './classifier-api';
import { ClassDefinition, ClassifyResult, EvaluationResult, Prediction, TrainingRun } from './types';

declare const process: { env: { CLASSIFIER_API_URL?: string } };
const API_BASE = process.env.CLASSIFIER_API_URL || '';

export const realClassifierApi: ClassifierApi = {
  async getEvaluation(): Promise<EvaluationResult> {
    const [defRes, evalRes] = await Promise.all([
      fetch(`${API_BASE}/api/v1/definition`),
      fetch(`${API_BASE}/api/v1/evaluate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({}),
      }),
    ]);

    if (!defRes.ok) throw new Error(`Definition fetch failed: ${defRes.statusText}`);
    if (!evalRes.ok) throw new Error(`Evaluation failed: ${evalRes.statusText}`);

    const definition = await defRes.json();
    const evaluation = await evalRes.json();

    const classes: ClassDefinition[] = definition.labels.map((label: string) => ({
      name: label,
      anchors: definition.anchors[label] || [],
    }));

    const predictions: Prediction[] = evaluation.predictions.map((p: any, i: number) => ({
      id: String(i + 1).padStart(3, '0'),
      text: p.text,
      expected: p.expected,
      predicted: p.predicted,
      confidence: p.confidence,
      alternatives: p.alternatives?.map((a: any) => ({
        className: a.label,
        score: a.score,
      })),
    }));

    return {
      modelName: definition.model_repo,
      anchorSetVersion: definition.taxonomy_revision,
      datasetName: evaluation.dataset,
      predictions,
      classes,
      metrics: evaluation.label_metrics.map((m: any) => ({
        name: m.label,
        precision: m.precision,
        recall: m.recall,
        f1: m.f1,
        support: m.support,
      })),
      accuracy: evaluation.accuracy,
      macroF1: evaluation.macro_f1,
      totalExamples: evaluation.total,
      errorCount: evaluation.error_count,
    };
  },

  async getTrainingHistory(): Promise<TrainingRun[]> {
    return [];
  },

  async startRetraining(classes: ClassDefinition[]): Promise<TrainingRun> {
    const anchors: Record<string, string[]> = {};
    const labels: string[] = [];
    for (const cls of classes) {
      labels.push(cls.name);
      anchors[cls.name] = cls.anchors;
    }

    const res = await fetch(`${API_BASE}/api/v1/definition/anchors`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ labels, anchors }),
    });

    if (!res.ok) throw new Error(`Anchor update failed: ${res.statusText}`);
    const updated = await res.json();

    return {
      id: `run-${Date.now()}`,
      startedAt: new Date().toISOString(),
      completedAt: new Date().toISOString(),
      status: 'completed',
      modelName: updated.model_repo,
      anchorSetVersion: updated.taxonomy_revision,
    };
  },

  async classify(text: string): Promise<ClassifyResult> {
    const res = await fetch(`${API_BASE}/api/v1/classify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text }),
    });
    if (!res.ok) throw new Error(`Classification failed: ${res.statusText}`);
    const data = await res.json();
    return {
      text: data.text,
      predicted: data.predicted,
      confidence: data.confidence,
      labelScores: data.label_scores.map((s: any) => ({ label: s.label, score: s.score })),
      anchorDetails: data.anchor_details.map((a: any) => ({
        label: a.label,
        anchorText: a.anchor_text,
        similarity: a.similarity,
      })),
    };
  },

  async getTrainingStatus(runId: string): Promise<TrainingRun> {
    return {
      id: runId,
      startedAt: new Date().toISOString(),
      completedAt: new Date().toISOString(),
      status: 'completed',
      modelName: '',
      anchorSetVersion: '',
    };
  },
};
