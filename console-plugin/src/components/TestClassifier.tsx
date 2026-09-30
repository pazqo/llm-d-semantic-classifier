import * as React from 'react';
import { useTranslation } from 'react-i18next';
import {
  Title,
  Button,
  TextArea,
  Spinner,
  Alert,
  Card,
  CardBody,
  DescriptionList,
  DescriptionListGroup,
  DescriptionListTerm,
  DescriptionListDescription,
  Progress,
  ProgressVariant,
} from '@patternfly/react-core';
import { SearchIcon } from '@patternfly/react-icons';
import { ClassifyResult } from '../api/types';
import { classifierApi } from '../api';

export const TestClassifier: React.FC = () => {
  const { t } = useTranslation('plugin__llm-d-sc-console-plugin');
  const [text, setText] = React.useState('');
  const [result, setResult] = React.useState<ClassifyResult | null>(null);
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);

  const handleClassify = async () => {
    const input = text.trim();
    if (!input) return;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const res = await classifierApi.classify(input);
      setResult(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Classification failed');
    } finally {
      setLoading(false);
    }
  };

  const maxScore = result
    ? Math.max(...result.labelScores.map((s) => s.score))
    : 0;

  const fmt = (v: number) => `${(v * 100).toFixed(1)}%`;

  return (
    <div className="semantic-classifier__section" style={{ marginTop: 0 }}>
      <Title headingLevel="h2">{t('Test classifier')}</Title>
      <p className="semantic-classifier__muted">
        {t('Enter a prompt to classify against the current anchors.')}
      </p>

      <div className="semantic-classifier__test-input-area">
        <TextArea
          aria-label={t('Text to classify')}
          placeholder={t('Enter a prompt to classify, e.g. "Can you help me understand the difference between transformers and RNNs?"')}
          value={text}
          onChange={(_event, value) => setText(value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              handleClassify();
            }
          }}
          rows={4}
          resizeOrientation="vertical"
        />
        <Button
          variant="primary"
          onClick={handleClassify}
          isDisabled={loading || !text.trim()}
        >
          <SearchIcon /> {loading ? t('Classifying...') : t('Classify')}
        </Button>
      </div>

      {loading && (
        <div className="semantic-classifier__loading">
          <Spinner size="md" />
          <span>{t('Running classification...')}</span>
        </div>
      )}

      {error && (
        <Alert variant="danger" title={t('Classification failed')} className="semantic-classifier__alert">
          {error}
        </Alert>
      )}

      {result && (
        <div className="semantic-classifier__test-result">
          <Card isCompact>
            <CardBody>
              <DescriptionList isHorizontal isCompact>
                <DescriptionListGroup>
                  <DescriptionListTerm>{t('Predicted class')}</DescriptionListTerm>
                  <DescriptionListDescription>
                    <strong style={{ fontSize: '18px' }}>{result.predicted}</strong>
                  </DescriptionListDescription>
                </DescriptionListGroup>
                <DescriptionListGroup>
                  <DescriptionListTerm>{t('Confidence')}</DescriptionListTerm>
                  <DescriptionListDescription>
                    <strong style={{ fontSize: '18px' }}>{fmt(result.confidence)}</strong>
                  </DescriptionListDescription>
                </DescriptionListGroup>
              </DescriptionList>
            </CardBody>
          </Card>

          <div className="semantic-classifier__section">
            <Title headingLevel="h3">{t('Label scores')}</Title>
            <div className="semantic-classifier__score-bars">
              {result.labelScores.map((s) => (
                <Progress
                  key={s.label}
                  title={s.label}
                  value={maxScore > 0 ? (s.score / maxScore) * 100 : 0}
                  label={fmt(s.score)}
                  variant={
                    s.label === result.predicted
                      ? ProgressVariant.success
                      : undefined
                  }
                  style={{ marginBottom: '8px' }}
                />
              ))}
            </div>
          </div>

          {result.anchorDetails.length > 0 && (
            <div className="semantic-classifier__section">
              <Title headingLevel="h3">{t('Anchor contributions')}</Title>
              <table className="semantic-classifier__table">
                <thead>
                  <tr>
                    <th>{t('Label')}</th>
                    <th>{t('Anchor')}</th>
                    <th>{t('Similarity')}</th>
                  </tr>
                </thead>
                <tbody>
                  {result.anchorDetails.map((a, i) => (
                    <tr key={i}>
                      <td><strong>{a.label}</strong></td>
                      <td>{a.anchorText}</td>
                      <td
                        className={
                          a.similarity > 0.5
                            ? 'semantic-classifier__similarity-high'
                            : a.similarity > 0.3
                            ? 'semantic-classifier__similarity-mid'
                            : 'semantic-classifier__similarity-low'
                        }
                      >
                        {a.similarity.toFixed(4)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
