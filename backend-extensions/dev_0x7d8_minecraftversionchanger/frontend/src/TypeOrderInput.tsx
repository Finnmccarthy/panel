import { faGripVertical, faPlus, faTrash } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { ActionIcon, Box, Divider, Group, Paper, Stack, Text } from '@mantine/core';
import { ComponentProps, useCallback, useState } from 'react';
import Badge from '@/elements/Badge.tsx';
import Button from '@/elements/Button.tsx';
import { DndContainer, DndItem, SortableItem } from '@/elements/DragAndDrop.tsx';
import MultiSelect from '@/elements/input/MultiSelect.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import Tooltip from '@/elements/Tooltip.tsx';
import { useTranslations } from '@/providers/TranslationProvider';
import { useExtTranslations } from './translations.ts';

export interface TypeOrderInputProps {
  value: Record<string, string[]>;
  onChange?: (value: Record<string, string[]>) => void;
  allowedTypes: string[];
  readOnly?: boolean;
  label?: string;
  description?: string;
}

interface DndEntry extends DndItem {
  id: string;
  key: string;
  types: string[];
}

export default function TypeOrderInput({
  value,
  onChange,
  allowedTypes,
  readOnly = false,
  label,
  description,
}: TypeOrderInputProps) {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();

  const [newKey, setNewKey] = useState('');
  const [keyError, setKeyError] = useState<string | null>(null);

  const entries = Object.entries(value);

  const emit = useCallback(
    (next: Record<string, string[]>) => {
      onChange?.(next);
    },
    [onChange],
  );

  const handleAddKey = () => {
    const trimmed = newKey.trim();
    if (!trimmed) {
      setKeyError(tExt('elements.typeOrderInput.error.empty', {}));
      return;
    }
    if (trimmed in value) {
      setKeyError(tExt('elements.typeOrderInput.error.duplicate', {}));
      return;
    }
    setKeyError(null);
    setNewKey('');
    emit({ ...value, [trimmed]: [] });
  };

  const handleRemoveKey = (key: string) => {
    const next = { ...value };
    delete next[key];
    emit(next);
  };

  const handleRenameKey = (oldKey: string, newKeyName: string) => {
    if (newKeyName === oldKey) return;
    if (newKeyName.trim() === '') return;
    if (newKeyName in value) return;

    const next: Record<string, string[]> = {};
    for (const [k, v] of Object.entries(value)) {
      if (k === oldKey) {
        next[newKeyName] = v;
      } else {
        next[k] = v;
      }
    }
    emit(next);
  };

  const handleTypesChange = (key: string, types: string[]) => {
    emit({ ...value, [key]: types });
  };

  const dndItems: DndEntry[] = entries.map(([key, types]) => ({
    id: key,
    key,
    types,
  }));

  const handleDragEnd = (items: DndEntry[]) => {
    const next: Record<string, string[]> = {};
    for (const item of items) {
      next[item.key] = item.types;
    }
    emit(next);
  };

  const renderEntryItem = (item: DndEntry, dragHandleProps?: ComponentProps<'button'>) => {
    const { key, types } = item;

    return (
      <Paper
        withBorder
        p='xs'
        radius='sm'
        style={{
          display: 'flex',
          alignItems: readOnly ? 'center' : 'flex-start',
          gap: 8,
        }}
      >
        {!readOnly && (
          <ActionIcon
            size='sm'
            variant='subtle'
            color='gray'
            mt={4}
            style={{ flexShrink: 0, cursor: 'grab' }}
            {...dragHandleProps}
          >
            <FontAwesomeIcon icon={faGripVertical} style={{ fontSize: 14 }} />
          </ActionIcon>
        )}

        <Box style={{ flex: '0 0 200px' }}>
          {readOnly ? (
            <Text size='sm' fw={500}>
              {key}
            </Text>
          ) : (
            <TextInput
              size='sm'
              defaultValue={key}
              onBlur={(e) => handleRenameKey(key, e.currentTarget.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  handleRenameKey(key, e.currentTarget.value);
                  e.currentTarget.blur();
                }
              }}
              styles={{
                input: { fontWeight: 500 },
              }}
            />
          )}
        </Box>

        <Box style={{ flex: 1, minWidth: 0 }}>
          {readOnly ? (
            types.length > 0 ? (
              <Group gap={4} wrap='wrap'>
                {types.map((type) => (
                  <Badge key={type} variant='light' size='sm'>
                    {type}
                  </Badge>
                ))}
              </Group>
            ) : (
              <Text size='sm' c='dimmed'>
                {tExt('elements.typeOrderInput.noTypes', {})}
              </Text>
            )
          ) : (
            <MultiSelect
              size='sm'
              data={allowedTypes}
              value={types}
              onChange={(t) => handleTypesChange(key, t)}
              placeholder={tExt('elements.typeOrderInput.selectTypes', {})}
              searchable
              clearable
            />
          )}
        </Box>

        {!readOnly && (
          <Tooltip label={tExt('elements.typeOrderInput.tooltip.removeEntry', {})} withArrow>
            <ActionIcon
              variant='subtle'
              color='red'
              size='lg'
              mt={2}
              onClick={() => handleRemoveKey(key)}
              style={{ flexShrink: 0 }}
            >
              <FontAwesomeIcon icon={faTrash} style={{ fontSize: 14 }} />
            </ActionIcon>
          </Tooltip>
        )}
      </Paper>
    );
  };

  return (
    <Box>
      {label && (
        <Text fw={500} size='sm' mb={2}>
          {label}
        </Text>
      )}
      {description && (
        <Text size='xs' c='dimmed' mb='xs'>
          {description}
        </Text>
      )}

      <Stack gap='xs'>
        {entries.length > 0 ? (
          <DndContainer
            items={dndItems}
            callbacks={{
              onDragEnd: handleDragEnd,
            }}
            renderOverlay={(activeItem) =>
              activeItem ? (
                <div style={{ cursor: 'grabbing' }}>
                  {renderEntryItem(activeItem, { style: { cursor: 'grabbing' } })}
                </div>
              ) : null
            }
          >
            {(items) => (
              <Stack gap='xs'>
                {items.map((item) => (
                  <SortableItem
                    key={item.id}
                    id={item.id}
                    renderItem={({ dragHandleProps }) =>
                      renderEntryItem(item, dragHandleProps as unknown as ComponentProps<'button'>)
                    }
                  />
                ))}
              </Stack>
            )}
          </DndContainer>
        ) : (
          <Text size='sm' c='dimmed' ta='center' py='md'>
            {tExt('elements.typeOrderInput.noEntries', {})}
          </Text>
        )}

        {!readOnly && (
          <>
            <Divider />
            <Group gap='xs'>
              <TextInput
                placeholder={tExt('elements.typeOrderInput.newKey', {})}
                size='sm'
                value={newKey}
                onChange={(e) => {
                  setNewKey(e.currentTarget.value);
                  if (keyError) setKeyError(null);
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleAddKey();
                }}
                error={keyError}
                style={{ flex: 1 }}
              />
              <Button
                size='sm'
                variant='light'
                leftSection={<FontAwesomeIcon icon={faPlus} style={{ fontSize: 12 }} />}
                onClick={handleAddKey}
              >
                {t('common.button.add', {})}
              </Button>
            </Group>
          </>
        )}
      </Stack>
    </Box>
  );
}
