import { faGripVertical, faPlus, faTrash } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Box, Divider, Group, Paper, Stack, Text } from '@mantine/core';
import { UseFormReturnType } from '@mantine/form';
import { ComponentProps } from 'react';
import { z } from 'zod';
import getAllEggs from '@/api/admin/nests/getAllEggs.ts';
import ActionIcon from '@/elements/ActionIcon.tsx';
import Button from '@/elements/Button.tsx';
import { DndContainer, DndItem, SortableItem } from '@/elements/DragAndDrop.tsx';
import LocalizedTextInput from '@/elements/input/LocalizedTextInput.tsx';
import MultiSelectGroup from '@/elements/input/MultiSelectGroup.tsx';
import Switch from '@/elements/input/Switch.tsx';
import Tooltip from '@/elements/Tooltip.tsx';
import { useResource } from '@/plugins/useResource.ts';
import { useGlobalStore } from '@/stores/global.ts';
import { adminExtensionSettingsSchema } from './lib/schemas.ts';
import { useExtTranslations } from './translations.ts';

interface DndGroup extends DndItem {
  id: string;
  index: number;
}

interface DndEgg extends DndItem {
  id: string;
  uuid: string;
}

interface EggGroupEditorProps {
  index: number;
  form: UseFormReturnType<z.infer<typeof adminExtensionSettingsSchema>>;
  eggData: { group: string; items: { label: string; value: string }[] }[];
  onRemove: () => void;
  dragHandleProps?: ComponentProps<'button'>;
}

function EggGroupEditor({ index, form, eggData, onRemove, dragHandleProps }: EggGroupEditorProps) {
  const { t: tExt } = useExtTranslations();
  const { languages } = useGlobalStore();

  const group = form.getValues().eggGroups[index];
  if (!group) return null;

  const eggItems: DndEgg[] = group.eggs.map((uuid) => ({ id: uuid, uuid }));
  const flatEggData = eggData.flatMap((g) => g.items);

  const getEggLabel = (uuid: string) => {
    const egg = flatEggData.find((e) => e.value === uuid);
    return egg?.label ?? uuid;
  };

  const handleEggsDragEnd = (items: DndEgg[]) => {
    form.setFieldValue(
      `eggGroups.${index}.eggs`,
      items.map((i) => i.uuid),
    );
  };

  const handleRemoveEgg = (uuid: string) => {
    const current = form.getValues().eggGroups[index].eggs;
    form.setFieldValue(
      `eggGroups.${index}.eggs`,
      current.filter((e) => e !== uuid),
    );
  };

  const renderEggItem = (item: DndEgg, eggDragHandleProps?: ComponentProps<'button'>) => (
    <Paper withBorder p={6} radius='sm'>
      <Group gap='xs' wrap='nowrap'>
        <ActionIcon
          size='sm'
          variant='subtle'
          color='gray'
          style={{ cursor: 'grab', flexShrink: 0 }}
          {...eggDragHandleProps}
        >
          <FontAwesomeIcon icon={faGripVertical} style={{ fontSize: 12 }} />
        </ActionIcon>

        <Text size='sm' style={{ flex: 1 }}>
          {getEggLabel(item.uuid)}
        </Text>

        <Tooltip label={tExt('elements.extensionSettingsEditor.tooltip.removeEgg', {})}>
          <ActionIcon size='sm' variant='subtle' color='red' onClick={() => handleRemoveEgg(item.uuid)}>
            <FontAwesomeIcon icon={faTrash} style={{ fontSize: 12 }} />
          </ActionIcon>
        </Tooltip>
      </Group>
    </Paper>
  );

  return (
    <Paper withBorder p='sm' radius='sm'>
      <Stack gap='sm'>
        <Group gap='xs' align='flex-start' wrap='nowrap'>
          <ActionIcon
            size='sm'
            variant='subtle'
            color='gray'
            mt={4}
            style={{ cursor: 'grab', flexShrink: 0 }}
            {...dragHandleProps}
          >
            <FontAwesomeIcon icon={faGripVertical} style={{ fontSize: 14 }} />
          </ActionIcon>

          <Text size='xs' fw={500} c='dimmed' my='auto'>
            #{index + 1}
          </Text>

          <span className='flex-1 my-auto'>
            <LocalizedTextInput
              placeholder={tExt('elements.extensionSettingsEditor.form.groupNamePlaceholder', {})}
              languages={languages}
              value={form.getValues().eggGroups[index].name}
              setValue={(v) => form.setFieldValue(`eggGroups.${index}.name`, v || '')}
              valueTranslations={form.getValues().eggGroups[index].nameTranslations}
              setValueTranslations={(t) => form.setFieldValue(`eggGroups.${index}.nameTranslations`, t)}
            />
          </span>

          <Tooltip label={tExt('elements.extensionSettingsEditor.tooltip.removeGroup', {})}>
            <ActionIcon variant='subtle' color='red' size='input-md' my='auto' onClick={onRemove}>
              <FontAwesomeIcon icon={faTrash} style={{ fontSize: 14 }} />
            </ActionIcon>
          </Tooltip>
        </Group>

        <Divider label={tExt('elements.extensionSettingsEditor.divider.eggs', {})} labelPosition='left' />

        <MultiSelectGroup
          label={tExt('elements.extensionSettingsEditor.form.selectableEggs', {})}
          placeholder={tExt('elements.extensionSettingsEditor.form.selectableEggsPlaceholder', {})}
          data={eggData}
          value={group.eggs}
          onChange={(selected) => form.setFieldValue(`eggGroups.${index}.eggs`, selected)}
          searchable
          clearable
        />

        {eggItems.length > 0 && (
          <DndContainer
            items={eggItems}
            callbacks={{ onDragEnd: handleEggsDragEnd }}
            renderOverlay={(activeItem) =>
              activeItem ? (
                <div style={{ cursor: 'grabbing' }}>{renderEggItem(activeItem, { style: { cursor: 'grabbing' } })}</div>
              ) : null
            }
          >
            {(items) => (
              <Stack gap={4}>
                {items.map((item) => (
                  <SortableItem
                    key={item.id}
                    id={item.id}
                    renderItem={({ dragHandleProps: ehp }) =>
                      renderEggItem(item, ehp as unknown as ComponentProps<'button'>)
                    }
                  />
                ))}
              </Stack>
            )}
          </DndContainer>
        )}

        <Divider label={tExt('elements.extensionSettingsEditor.divider.options', {})} labelPosition='left' />

        <Group grow>
          <Switch
            label={tExt('elements.extensionSettingsEditor.form.forceUpdateStartup', {})}
            description={tExt('elements.extensionSettingsEditor.form.forceUpdateStartupDescription', {})}
            {...form.getInputProps(`eggGroups.${index}.forceUpdateStartup`, { type: 'checkbox' })}
          />
          <Switch
            label={tExt('elements.extensionSettingsEditor.form.forceReinstall', {})}
            description={tExt('elements.extensionSettingsEditor.form.forceReinstallDescription', {})}
            {...form.getInputProps(`eggGroups.${index}.forceReinstall`, { type: 'checkbox' })}
          />
        </Group>

        <Switch
          label={tExt('elements.extensionSettingsEditor.form.forceReinstallTruncateFiles', {})}
          description={tExt('elements.extensionSettingsEditor.form.forceReinstallTruncateFilesDescription', {})}
          {...form.getInputProps(`eggGroups.${index}.forceReinstallTruncateFiles`, { type: 'checkbox' })}
        />

        <Switch
          label={tExt('elements.extensionSettingsEditor.form.reassignAllocations', {})}
          description={tExt('elements.extensionSettingsEditor.form.reassignAllocationsDescription', {})}
          {...form.getInputProps(`eggGroups.${index}.reassignAllocations`, { type: 'checkbox' })}
        />

        <Divider label={tExt('elements.extensionSettingsEditor.divider.affectedEggs', {})} labelPosition='left' />

        <MultiSelectGroup
          label={tExt('elements.extensionSettingsEditor.form.affectedEggs', {})}
          description={tExt('elements.extensionSettingsEditor.form.affectedEggsDescription', {})}
          placeholder={tExt('elements.extensionSettingsEditor.form.affectedEggsPlaceholder', {})}
          data={eggData}
          value={group.affectedEggs}
          onChange={(selected) => form.setFieldValue(`eggGroups.${index}.affectedEggs`, selected)}
          searchable
          clearable
        />
      </Stack>
    </Paper>
  );
}

export interface ExtensionSettingsEditorProps {
  form: UseFormReturnType<z.infer<typeof adminExtensionSettingsSchema>>;
}

export default function ExtensionSettingsEditor({ form }: ExtensionSettingsEditorProps) {
  const { t: tExt } = useExtTranslations();

  const { data: eggData = [] } = useResource<{ group: string; items: { label: string; value: string }[] }[]>({
    queryKey: ['extension', 'eggchanger', 'allEggs'],
    queryFn: async () => {
      const eggs = await getAllEggs();
      return eggs.map((v) => ({
        group: v.nest.name,
        items: v.eggs.map((e) => ({ label: e.name, value: e.uuid })),
      }));
    },
  });

  const eggGroups = form.values.eggGroups;

  const dndGroups: DndGroup[] = eggGroups.map((group, index) => ({
    id: JSON.stringify(group),
    index,
  }));

  const handleGroupsDragEnd = (items: DndGroup[]) => {
    const current = form.getValues().eggGroups;
    form.setFieldValue(
      'eggGroups',
      items.map((i) => current[i.index]),
    );
  };

  const handleAddGroup = () => {
    form.insertListItem('eggGroups', {
      name: '',
      nameTranslations: {},
      eggs: [],
      forceUpdateStartup: false,
      forceReinstall: false,
      forceReinstallTruncateFiles: false,
      reassignAllocations: false,
      affectedEggs: [],
    });
  };

  const handleRemoveGroup = (index: number) => {
    form.removeListItem('eggGroups', index);
  };

  const renderGroupItem = (item: DndGroup, dragHandleProps?: ComponentProps<'button'>) => (
    <EggGroupEditor
      index={item.index}
      form={form}
      eggData={eggData}
      onRemove={() => handleRemoveGroup(item.index)}
      dragHandleProps={dragHandleProps}
    />
  );

  return (
    <Box>
      <Stack gap='sm'>
        {dndGroups.length > 0 ? (
          <DndContainer
            items={dndGroups}
            callbacks={{ onDragEnd: handleGroupsDragEnd }}
            renderOverlay={(activeItem) =>
              activeItem ? (
                <div style={{ cursor: 'grabbing', opacity: 0.9 }}>
                  {renderGroupItem(activeItem, { style: { cursor: 'grabbing' } })}
                </div>
              ) : null
            }
          >
            {(items) => (
              <Stack gap='sm'>
                {items.map((item, index) => (
                  <SortableItem
                    key={`egg-group-${index}`}
                    id={item.id}
                    renderItem={({ dragHandleProps }) =>
                      renderGroupItem(item, dragHandleProps as unknown as ComponentProps<'button'>)
                    }
                  />
                ))}
              </Stack>
            )}
          </DndContainer>
        ) : (
          <Text size='sm' c='dimmed' ta='center' py='md'>
            {tExt('elements.extensionSettingsEditor.noGroups', {})}
          </Text>
        )}

        <Divider />

        <Button
          variant='light'
          leftSection={<FontAwesomeIcon icon={faPlus} style={{ fontSize: 12 }} />}
          onClick={handleAddGroup}
        >
          {tExt('elements.extensionSettingsEditor.button.addGroup', {})}
        </Button>
      </Stack>
    </Box>
  );
}
