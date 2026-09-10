//! Evaluate ClassifyChain strategies through the real gRPC API.
//!
//! Usage:
//!   cargo run --release --bin eval-chain-strategies -- \
//!     --model artifacts/models/complexity \
//!     --classifier classifiers/complexity.json \
//!     --dataset evals/datasets/multiturn-complexity-ambiguous.jsonl

use std::path::Path;

use llm_d_sc::classify::{load_and_warm_modelcar, ClassifierRuntime};
use llm_d_sc::grpc::classify::generated::{conversation_message, ConversationStrategy};
use llm_d_sc::grpc::classify::{
    ClassifyChainRequest, ClassifyClient, ClassifyServer, ConversationMessage,
};
use llm_d_sc::taxonomy::{ClassifierDefinition, DEFAULT_CLASSIFIER};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Turn {
    role: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct Record {
    conversation_id: String,
    turn_id: usize,
    history: Vec<Turn>,
    current_user_turn: String,
    gold_tier: String,
}

fn arg(name: &str, default: &str) -> String {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
        .unwrap_or_else(|| default.to_string())
}

fn role(value: &str) -> i32 {
    match value.to_ascii_lowercase().as_str() {
        "system" => conversation_message::Role::System as i32,
        "user" => conversation_message::Role::User as i32,
        "assistant" => conversation_message::Role::Assistant as i32,
        other => panic!("unsupported conversation role '{other}'"),
    }
}

fn request(record: &Record, strategy: ConversationStrategy, suffix: &str) -> ClassifyChainRequest {
    let mut messages: Vec<ConversationMessage> = record
        .history
        .iter()
        .map(|turn| ConversationMessage {
            role: role(&turn.role),
            content: turn.text.clone(),
        })
        .collect();
    messages.push(ConversationMessage {
        role: conversation_message::Role::User as i32,
        content: record.current_user_turn.clone(),
    });
    ClassifyChainRequest {
        request_id: format!(
            "chain-{}-{}-{suffix}",
            record.conversation_id, record.turn_id
        ),
        session_id: record.conversation_id.clone(),
        messages,
        signals: Vec::new(),
        strategy: strategy as i32,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_dir = arg("--model", "artifacts/models/complexity");
    let classifier_spec = arg("--classifier", DEFAULT_CLASSIFIER);
    let dataset_path = arg(
        "--dataset",
        "evals/datasets/multiturn-complexity-ambiguous.jsonl",
    );
    let definition = ClassifierDefinition::resolve(&classifier_spec)?;
    let classifier = load_and_warm_modelcar(Path::new(&model_dir))?;
    if classifier.metadata().signal != definition.signal {
        return Err(format!(
            "model signal '{}' does not match classifier signal '{}'",
            classifier.metadata().signal,
            definition.signal
        )
        .into());
    }
    let raw = std::fs::read_to_string(&dataset_path)?;
    let records: Vec<Record> = raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;

    let server = ClassifyServer::bind_with_classifier("127.0.0.1:0", classifier)?;
    let mut client = ClassifyClient::connect(server.local_addr())?;
    for strategy in [
        ConversationStrategy::LastMessage,
        ConversationStrategy::FullHistory,
    ] {
        let mut correct = 0usize;
        for record in &records {
            let response =
                client.classify_chain(request(record, strategy, strategy.as_str_name()))?;
            let predicted = response.ranked.first().map(|signal| signal.label.as_str());
            if predicted == Some(record.gold_tier.as_str()) {
                correct += 1;
            }
        }
        println!(
            "{:<13} accuracy {:>4}/{:<4} ({:.1}%)",
            strategy.as_str_name(),
            correct,
            records.len(),
            100.0 * correct as f64 / records.len() as f64
        );
    }
    Ok(())
}
