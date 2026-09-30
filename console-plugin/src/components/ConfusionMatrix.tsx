import * as React from 'react';
import { useTranslation } from 'react-i18next';
import { Title } from '@patternfly/react-core';
import { Prediction } from '../api/types';

interface Props {
  predictions: Prediction[];
  classNames: string[];
}

export const ConfusionMatrix: React.FC<Props> = ({ predictions, classNames }) => {
  const { t } = useTranslation('plugin__llm-d-sc-console-plugin');

  const matrix = classNames.map((expected) =>
    classNames.map(
      (predicted) => predictions.filter((p) => p.expected === expected && p.predicted === predicted).length,
    ),
  );

  return (
    <div className="semantic-classifier__section">
      <Title headingLevel="h2">{t('Confusion matrix')}</Title>
      <p className="semantic-classifier__muted">
        {t('Rows are expected classes; columns are predicted classes.')}
      </p>
      <div className="semantic-classifier__matrix-wrapper">
        <table className="semantic-classifier__matrix">
          <thead>
            <tr>
              <th></th>
              {classNames.map((name) => (
                <th key={name}>{name}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {matrix.map((row, rowIdx) => (
              <tr key={classNames[rowIdx]}>
                <th>{classNames[rowIdx]}</th>
                {row.map((value, colIdx) => (
                  <td
                    key={classNames[colIdx]}
                    className={
                      rowIdx === colIdx
                        ? 'semantic-classifier__matrix-diagonal'
                        : value > 0
                        ? 'semantic-classifier__matrix-offdiag'
                        : ''
                    }
                  >
                    {value}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
