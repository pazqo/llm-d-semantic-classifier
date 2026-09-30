//! HTTP REST API server for the semantic classifier.
//!
//! Exposes classifier definition, anchor editing, single-text classification
//! with per-anchor detail, and full evaluation against held-out datasets.
//! Designed as a dev/ops companion to the production gRPC server.
//!
//! Usage:
//!   LLM_D_SC_MODEL_DIR=artifacts/models/complexity cargo run --bin llm-d-sc-api

use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

use llm_d_sc::embedding::Embedder;
use llm_d_sc::eval;
use llm_d_sc::ranker::AnchorSet;
use llm_d_sc::taxonomy::ClassifierDefinition;

struct AppState {
    embedder: Arc<Embedder>,
    classifier: RwLock<ClassifierState>,
    datasets_dir: PathBuf,
}

struct ClassifierState {
    definition: ClassifierDefinition,
    embedded_anchors: Vec<AnchorSet>,
}

// --- Request / Response types ---

#[derive(Serialize)]
struct DefinitionResponse {
    classifier_id: String,
    signal: String,
    taxonomy_revision: String,
    model_repo: String,
    model_revision: String,
    top_k: usize,
    labels: Vec<String>,
    anchors: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct UpdateAnchorsRequest {
    labels: Vec<String>,
    anchors: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct AddAnchorRequest {
    text: String,
}

#[derive(Deserialize)]
struct ClassifyRequest {
    text: String,
}

#[derive(Serialize)]
struct ClassifyResponse {
    text: String,
    predicted: String,
    confidence: f64,
    label_scores: Vec<LabelScore>,
    anchor_details: Vec<AnchorDetail>,
}

#[derive(Serialize)]
struct LabelScore {
    label: String,
    score: f64,
}

#[derive(Serialize)]
struct AnchorDetail {
    label: String,
    anchor_text: String,
    similarity: f64,
}

#[derive(Deserialize)]
struct EvaluateRequest {
    dataset: Option<String>,
}

#[derive(Serialize)]
struct EvaluateResponse {
    dataset: String,
    total: usize,
    accuracy: f64,
    macro_f1: f64,
    error_count: usize,
    labels: Vec<String>,
    confusion: Vec<Vec<usize>>,
    label_metrics: Vec<LabelMetricJson>,
    predictions: Vec<PredictionJson>,
}

#[derive(Serialize)]
struct LabelMetricJson {
    label: String,
    precision: f64,
    recall: f64,
    f1: f64,
    support: usize,
}

#[derive(Serialize)]
struct PredictionJson {
    text: String,
    expected: String,
    predicted: String,
    confidence: f64,
    alternatives: Vec<LabelScore>,
}

#[derive(Serialize)]
struct DatasetsResponse {
    datasets: Vec<DatasetInfo>,
}

#[derive(Serialize)]
struct DatasetInfo {
    name: String,
    count: usize,
}

// --- Helpers ---

fn definition_response(def: &ClassifierDefinition) -> DefinitionResponse {
    DefinitionResponse {
        classifier_id: def.classifier_id.clone(),
        signal: def.signal.clone(),
        taxonomy_revision: def.taxonomy_revision.clone(),
        model_repo: def.model_repo.clone(),
        model_revision: def.model_revision.clone(),
        top_k: def.top_k,
        labels: def.labels.clone(),
        anchors: def.anchors.clone(),
    }
}

// --- Handlers ---

async fn get_definition(
    State(state): State<Arc<AppState>>,
) -> Json<DefinitionResponse> {
    let cls = state.classifier.read().await;
    Json(definition_response(&cls.definition))
}

async fn update_anchors(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateAnchorsRequest>,
) -> Result<Json<DefinitionResponse>, (StatusCode, String)> {
    let mut new_def = {
        let cls = state.classifier.read().await;
        cls.definition.clone()
    };
    new_def.labels = req.labels;
    new_def.anchors = req.anchors;

    let embedder = state.embedder.clone();
    let def_for_embed = new_def.clone();
    let embedded = tokio::task::spawn_blocking(move || def_for_embed.embed_anchors(&embedder))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut cls = state.classifier.write().await;
    cls.definition = new_def;
    cls.embedded_anchors = embedded;
    Ok(Json(definition_response(&cls.definition)))
}

async fn add_anchor(
    State(state): State<Arc<AppState>>,
    Path(label): Path<String>,
    Json(req): Json<AddAnchorRequest>,
) -> Result<Json<DefinitionResponse>, (StatusCode, String)> {
    let mut new_def = {
        let cls = state.classifier.read().await;
        cls.definition.clone()
    };

    let anchor_list = new_def
        .anchors
        .get_mut(&label)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("label '{label}' not found")))?;
    anchor_list.push(req.text);

    let embedder = state.embedder.clone();
    let def_for_embed = new_def.clone();
    let embedded = tokio::task::spawn_blocking(move || def_for_embed.embed_anchors(&embedder))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut cls = state.classifier.write().await;
    cls.definition = new_def;
    cls.embedded_anchors = embedded;
    Ok(Json(definition_response(&cls.definition)))
}

async fn remove_anchor(
    State(state): State<Arc<AppState>>,
    Path((label, index)): Path<(String, usize)>,
) -> Result<Json<DefinitionResponse>, (StatusCode, String)> {
    let mut new_def = {
        let cls = state.classifier.read().await;
        cls.definition.clone()
    };

    let anchor_list = new_def
        .anchors
        .get_mut(&label)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("label '{label}' not found")))?;
    if index >= anchor_list.len() {
        return Err((StatusCode::NOT_FOUND, format!("anchor index {index} out of range")));
    }
    anchor_list.remove(index);

    let embedder = state.embedder.clone();
    let def_for_embed = new_def.clone();
    let embedded = tokio::task::spawn_blocking(move || def_for_embed.embed_anchors(&embedder))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut cls = state.classifier.write().await;
    cls.definition = new_def;
    cls.embedded_anchors = embedded;
    Ok(Json(definition_response(&cls.definition)))
}

async fn classify_text(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ClassifyRequest>,
) -> Result<Json<ClassifyResponse>, (StatusCode, String)> {
    let embedder = state.embedder.clone();
    let text = req.text.clone();
    let embedding = tokio::task::spawn_blocking(move || embedder.embed(&text))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let cls = state.classifier.read().await;
    let (predicted, confidence, scores) =
        eval::classify_example(&embedding, &cls.embedded_anchors, cls.definition.top_k);

    let label_scores: Vec<LabelScore> = scores
        .iter()
        .map(|(l, s)| LabelScore {
            label: l.clone(),
            score: *s,
        })
        .collect();

    let anchor_details =
        eval::anchor_contributions(&embedding, &cls.embedded_anchors, &cls.definition.anchors)
            .into_iter()
            .map(|(label, anchor_text, similarity)| AnchorDetail {
                label,
                anchor_text,
                similarity,
            })
            .collect();

    Ok(Json(ClassifyResponse {
        text: req.text,
        predicted,
        confidence,
        label_scores,
        anchor_details,
    }))
}

async fn run_evaluate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, (StatusCode, String)> {
    let dataset_name = req.dataset.unwrap_or_else(|| "complexity-heldout".to_string());
    let dataset_path = state.datasets_dir.join(format!("{dataset_name}.jsonl"));

    let raw = std::fs::read_to_string(&dataset_path)
        .map_err(|e| (StatusCode::NOT_FOUND, format!("dataset '{dataset_name}': {e}")))?;
    let examples = eval::load_dataset(&raw);

    let embedder = state.embedder.clone();
    let anchors = {
        let cls = state.classifier.read().await;
        cls.embedded_anchors.clone()
    };
    let labels = {
        let cls = state.classifier.read().await;
        cls.definition.labels.clone()
    };
    let top_k = {
        let cls = state.classifier.read().await;
        cls.definition.top_k
    };

    let report = tokio::task::spawn_blocking(move || {
        eval::run_evaluation(&embedder, &anchors, &labels, top_k, &examples)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let predictions: Vec<PredictionJson> = report
        .predictions
        .iter()
        .map(|p| {
            let alternatives: Vec<LabelScore> = p
                .label_scores
                .iter()
                .filter(|(l, _)| *l != p.predicted)
                .map(|(l, s)| LabelScore {
                    label: l.clone(),
                    score: *s,
                })
                .collect();
            PredictionJson {
                text: p.text.clone(),
                expected: p.expected.clone(),
                predicted: p.predicted.clone(),
                confidence: p.confidence,
                alternatives,
            }
        })
        .collect();

    Ok(Json(EvaluateResponse {
        dataset: dataset_name,
        total: report.total,
        accuracy: report.accuracy,
        macro_f1: report.macro_f1,
        error_count: report.error_count,
        labels: report.labels,
        confusion: report.confusion,
        label_metrics: report
            .label_metrics
            .into_iter()
            .map(|m| LabelMetricJson {
                label: m.label,
                precision: m.precision,
                recall: m.recall,
                f1: m.f1,
                support: m.support,
            })
            .collect(),
        predictions,
    }))
}

async fn list_datasets(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DatasetsResponse>, (StatusCode, String)> {
    let mut datasets = Vec::new();
    let entries = std::fs::read_dir(&state.datasets_dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for entry in entries {
        let entry = entry.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "jsonl") {
            let name = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let raw = std::fs::read_to_string(&path)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            let count = raw.lines().filter(|l| !l.trim().is_empty()).count();
            datasets.push(DatasetInfo { name, count });
        }
    }
    datasets.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(DatasetsResponse { datasets }))
}

#[tokio::main]
async fn main() {
    let listen = env::var("LLM_D_SC_API_LISTEN").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let model_dir = env::var("LLM_D_SC_MODEL_DIR").unwrap_or_else(|_| "/models".to_string());
    let datasets_dir =
        env::var("LLM_D_SC_DATASETS_DIR").unwrap_or_else(|_| "evals/datasets".to_string());

    eprintln!("llm-d-sc-api: loading embedder from {model_dir}");
    let embedder = Arc::new(
        Embedder::load(
            format!("{model_dir}/config.json"),
            format!("{model_dir}/model.safetensors"),
            format!("{model_dir}/tokenizer.json"),
            format!("{model_dir}/1_Pooling/config.json"),
        )
        .expect("embedder must load"),
    );

    let definition =
        ClassifierDefinition::from_env().expect("classifier definition must resolve");
    eprintln!(
        "llm-d-sc-api: classifier '{}' with {} labels, {} anchors",
        definition.classifier_id,
        definition.labels.len(),
        definition.anchor_count()
    );

    let embedded_anchors = definition
        .embed_anchors(&embedder)
        .expect("anchors must embed");

    let state = Arc::new(AppState {
        embedder,
        classifier: RwLock::new(ClassifierState {
            definition,
            embedded_anchors,
        }),
        datasets_dir: PathBuf::from(&datasets_dir),
    });

    let app = Router::new()
        .route("/api/v1/definition", get(get_definition))
        .route("/api/v1/definition/anchors", put(update_anchors))
        .route("/api/v1/definition/anchors/{label}", post(add_anchor))
        .route(
            "/api/v1/definition/anchors/{label}/{index}",
            delete(remove_anchor),
        )
        .route("/api/v1/classify", post(classify_text))
        .route("/api/v1/evaluate", post(run_evaluate))
        .route("/api/v1/datasets", get(list_datasets))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&listen)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {listen}: {e}"));
    eprintln!("llm-d-sc-api: listening on {listen}");
    eprintln!("llm-d-sc-api: datasets from {datasets_dir}");
    axum::serve(listener, app).await.unwrap();
}
