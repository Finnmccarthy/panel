import { faPlus, faTrash } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Divider, Group, Paper, Stack, Text } from '@mantine/core';
import { useEffect, useMemo, useState } from 'react';
import getAllEggs from '@/api/admin/nests/getAllEggs.ts';
import { httpErrorToHuman } from '@/api/axios.ts';
import ActionIcon from '@/elements/ActionIcon.tsx';
import Button from '@/elements/Button.tsx';
import MultiSelect from '@/elements/input/MultiSelect.tsx';
import Select from '@/elements/input/Select.tsx';
import Tooltip from '@/elements/Tooltip.tsx';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useExtTranslations } from './translations.ts';

interface EggAssignment {
  affectedTypes: string[];
  egg: string;
}

interface NestGroup {
  nestUuid: string;
  nestName: string;
  eggs: { uuid: string; name: string }[];
}

interface AssignmentEntryProps {
  index: number;
  assignment: EggAssignment;
  allTypes: string[];
  usedTypes: string[];
  nestGroups: NestGroup[];
  onChange: (assignment: EggAssignment) => void;
  onRemove: () => void;
}

function AssignmentEntry({
  index,
  assignment,
  allTypes,
  usedTypes,
  nestGroups,
  onChange,
  onRemove,
}: AssignmentEntryProps) {
  const { t: tExt } = useExtTranslations();

  const initialNestUuid = useMemo(() => {
    if (!assignment.egg) return null;
    return nestGroups.find((g) => g.eggs.some((e) => e.uuid === assignment.egg))?.nestUuid ?? null;
  }, []);

  const [selectedNestUuid, setSelectedNestUuid] = useState<string | null>(initialNestUuid);

  useEffect(() => {
    if (!assignment.egg) return;
    const nest = nestGroups.find((g) => g.eggs.some((e) => e.uuid === assignment.egg));
    if (nest) setSelectedNestUuid(nest.nestUuid);
  }, [assignment.egg, nestGroups]);

  const availableTypes = allTypes.filter((t) => !usedTypes.includes(t) || assignment.affectedTypes.includes(t));

  const nestOptions = nestGroups.map((g) => ({ label: g.nestName, value: g.nestUuid }));
  const eggOptions = selectedNestUuid
    ? (nestGroups.find((g) => g.nestUuid === selectedNestUuid)?.eggs ?? []).map((e) => ({
        label: e.name,
        value: e.uuid,
      }))
    : [];

  const handleNestChange = (nestUuid: string | null) => {
    setSelectedNestUuid(nestUuid);
    onChange({ ...assignment, egg: '' });
  };

  return (
    <Paper withBorder p='sm' radius='sm'>
      <Stack gap='sm'>
        <Group gap='xs' align='flex-start' wrap='nowrap'>
          <Text size='xs' fw={500} c='dimmed' my='auto' style={{ flexShrink: 0 }}>
            #{index + 1}
          </Text>

          <MultiSelect
            className='flex-1'
            label={tExt('pages.admin.configuration.typeEggAssignments.form.affectedTypes', {})}
            placeholder={tExt('pages.admin.configuration.typeEggAssignments.form.affectedTypesPlaceholder', {})}
            data={availableTypes}
            value={assignment.affectedTypes}
            onChange={(v) => onChange({ ...assignment, affectedTypes: v })}
            searchable
            clearable
          />

          <Tooltip label={tExt('pages.admin.configuration.typeEggAssignments.tooltip.removeAssignment', {})}>
            <ActionIcon variant='subtle' color='red' size='input-md' mt={24} onClick={onRemove}>
              <FontAwesomeIcon icon={faTrash} style={{ fontSize: 14 }} />
            </ActionIcon>
          </Tooltip>
        </Group>

        <Group grow>
          <Select
            label={tExt('pages.admin.configuration.typeEggAssignments.form.nest', {})}
            data={nestOptions}
            value={selectedNestUuid}
            onChange={handleNestChange}
            searchable
          />
          <Select
            label={tExt('pages.admin.configuration.typeEggAssignments.form.egg', {})}
            data={eggOptions}
            value={assignment.egg || null}
            onChange={(v) => onChange({ ...assignment, egg: v ?? '' })}
            disabled={!selectedNestUuid}
            searchable
          />
        </Group>
      </Stack>
    </Paper>
  );
}

export interface TypeEggAssignmentsInputProps {
  assignments: EggAssignment[];
  allTypes: string[];
  onChange: (assignments: EggAssignment[]) => void;
}

export default function TypeEggAssignmentsInput({ assignments, allTypes, onChange }: TypeEggAssignmentsInputProps) {
  const { addToast } = useToast();
  const { t: tExt } = useExtTranslations();

  const [nestGroups, setNestGroups] = useState<NestGroup[]>([]);

  useEffect(() => {
    getAllEggs()
      .then((data) =>
        setNestGroups(
          data.map((v) => ({
            nestUuid: v.nest.uuid,
            nestName: v.nest.name,
            eggs: v.eggs.map((e) => ({ uuid: e.uuid, name: e.name })),
          })),
        ),
      )
      .catch((err) => addToast(httpErrorToHuman(err), 'error'));
  }, []);

  const usedTypesByIndex = (index: number) => assignments.flatMap((a, i) => (i !== index ? a.affectedTypes : []));

  const handleChange = (index: number, updated: EggAssignment) => {
    const next = [...assignments];
    next[index] = updated;
    onChange(next);
  };

  const handleRemove = (index: number) => {
    onChange(assignments.filter((_, i) => i !== index));
  };

  const handleAdd = () => {
    onChange([...assignments, { affectedTypes: [], egg: '' }]);
  };

  return (
    <Stack gap='sm'>
      {assignments.length === 0 ? (
        <Text size='sm' c='dimmed' ta='center' py='xs'>
          {tExt('pages.admin.configuration.typeEggAssignments.noEntries', {})}
        </Text>
      ) : (
        assignments.map((assignment, index) => (
          <AssignmentEntry
            key={index}
            index={index}
            assignment={assignment}
            allTypes={allTypes}
            usedTypes={usedTypesByIndex(index)}
            nestGroups={nestGroups}
            onChange={(updated) => handleChange(index, updated)}
            onRemove={() => handleRemove(index)}
          />
        ))
      )}

      <Divider />

      <Button
        variant='light'
        leftSection={<FontAwesomeIcon icon={faPlus} style={{ fontSize: 12 }} />}
        onClick={handleAdd}
      >
        {tExt('pages.admin.configuration.typeEggAssignments.button.addAssignment', {})}
      </Button>
    </Stack>
  );
}
