import * as React from 'react';
import { EvaluationResult } from '../api/types';

interface Props {
  evaluation: EvaluationResult;
}

export const ClassifierOverview: React.FC<Props> = ({ evaluation }) => {
  const cards = [
    { label: 'Examples', value: evaluation.totalExamples },
    { label: 'Accuracy', value: `${(evaluation.accuracy * 100).toFixed(1)}%` },
    { label: 'Macro F1', value: `${(evaluation.macroF1 * 100).toFixed(1)}%` },
    { label: 'Errors', value: evaluation.errorCount, danger: evaluation.errorCount > 0 },
  ];

  return (
    <div className="semantic-classifier__summary">
      {cards.map((card) => (
        <div
          key={card.label}
          className={`semantic-classifier__card${card.danger ? ' semantic-classifier__card--danger' : ''}`}
        >
          <small>{card.label}</small>
          <strong>{card.value}</strong>
        </div>
      ))}
    </div>
  );
};
