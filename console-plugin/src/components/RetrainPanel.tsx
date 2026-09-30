import * as React from 'react';
import { useTranslation } from 'react-i18next';
import { Title, Button, Alert, Spinner } from '@patternfly/react-core';
import { SyncAltIcon } from '@patternfly/react-icons';
import { ClassDefinition, TrainingRun } from '../api/types';
import { classifierApi } from '../api';

interface Props {
  classes: ClassDefinition[];
  trainingHistory: TrainingRun[];
  onRetrainComplete: (run: TrainingRun) => void;
}

export const RetrainPanel: React.FC<Props> = ({ classes, trainingHistory, onRetrainComplete }) => {
  const { t } = useTranslation('plugin__llm-d-sc-console-plugin');
  const [activeRun, setActiveRun] = React.useState<TrainingRun | null>(null);
  const pollRef = React.useRef<number | null>(null);

  const handleRetrain = async () => {
    const activeClasses = classes
      .filter((c) => !c.disabled)
      .map((c) => ({
        ...c,
        anchors: c.anchors.filter((a) => !(c.disabledAnchors || []).includes(a)),
      }));
    const run = await classifierApi.startRetraining(activeClasses);
    setActiveRun(run);

    pollRef.current = window.setInterval(async () => {
      const status = await classifierApi.getTrainingStatus(run.id);
      if (status.status === 'completed' || status.status === 'failed') {
        if (pollRef.current) window.clearInterval(pollRef.current);
        pollRef.current = null;
        setActiveRun(null);
        onRetrainComplete(status);
      }
    }, 500);
  };

  React.useEffect(() => {
    return () => {
      if (pollRef.current) window.clearInterval(pollRef.current);
    };
  }, []);

  const formatDate = (iso: string) => new Date(iso).toLocaleString();

  return (
    <div className="semantic-classifier__section">
      <Title headingLevel="h2">{t('Retraining')}</Title>
      <p className="semantic-classifier__muted">
        {t('Start a new training run with the current class configuration.')}
      </p>

      {activeRun && (
        <Alert variant="info" title={t('Training in progress')} className="semantic-classifier__alert">
          <Spinner size="md" /> {t('Run {{id}} is training...', { id: activeRun.id })}
        </Alert>
      )}

      <Button variant="primary" onClick={handleRetrain} isDisabled={!!activeRun}>
        <SyncAltIcon /> {activeRun ? t('Training...') : t('Start retraining')}
      </Button>

      {trainingHistory.length > 0 && (
        <div className="semantic-classifier__section">
          <Title headingLevel="h3">{t('Training history')}</Title>
          <table className="semantic-classifier__table">
            <thead>
              <tr>
                <th>{t('Run')}</th>
                <th>{t('Started')}</th>
                <th>{t('Status')}</th>
                <th>{t('Anchor set')}</th>
                <th>{t('Accuracy')}</th>
                <th>{t('Macro F1')}</th>
              </tr>
            </thead>
            <tbody>
              {trainingHistory.map((run) => (
                <tr key={run.id}>
                  <td>{run.id}</td>
                  <td>{formatDate(run.startedAt)}</td>
                  <td>
                    <span
                      className={`semantic-classifier__status semantic-classifier__status--${run.status}`}
                    >
                      {run.status}
                    </span>
                  </td>
                  <td>{run.anchorSetVersion}</td>
                  <td>{run.metrics ? `${(run.metrics.accuracy * 100).toFixed(1)}%` : '—'}</td>
                  <td>{run.metrics ? `${(run.metrics.macroF1 * 100).toFixed(1)}%` : '—'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
};
