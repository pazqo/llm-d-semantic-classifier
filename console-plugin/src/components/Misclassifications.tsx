import * as React from 'react';
import { useTranslation } from 'react-i18next';
import { Title } from '@patternfly/react-core';
import { Prediction } from '../api/types';

interface Props {
  predictions: Prediction[];
}

export const Misclassifications: React.FC<Props> = ({ predictions }) => {
  const { t } = useTranslation('plugin__llm-d-sc-console-plugin');
  const [filter, setFilter] = React.useState('all');

  const errors = predictions
    .filter((p) => p.expected !== p.predicted)
    .filter((p) => {
      if (filter === 'high') return p.confidence >= 0.75;
      if (filter === 'low') return p.confidence < 0.75;
      return true;
    });

  return (
    <div className="semantic-classifier__section">
      <div className="semantic-classifier__section-header">
        <div>
          <Title headingLevel="h2">{t('Misclassifications')}</Title>
          <p className="semantic-classifier__muted">
            {t('These examples are candidates for anchor changes or label review.')}
          </p>
        </div>
        <label className="semantic-classifier__filter">
          {t('Show')}{' '}
          <select value={filter} onChange={(e) => setFilter(e.target.value)}>
            <option value="all">{t('all errors')}</option>
            <option value="high">{t('high confidence only')}</option>
            <option value="low">{t('low confidence only')}</option>
          </select>
        </label>
      </div>
      {errors.length === 0 ? (
        <p className="semantic-classifier__empty">{t('No errors match this filter.')}</p>
      ) : (
        <table className="semantic-classifier__table">
          <thead>
            <tr>
              <th>{t('Text')}</th>
              <th>{t('Expected')}</th>
              <th>{t('Predicted')}</th>
              <th>{t('Confidence')}</th>
              <th>{t('Alternatives')}</th>
            </tr>
          </thead>
          <tbody>
            {errors.map((e) => (
              <tr key={e.id}>
                <td>{e.text}</td>
                <td>{e.expected}</td>
                <td>{e.predicted}</td>
                <td
                  className={
                    e.confidence >= 0.75
                      ? 'semantic-classifier__conf-high'
                      : 'semantic-classifier__conf-low'
                  }
                >
                  {(e.confidence * 100).toFixed(1)}%
                </td>
                <td>
                  {e.alternatives
                    ?.map((a) => `${a.className} (${(a.score * 100).toFixed(1)}%)`)
                    .join(', ') || '—'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
};
