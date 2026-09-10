//! Evaluate context-dependent, per-turn classification.
//!
//! This harness deliberately evaluates three representations of every target
//! turn:
//!
//!   current-only    classify only the current user turn;
//!   full-history    serialize the whole transcript, allowing model truncation;
//!   bounded-history keep the current turn and add as much recent history as
//!                   fits in the tokenizer's configured input window.
//!   split-weighted  embed current and history independently, then combine
//!                   their vectors with configurable weights.
//!
//! Usage:
//!   cargo run --release --bin eval-multiturn -- \
//!     --model artifacts/models/complexity \
//!     --classifier classifiers/complexity.json \
//!     --dataset evals/datasets/multiturn-complexity-seed.jsonl \
//!     --mode all \
//!     --json /tmp/multiturn-report.json

use std::collections::BTreeMap;
use std::path::Path;

use llm_d_sc::classify::{
    CandleClassifier, ClassificationInput, ClassifierRuntime, ClassifyStatus,
};
use llm_d_sc::taxonomy::{ClassifierDefinition, DEFAULT_CLASSIFIER};
use llm_d_sc::tokenizer::Tokenizer;
use serde::Deserialize;
use serde_json::{json, Value};

const DEFAULT_DATASET: &str = "evals/datasets/multiturn-complexity-seed.jsonl";

#[derive(Debug, Deserialize)]
struct Turn {
    role: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct Record {
    conversation_id: String,
    turn_id: usize,
    #[serde(default)]
    history: Vec<Turn>,
    current_user_turn: String,
    gold_tier: String,
    #[serde(default)]
    context_required: bool,
    #[serde(default)]
    case: String,
    #[serde(default)]
    expected_status: Option<String>,
}

#[derive(Clone, Copy, Debug)]
enum Mode {
    CurrentOnly,
    FullHistory,
    BoundedHistory,
    SplitWeighted,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Mode::CurrentOnly => "current-only",
            Mode::FullHistory => "full-history",
            Mode::BoundedHistory => "bounded-history",
            Mode::SplitWeighted => "split-weighted",
        }
    }
}

fn arg(name: &str, default: Option<&str>) -> String {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
        .or_else(|| default.map(str::to_string))
        .unwrap_or_else(|| panic!("missing required argument {name}"))
}

fn optional_arg(name: &str) -> Option<String> {
    std::env::args()
        .collect::<Vec<_>>()
        .iter()
        .position(|a| a == name)
        .and_then(|i| std::env::args().nth(i + 1))
}

fn tokenizer_max_length(model_dir: &str) -> usize {
    let path = Path::new(model_dir).join("tokenizer.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read tokenizer {}: {e}", path.display()));
    let root: Value = serde_json::from_str(&raw).expect("tokenizer.json must be valid JSON");
    root["truncation"]["max_length"].as_u64().unwrap_or(256) as usize
}

fn model_max_position_embeddings(model_dir: &str) -> usize {
    let path = Path::new(model_dir).join("config.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read model config {}: {e}", path.display()));
    let root: Value = serde_json::from_str(&raw).expect("config.json must be valid JSON");
    root["max_position_embeddings"].as_u64().unwrap_or(256) as usize
}

fn format_turn(turn: &Turn) -> String {
    format!("{}: {}\n", turn.role, turn.text)
}

fn format_current(text: &str) -> String {
    format!("Current user: {text}\n")
}

fn full_history(record: &Record) -> String {
    let mut out = String::new();
    for turn in &record.history {
        out.push_str(&format_turn(turn));
    }
    out.push_str(&format_current(&record.current_user_turn));
    out
}

fn history_only(record: &Record) -> String {
    record.history.iter().map(format_turn).collect()
}

fn weighted_embedding(
    current: &[f32],
    history: &[f32],
    current_weight: f32,
    history_weight: f32,
) -> Vec<f32> {
    assert_eq!(
        current.len(),
        history.len(),
        "embedding dimensions must match"
    );
    let norm = (current_weight * current_weight + history_weight * history_weight).sqrt();
    assert!(
        norm > 0.0,
        "at least one split-context weight must be positive"
    );
    let mut combined: Vec<f32> = current
        .iter()
        .zip(history)
        .map(|(c, h)| (current_weight * c + history_weight * h) / norm)
        .collect();
    let combined_norm = combined.iter().map(|x| x * x).sum::<f32>().sqrt();
    if combined_norm > 0.0 {
        for value in &mut combined {
            *value /= combined_norm;
        }
    }
    combined
}

/// Group prior messages into complete conversational exchanges. A user turn
/// owns the following assistant response; unusual leading/system messages are
/// kept as their own group rather than being silently dropped.
fn history_groups(history: &[Turn]) -> Vec<Vec<&Turn>> {
    let mut groups: Vec<Vec<&Turn>> = Vec::new();
    for turn in history {
        if turn.role == "user" || groups.is_empty() {
            groups.push(vec![turn]);
        } else {
            groups.last_mut().expect("history group exists").push(turn);
        }
    }
    groups
}

/// Build a context with the current turn first, then the newest complete prior
/// turns that fit. Putting the current turn first is intentional: the resident
/// tokenizer right-truncates over-length input, so this guarantees that history
/// is discarded before the current request is discarded.
fn bounded_history(
    record: &Record,
    tokenizer_for_counting: &Tokenizer,
    max_tokens: usize,
) -> (String, usize) {
    let current = format_current(&record.current_user_turn);
    let current_tokens = tokenizer_for_counting
        .tokenize(&current)
        .expect("current turn must tokenize")
        .len();
    let mut out = current.clone();
    let mut used = 0usize;

    if current_tokens < max_tokens {
        let groups = history_groups(&record.history);
        let mut selected: Vec<Vec<&Turn>> = Vec::new();
        for group in groups.iter().rev() {
            let candidate: String = group.iter().map(|turn| format_turn(turn)).collect();
            let trial = format!("{out}{candidate}");
            let count = tokenizer_for_counting
                .tokenize(&trial)
                .expect("conversation turn must tokenize")
                .len();
            if count <= max_tokens {
                selected.push(group.clone());
            } else {
                break;
            }
        }
        selected.reverse();
        for group in selected {
            for turn in group {
                out.push_str(&format_turn(turn));
            }
        }
        used = out.matches("user:").count().saturating_sub(1);
    }

    // The current turn itself may exceed the model window. The production
    // tokenizer will apply its normal deterministic right truncation in that
    // case; no history is included because it cannot help preserve the input.
    (out, used)
}

fn status_name(status: ClassifyStatus) -> &'static str {
    match status {
        ClassifyStatus::Ok => "OK",
        ClassifyStatus::Abstain => "ABSTAIN",
        ClassifyStatus::Error => "ERROR",
    }
}

fn modes(requested: &str) -> Vec<Mode> {
    match requested {
        "current-only" => vec![Mode::CurrentOnly],
        "full-history" => vec![Mode::FullHistory],
        "bounded-history" => vec![Mode::BoundedHistory],
        "split-weighted" => vec![Mode::SplitWeighted],
        "all" => vec![
            Mode::CurrentOnly,
            Mode::FullHistory,
            Mode::BoundedHistory,
            Mode::SplitWeighted,
        ],
        other => panic!(
            "unknown --mode {other}; use current-only, full-history, bounded-history, split-weighted, or all"
        ),
    }
}

fn main() {
    let model_dir = arg("--model", Some("artifacts/models/complexity"));
    let classifier_spec = arg("--classifier", Some(DEFAULT_CLASSIFIER));
    let dataset_path = arg("--dataset", Some(DEFAULT_DATASET));
    let mode_arg = arg("--mode", Some("all"));
    let json_path = arg("--json", Some(""));
    let current_weight = optional_arg("--current-weight")
        .map(|value| {
            value
                .parse::<f32>()
                .expect("--current-weight must be numeric")
        })
        .unwrap_or(0.4);
    let history_weight = optional_arg("--history-weight")
        .map(|value| {
            value
                .parse::<f32>()
                .expect("--history-weight must be numeric")
        })
        .unwrap_or(0.6);
    assert!(
        current_weight >= 0.0 && history_weight >= 0.0,
        "split-context weights must be non-negative"
    );
    assert!(
        current_weight > 0.0 || history_weight > 0.0,
        "at least one split-context weight must be positive"
    );
    let max_override = optional_arg("--max-tokens").map(|value| {
        value
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("--max-tokens must be a positive integer"))
    });

    let definition = ClassifierDefinition::resolve(&classifier_spec)
        .unwrap_or_else(|e| panic!("resolve classifier {classifier_spec}: {e}"));
    let classifier =
        CandleClassifier::from_modelcar_with(Path::new(&model_dir), definition.clone())
            .unwrap_or_else(|e| panic!("load classifier from {model_dir}: {e}"));
    let tokenizer_path = Path::new(&model_dir).join("tokenizer.json");
    let configured_max_tokens = tokenizer_max_length(&model_dir);
    let max_tokens = max_override.unwrap_or(configured_max_tokens);
    let model_max_tokens = model_max_position_embeddings(&model_dir);
    assert!(max_tokens > 0, "--max-tokens must be positive");
    assert!(
        max_tokens <= model_max_tokens,
        "requested --max-tokens={max_tokens} exceeds model max_position_embeddings={model_max_tokens}"
    );
    let tokenizer = match max_override {
        Some(_) => Tokenizer::load_with_max_length(&tokenizer_path, max_tokens),
        None => Tokenizer::load(&tokenizer_path),
    }
    .expect("load resident tokenizer");
    // The production tokenizer may truncate at the requested evaluation
    // window. Use a separate tokenizer at the model's full positional capacity
    // when deciding which history groups fit, otherwise a long candidate would
    // appear to fit simply because it had already been truncated during the
    // fit check.
    let counting_tokenizer = Tokenizer::load_with_max_length(&tokenizer_path, model_max_tokens)
        .expect("load counting tokenizer");

    let raw = std::fs::read_to_string(&dataset_path)
        .unwrap_or_else(|e| panic!("read dataset {dataset_path}: {e}"));
    let records: Vec<Record> = raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("dataset line must be valid JSON"))
        .collect();

    let mut all_results = Vec::new();
    for mode in modes(&mode_arg) {
        let mut correct = 0usize;
        let mut status_correct = 0usize;
        let mut context_cases = 0usize;
        let mut context_correct = 0usize;
        let mut history_turns = 0usize;
        let mut by_case: BTreeMap<String, (usize, usize)> = BTreeMap::new();

        for record in &records {
            let (text, used_history, result) = match mode {
                Mode::CurrentOnly => {
                    let text = record.current_user_turn.clone();
                    let result = classifier.classify(ClassificationInput {
                        text: text.clone(),
                        requested_signals: vec![definition.signal.clone()],
                        session_metadata: Default::default(),
                    });
                    (text, 0, result)
                }
                Mode::FullHistory => {
                    let text = full_history(record);
                    let result = classifier.classify(ClassificationInput {
                        text: text.clone(),
                        requested_signals: vec![definition.signal.clone()],
                        session_metadata: Default::default(),
                    });
                    (text, record.history.len(), result)
                }
                Mode::BoundedHistory => {
                    let (text, used) = bounded_history(record, &counting_tokenizer, max_tokens);
                    let result = classifier.classify(ClassificationInput {
                        text: text.clone(),
                        requested_signals: vec![definition.signal.clone()],
                        session_metadata: Default::default(),
                    });
                    (text, used, result)
                }
                Mode::SplitWeighted => {
                    let current_text = format_current(&record.current_user_turn);
                    let history_text = history_only(record);
                    let result = match (
                        classifier.embed_text(&current_text),
                        classifier.embed_text(&history_text),
                    ) {
                        (Ok(current), Ok(history)) => classifier.rank_embedding(
                            &weighted_embedding(&current, &history, current_weight, history_weight),
                        ),
                        (Err(e), _) | (_, Err(e)) => Err(e),
                    };
                    (
                        format!("{current_text}{history_text}"),
                        record.history.len(),
                        result,
                    )
                }
            };
            history_turns += used_history;

            let (predicted, observed_status, error) = match result {
                Ok(result) => (
                    result.ranked.first().map(|s| s.id.clone()),
                    status_name(result.status).to_string(),
                    None,
                ),
                Err(e) => (None, "ERROR".to_string(), Some(e.to_string())),
            };
            let label_correct = predicted.as_deref() == Some(record.gold_tier.as_str());
            let expected_status = record.expected_status.as_deref().unwrap_or("OK");
            let status_ok = observed_status == expected_status;
            if label_correct {
                correct += 1;
            }
            if status_ok {
                status_correct += 1;
            }
            if record.context_required {
                context_cases += 1;
                if label_correct {
                    context_correct += 1;
                }
            }
            let entry = by_case.entry(record.case.clone()).or_default();
            entry.1 += 1;
            if label_correct {
                entry.0 += 1;
            }

            all_results.push(json!({
                "mode": mode.name(),
                "conversation_id": record.conversation_id,
                "turn_id": record.turn_id,
                "case": record.case,
                "context_required": record.context_required,
                "history_turns_available": record.history.len(),
                "history_turns_used": used_history,
                "input_tokens": tokenizer.tokenize(&text).expect("assembled context must tokenize").len(),
                "max_tokens": max_tokens,
                "current_weight": current_weight,
                "history_weight": history_weight,
                "gold_tier": record.gold_tier,
                "predicted_tier": predicted,
                "label_correct": label_correct,
                "expected_status": expected_status,
                "observed_status": observed_status,
                "status_correct": status_ok,
                "error": error,
            }));
        }

        let n = records.len();
        println!(
            "{:<17} accuracy {:>4}/{:<4} ({:.1}%) | status {:>4}/{:<4} | context {:>4}/{:<4} | avg history turns {:.2}",
            mode.name(),
            correct,
            n,
            100.0 * correct as f64 / n as f64,
            status_correct,
            n,
            context_correct,
            context_cases,
            history_turns as f64 / n as f64,
        );
        for (case, (ok, total)) in &by_case {
            println!("  {case:<32} {ok}/{total}");
        }
    }

    if !json_path.is_empty() {
        let report = json!({
            "classifier": classifier_spec,
            "model": model_dir,
            "dataset": dataset_path,
            "max_tokens": max_tokens,
            "configured_max_tokens": configured_max_tokens,
            "model_max_position_embeddings": model_max_tokens,
            "records": records.len(),
            "results": all_results,
        });
        std::fs::write(&json_path, serde_json::to_string_pretty(&report).unwrap())
            .unwrap_or_else(|e| panic!("write report {json_path}: {e}"));
        println!("report written to {json_path}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_history_keeps_current_turn_before_history() {
        let tokenizer = Tokenizer::load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modelcar/tokenizer.json"),
        )
        .expect("fixture tokenizer must load");
        let record = Record {
            conversation_id: "test".into(),
            turn_id: 3,
            history: vec![
                Turn {
                    role: "user".into(),
                    text: "old context that should be displaced".into(),
                },
                Turn {
                    role: "assistant".into(),
                    text: "newest context that should fit".into(),
                },
            ],
            current_user_turn: "CURRENT-TURN-MUST-BE-PRESERVED".into(),
            gold_tier: "SIMPLE".into(),
            context_required: true,
            case: "test".into(),
            expected_status: None,
        };

        let (assembled, used) = bounded_history(&record, &tokenizer, 32);
        assert!(assembled.starts_with("Current user: CURRENT-TURN-MUST-BE-PRESERVED"));
        assert!(used >= 1, "the newest history turn should fit");
        assert!(
            assembled.find("old context").unwrap() < assembled.find("newest context").unwrap(),
            "prior user and assistant messages should remain chronological"
        );
        assert!(
            assembled.contains("newest context that should fit"),
            "bounded history should prefer the newest complete turn"
        );
    }
}
