//! Reusable evaluation logic for anchor-based classifiers.
//!
//! Provides dataset loading, per-example classification with softmax confidence,
//! and full evaluation reporting (confusion matrix, per-label P/R/F1). Used by
//! both the `eval-classifier` CLI and the REST API server.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::embedding::Embedder;
use crate::ranker::{anchor_rank, cosine_similarity, AnchorSet};

/// A single labelled example from an eval dataset (JSONL format).
#[derive(Debug, Clone, Deserialize)]
pub struct EvalExample {
    pub text: String,
    pub tier: String,
    #[serde(default)]
    pub hard: bool,
}

/// Classification result for one example.
#[derive(Debug, Clone, Serialize)]
pub struct ExamplePrediction {
    pub text: String,
    pub expected: String,
    pub predicted: String,
    pub confidence: f64,
    pub label_scores: Vec<(String, f64)>,
}

/// Per-label precision, recall, F1, and support.
#[derive(Debug, Clone, Serialize)]
pub struct LabelMetrics {
    pub label: String,
    pub precision: f64,
    pub recall: f64,
    pub f1: f64,
    pub support: usize,
}

/// Full evaluation report.
#[derive(Debug, Clone, Serialize)]
pub struct EvalReport {
    pub labels: Vec<String>,
    pub predictions: Vec<ExamplePrediction>,
    pub confusion: Vec<Vec<usize>>,
    pub label_metrics: Vec<LabelMetrics>,
    pub accuracy: f64,
    pub macro_f1: f64,
    pub total: usize,
    pub error_count: usize,
}

/// Parse a JSONL dataset string into eval examples.
pub fn load_dataset(raw: &str) -> Vec<EvalExample> {
    raw.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("each JSONL line must be a valid EvalExample"))
        .collect()
}

/// Classify one pre-embedded input against pre-embedded anchors.
///
/// Returns `(predicted_label, softmax_confidence, all_label_scores)`.
/// `all_label_scores` are sorted descending by score (same order as
/// `anchor_rank`).
pub fn classify_example(
    embedding: &[f32],
    anchors: &[AnchorSet],
    top_k: usize,
) -> (String, f64, Vec<(String, f64)>) {
    let scores = anchor_rank(embedding, anchors, top_k);
    let predicted = scores[0].0.clone();

    // Softmax confidence over label scores.
    let max_score = scores
        .iter()
        .map(|s| s.1)
        .fold(f64::MIN, f64::max);
    let exps: Vec<f64> = scores.iter().map(|s| (s.1 - max_score).exp()).collect();
    let sum: f64 = exps.iter().sum();
    let confidence = exps[0] / sum;

    (predicted, confidence, scores)
}

/// Compute per-label metrics from a confusion map.
pub fn compute_label_metrics(
    labels: &[String],
    confusion: &BTreeMap<(String, String), usize>,
) -> Vec<LabelMetrics> {
    labels
        .iter()
        .map(|l| {
            let tp = *confusion.get(&(l.clone(), l.clone())).unwrap_or(&0) as f64;
            let fp: f64 = labels
                .iter()
                .map(|t| *confusion.get(&(t.clone(), l.clone())).unwrap_or(&0) as f64)
                .sum::<f64>()
                - tp;
            let fn_: f64 = labels
                .iter()
                .map(|p| *confusion.get(&(l.clone(), p.clone())).unwrap_or(&0) as f64)
                .sum::<f64>()
                - tp;
            let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };
            let recall = if tp + fn_ > 0.0 {
                tp / (tp + fn_)
            } else {
                0.0
            };
            let f1 = if precision + recall > 0.0 {
                2.0 * precision * recall / (precision + recall)
            } else {
                0.0
            };
            LabelMetrics {
                label: l.clone(),
                precision,
                recall,
                f1,
                support: (tp + fn_) as usize,
            }
        })
        .collect()
}

/// Run a full evaluation: embed each example, classify, build confusion matrix
/// and per-label metrics.
pub fn run_evaluation(
    embedder: &Embedder,
    anchors: &[AnchorSet],
    labels: &[String],
    top_k: usize,
    dataset: &[EvalExample],
) -> Result<EvalReport, crate::embedding::EmbeddingError> {
    let mut confusion: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut predictions = Vec::with_capacity(dataset.len());
    let mut correct = 0usize;

    for example in dataset {
        let emb = embedder.embed(&example.text)?;
        let (predicted, confidence, label_scores) = classify_example(&emb, anchors, top_k);

        if predicted == example.tier {
            correct += 1;
        }
        *confusion
            .entry((example.tier.clone(), predicted.clone()))
            .or_insert(0) += 1;

        predictions.push(ExamplePrediction {
            text: example.text.clone(),
            expected: example.tier.clone(),
            predicted,
            confidence,
            label_scores,
        });
    }

    let total = dataset.len();
    let accuracy = if total > 0 {
        correct as f64 / total as f64
    } else {
        0.0
    };

    let label_metrics = compute_label_metrics(labels, &confusion);
    let macro_f1 = if label_metrics.is_empty() {
        0.0
    } else {
        label_metrics.iter().map(|m| m.f1).sum::<f64>() / label_metrics.len() as f64
    };

    let confusion_matrix = labels
        .iter()
        .map(|t| {
            labels
                .iter()
                .map(|p| *confusion.get(&(t.clone(), p.clone())).unwrap_or(&0))
                .collect()
        })
        .collect();

    Ok(EvalReport {
        labels: labels.to_vec(),
        predictions,
        confusion: confusion_matrix,
        label_metrics,
        accuracy,
        macro_f1,
        total,
        error_count: total - correct,
    })
}

/// Compute per-anchor cosine similarities for one input embedding.
///
/// Returns a vec of `(label, anchor_text, similarity)` sorted descending by
/// similarity.
pub fn anchor_contributions(
    embedding: &[f32],
    anchors: &[AnchorSet],
    anchor_texts: &BTreeMap<String, Vec<String>>,
) -> Vec<(String, String, f64)> {
    let mut details = Vec::new();
    for set in anchors {
        if let Some(texts) = anchor_texts.get(&set.label) {
            for (i, vec) in set.vectors.iter().enumerate() {
                let sim = cosine_similarity(embedding, vec);
                details.push((set.label.clone(), texts[i].clone(), sim));
            }
        }
    }
    details.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    details
}
