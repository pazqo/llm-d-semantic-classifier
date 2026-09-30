# Semantic Classifier — Architecture & Integration Plan

## Current State

### What exists

- **semantic-classifier-console-plugin/** — OpenShift console plugin, deployed and running on the `spascolu-cpu` ROSA cluster in namespace `semantica-demo`. Shows classifier evaluation metrics, confusion matrix, misclassifications, editable anchor prompts, and a retrain flow.
- **semantica-prototype/** — Static HTML/JS prototype used for design exploration. Superseded by the console plugin.

### What is real vs mocked

**Real:**
- OpenShift console plugin infrastructure (Deployment, Service, ConsolePlugin CR, nginx + TLS)
- UI components (PatternFly, tabs, tables, confusion matrix, editable anchors)
- Plugin registration and sidebar navigation in Developer perspective

**Mocked (all data):**
- 4 classes with hardcoded anchor prompts (`src/api/mock-data.ts`)
- 12 example predictions with hardcoded confidence scores
- All metrics (accuracy, F1, precision, recall) — computed from fake predictions
- Retraining flow — 3-second simulated delay with random metrics
- Training history — two hardcoded past runs

The mock is behind a `ClassifierApi` interface (`src/api/classifier-api.ts`). Swapping in real HTTP calls requires no UI changes.

---

## Target Architecture

### Components

```
OperatorHub catalog
  └── Semantic Classifier Operator (user clicks Install)
        ├── Deploys: classifier backend (the actual ML service)
        ├── Deploys: console plugin (the UI)
        ├── Creates: ConsolePlugin CR (auto-registers with console)
        └── Watches: SemanticClassifier CRs (user creates instances)
```

### Repos (recommended: separate repos)

```
semantic-classifier              # The classifier backend API service
semantic-classifier-console-plugin  # The console plugin (this repo, to be moved into SC repo later)
semantic-classifier-operator     # Operator that deploys both
```

Alternatively, all three can live in a monorepo. The operator doesn't contain backend or plugin code — it references their container images and orchestrates deployment.

### Custom Resource Definition

Users would create classifier instances via a CR:

```yaml
apiVersion: semantica.example.com/v1alpha1
kind: SemanticClassifier
metadata:
  name: my-classifier
  namespace: my-project
spec:
  model: "sentence-transformers/all-MiniLM-L6-v2"
  classes:
    - name: billing
      anchors: ["payment failed", "invoice or charge"]
    - name: shipping
      anchors: ["where is my order", "delivery status"]
```

### Backend API Contract

The console plugin expects a backend implementing the `ClassifierApi` interface:

```typescript
interface ClassifierApi {
  // Returns evaluation results: predictions, metrics, classes, model info
  getEvaluation(): Promise<EvaluationResult>;

  // Returns list of past training runs, most recent first
  getTrainingHistory(): Promise<TrainingRun[]>;

  // Triggers retraining with updated class/anchor configuration
  startRetraining(classes: ClassDefinition[]): Promise<TrainingRun>;

  // Polls status of an in-progress training run
  getTrainingStatus(runId: string): Promise<TrainingRun>;
}
```

Full type definitions are in `src/api/types.ts`.

---

## How OpenShift Operators Work

1. **User installs from OperatorHub** — browse catalog in OpenShift console, click "Install"
2. **Operator deploys automatically** — backend service, console plugin, ConsolePlugin CR
3. **User creates a Custom Resource** — configures a specific classifier instance
4. **Operator reconciles** — watches CRs, deploys/updates the classifier workload

Real-world examples:
- **OpenShift Pipelines**: Tekton + console plugin for pipeline views
- **OpenShift GitOps**: ArgoCD + console plugin for GitOps
- **OpenShift AI**: ML platform + dashboard plugin

### Operator tooling

- Built with [Operator SDK](https://sdk.operatorframework.io/)
- Can be Go-based (more control) or Helm-based (simpler, wraps a Helm chart)
- Packaged as an operator bundle for OperatorHub

---

## Next Steps

1. **Build the classifier backend API** — real classification, evaluation, and retraining endpoints
2. **Wire the console plugin to the real API** — replace `mockClassifierApi` with HTTP calls
3. **Build the operator** — automates deployment of backend + plugin
4. **Package for OperatorHub** — operator bundle for distribution

---

## Deployment Details (current)

- **Cluster**: `spascolu-cpu` (ROSA) — `https://api.spascolu-cpu.vgty.p3.openshiftapps.com:443`
- **Console**: `https://console-openshift-console.apps.rosa.spascolu-cpu.vgty.p3.openshiftapps.com`
- **Plugin URL**: `https://console-openshift-console.apps.rosa.spascolu-cpu.vgty.p3.openshiftapps.com/semantic-classifier`
- **Namespace**: `semantica-demo`
- **Image**: `image-registry.openshift-image-registry.svc:5000/semantica-demo/semantic-classifier-plugin:latest`
- **Build**: `oc start-build semantic-classifier-plugin --from-dir=. --follow`
