import * as React from 'react';
import { useTranslation } from 'react-i18next';
import { Title, Button, TextInput, Switch } from '@patternfly/react-core';
import { PlusCircleIcon, PlusIcon } from '@patternfly/react-icons';
import { ClassDefinition } from '../api/types';

interface Props {
  classes: ClassDefinition[];
  onChange: (classes: ClassDefinition[]) => void;
}

export const AnchorPrompts: React.FC<Props> = ({ classes, onChange }) => {
  const { t } = useTranslation('plugin__llm-d-sc-console-plugin');
  const [newClassName, setNewClassName] = React.useState('');
  const [newAnchorInputs, setNewAnchorInputs] = React.useState<Record<string, string>>({});

  const isAnchorDisabled = (cls: ClassDefinition, anchor: string): boolean => {
    return (cls.disabledAnchors || []).includes(anchor);
  };

  const handleToggleAnchor = (classIdx: number, anchor: string) => {
    const updated = classes.map((c, i) => {
      if (i !== classIdx) return c;
      const disabled = c.disabledAnchors || [];
      const isDisabled = disabled.includes(anchor);
      return {
        ...c,
        disabledAnchors: isDisabled
          ? disabled.filter((a) => a !== anchor)
          : [...disabled, anchor],
      };
    });
    onChange(updated);
  };

  const handleAddAnchor = (classIdx: number) => {
    const className = classes[classIdx].name;
    const value = (newAnchorInputs[className] || '').trim();
    if (!value) return;
    const updated = classes.map((c, i) =>
      i === classIdx ? { ...c, anchors: [...c.anchors, value] } : c,
    );
    onChange(updated);
    setNewAnchorInputs((prev) => ({ ...prev, [className]: '' }));
  };

  const handleAddClass = () => {
    const name = newClassName.trim().toUpperCase().replace(/\s+/g, '_');
    if (!name || classes.some((c) => c.name === name)) return;
    onChange([...classes, { name, anchors: [], disabledAnchors: [] }]);
    setNewClassName('');
  };

  const handleToggleClass = (classIdx: number) => {
    const updated = classes.map((c, i) =>
      i === classIdx ? { ...c, disabled: !c.disabled } : c,
    );
    onChange(updated);
  };

  const activeCount = (cls: ClassDefinition) => {
    if (cls.disabled) return 0;
    const disabled = cls.disabledAnchors || [];
    return cls.anchors.filter((a) => !disabled.includes(a)).length;
  };

  return (
    <div className="semantic-classifier__section">
      <Title headingLevel="h2">{t('Class configuration')}</Title>
      <p className="semantic-classifier__muted">
        {t('Toggle anchors on/off to experiment. Disabled anchors are excluded from retraining.')}
      </p>
      <div className="semantic-classifier__classes">
        {classes.map((cls, classIdx) => (
          <div
            key={cls.name}
            className={`semantic-classifier__class-card ${cls.disabled ? 'semantic-classifier__class-card--disabled' : ''}`}
          >
            <div className="semantic-classifier__class-header">
              <Switch
                aria-label={t('Toggle class {{name}}', { name: cls.name })}
                isChecked={!cls.disabled}
                onChange={() => handleToggleClass(classIdx)}
              />
              <strong>{cls.name}</strong>
              <span className="semantic-classifier__class-count">
                {cls.disabled ? t('excluded') : `${activeCount(cls)}/${cls.anchors.length} active`}
              </span>
            </div>
            {!cls.disabled && (
              <>
                <ul className="semantic-classifier__anchor-list">
                  {cls.anchors.map((anchor, anchorIdx) => {
                    const disabled = isAnchorDisabled(cls, anchor);
                    return (
                      <li
                        key={anchorIdx}
                        style={{ opacity: disabled ? 0.45 : 1 }}
                      >
                        <span style={disabled ? { textDecoration: 'line-through' } : undefined}>
                          {anchor}
                        </span>
                        <Switch
                          aria-label={t('Toggle anchor')}
                          isChecked={!disabled}
                          onChange={() => handleToggleAnchor(classIdx, anchor)}
                          isReversed
                        />
                      </li>
                    );
                  })}
                </ul>
                <div className="semantic-classifier__add-anchor">
                  <TextInput
                    type="text"
                    aria-label={t('New anchor prompt')}
                    placeholder={t('Add anchor prompt...')}
                    value={newAnchorInputs[cls.name] || ''}
                    onChange={(_event, value) =>
                      setNewAnchorInputs((prev) => ({ ...prev, [cls.name]: value }))
                    }
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleAddAnchor(classIdx);
                    }}
                  />
                  <Button variant="link" onClick={() => handleAddAnchor(classIdx)}>
                    <PlusCircleIcon /> {t('Add')}
                  </Button>
                </div>
              </>
            )}
          </div>
        ))}
      </div>
      <div className="semantic-classifier__add-class">
        <TextInput
          type="text"
          aria-label={t('New class name')}
          placeholder={t('New class name...')}
          value={newClassName}
          onChange={(_event, value) => setNewClassName(value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') handleAddClass();
          }}
        />
        <Button variant="secondary" onClick={handleAddClass}>
          <PlusIcon /> {t('Add class')}
        </Button>
      </div>
    </div>
  );
};
