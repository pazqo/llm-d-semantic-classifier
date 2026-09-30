import * as React from 'react';
import { useTranslation } from 'react-i18next';
import {
  Page,
  PageSection,
  Title,
  Alert,
  Spinner,
} from '@patternfly/react-core';
import { EvaluationResult, TrainingRun, ClassDefinition } from '../api/types';
import { classifierApi } from '../api';
import { ClassifierOverview } from './ClassifierOverview';
import { ConfusionMatrix } from './ConfusionMatrix';
import { Misclassifications } from './Misclassifications';
import { AnchorPrompts } from './AnchorPrompts';
import { RetrainPanel } from './RetrainPanel';
import { TestClassifier } from './TestClassifier';
import './semantic-classifier.css';

const TAB_KEYS = ['evaluation', 'test', 'training'] as const;
const TAB_LABELS = ['Evaluation', 'Test', 'Model & Training'];

interface DisabledConfig {
  disabledClasses: Set<string>;
  disabledAnchors: Record<string, Set<string>>;
}

function extractConfig(classes: ClassDefinition[]): DisabledConfig {
  const disabledClasses = new Set<string>();
  const disabledAnchors: Record<string, Set<string>> = {};
  for (const cls of classes) {
    if (cls.disabled) disabledClasses.add(cls.name);
    if (cls.disabledAnchors?.length) {
      disabledAnchors[cls.name] = new Set(cls.disabledAnchors);
    }
  }
  return { disabledClasses, disabledAnchors };
}

function applyConfig(classes: ClassDefinition[], config: DisabledConfig): ClassDefinition[] {
  return classes.map((c) => ({
    ...c,
    disabled: config.disabledClasses.has(c.name),
    disabledAnchors: config.disabledAnchors[c.name]
      ? Array.from(config.disabledAnchors[c.name]).filter((a) => c.anchors.includes(a))
      : [],
  }));
}

export default function SemanticClassifierPage() {
  const { t } = useTranslation('plugin__llm-d-sc-console-plugin');
  const [activeTab, setActiveTab] = React.useState(0);
  const [evaluation, setEvaluation] = React.useState<EvaluationResult | null>(null);
  const [trainingHistory, setTrainingHistory] = React.useState<TrainingRun[]>([]);
  const [editedClasses, setEditedClasses] = React.useState<ClassDefinition[] | null>(null);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);
  const configRef = React.useRef<DisabledConfig>({ disabledClasses: new Set(), disabledAnchors: {} });

  const loadData = React.useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const [evalResult, history] = await Promise.all([
        classifierApi.getEvaluation(),
        classifierApi.getTrainingHistory(),
      ]);
      setEvaluation(evalResult);
      setTrainingHistory(history);
      const merged = applyConfig(
        evalResult.classes.map((c) => ({ ...c, anchors: [...c.anchors] })),
        configRef.current,
      );
      setEditedClasses(merged);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  }, []);

  const handleClassesChange = React.useCallback((classes: ClassDefinition[]) => {
    configRef.current = extractConfig(classes);
    setEditedClasses(classes);
  }, []);

  React.useEffect(() => {
    loadData();
  }, [loadData]);

  React.useEffect(() => {
    document.title = t('Semantic Classifier');
  }, [t]);

  const handleRetrainComplete = React.useCallback(
    (run: TrainingRun) => {
      setTrainingHistory((prev) => [run, ...prev.filter((r) => r.id !== run.id)]);
      loadData();
    },
    [loadData],
  );

  if (loading) {
    return (
      <Page>
        <PageSection variant="light">
          <div className="sc-loading">
            <Spinner size="lg" />
            <span>{t('Loading classifier data...')}</span>
          </div>
        </PageSection>
      </Page>
    );
  }

  if (error || !evaluation) {
    return (
      <Page>
        <PageSection variant="light">
          <Alert variant="danger" title={t('Error loading classifier data')}>
            {error}
          </Alert>
        </PageSection>
      </Page>
    );
  }

  return (
    <Page>
      <PageSection variant="light">
        <div className="sc-header">
          <div className="sc-header__left">
            <Title headingLevel="h1">{t('Semantic Classifier')}</Title>
            <div className="sc-header__meta">
              <span>{t('Model')} <strong>{evaluation.modelName}</strong></span>
              <span className="sc-header__sep">/</span>
              <span>{t('Anchor set')} <strong>{evaluation.anchorSetVersion}</strong></span>
              <span className="sc-header__sep">/</span>
              <span>{t('Dataset')} <strong>{evaluation.datasetName}</strong></span>
            </div>
          </div>
          <span className="sc-badge sc-badge--green">{t('Active')}</span>
        </div>
      </PageSection>

      <PageSection variant="light" padding={{ default: 'noPadding' }}>
        <div className="sc-tabs">
          {TAB_LABELS.map((label, i) => (
            <button
              key={TAB_KEYS[i]}
              className={`sc-tabs__btn ${activeTab === i ? 'sc-tabs__btn--active' : ''}`}
              onClick={() => setActiveTab(i)}
              type="button"
            >
              {t(label)}
            </button>
          ))}
        </div>
      </PageSection>

      <PageSection variant="light">
        {activeTab === 0 && (
          <div>
            <ClassifierOverview evaluation={evaluation} />
            <ConfusionMatrix
              predictions={evaluation.predictions}
              classNames={evaluation.classes.map((c) => c.name)}
            />
            <div className="semantic-classifier__section">
              <Title headingLevel="h2">{t('Per-class metrics')}</Title>
              <table className="semantic-classifier__table">
                <thead>
                  <tr>
                    <th>{t('Class')}</th>
                    <th>{t('Precision')}</th>
                    <th>{t('Recall')}</th>
                    <th>{t('F1')}</th>
                    <th>{t('Support')}</th>
                  </tr>
                </thead>
                <tbody>
                  {evaluation.metrics.map((m) => (
                    <tr key={m.name}>
                      <td><strong>{m.name}</strong></td>
                      <td>{(m.precision * 100).toFixed(1)}%</td>
                      <td>{(m.recall * 100).toFixed(1)}%</td>
                      <td>{(m.f1 * 100).toFixed(1)}%</td>
                      <td>{m.support}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <Misclassifications predictions={evaluation.predictions} />
          </div>
        )}

        {activeTab === 1 && <TestClassifier />}

        {activeTab === 2 && (
          <div>
            <AnchorPrompts classes={editedClasses || []} onChange={handleClassesChange} />
            <RetrainPanel
              classes={editedClasses || []}
              trainingHistory={trainingHistory}
              onRetrainComplete={handleRetrainComplete}
            />
          </div>
        )}
      </PageSection>
    </Page>
  );
}
