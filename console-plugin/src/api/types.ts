export interface ClassDefinition {
  name: string;
  anchors: string[];
  disabledAnchors?: string[];
  disabled?: boolean;
}

export interface Prediction {
  id: string;
  text: string;
  expected: string;
  predicted: string;
  confidence: number;
  alternatives?: { className: string; score: number }[];
}

export interface ClassMetric {
  name: string;
  precision: number;
  recall: number;
  f1: number;
  support: number;
}

export interface EvaluationResult {
  modelName: string;
  anchorSetVersion: string;
  datasetName: string;
  predictions: Prediction[];
  classes: ClassDefinition[];
  metrics: ClassMetric[];
  accuracy: number;
  macroF1: number;
  totalExamples: number;
  errorCount: number;
}

export interface ClassifyResult {
  text: string;
  predicted: string;
  confidence: number;
  labelScores: { label: string; score: number }[];
  anchorDetails: { label: string; anchorText: string; similarity: number }[];
}

export interface TrainingRun {
  id: string;
  startedAt: string;
  completedAt?: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  modelName: string;
  anchorSetVersion: string;
  metrics?: {
    accuracy: number;
    macroF1: number;
  };
}
