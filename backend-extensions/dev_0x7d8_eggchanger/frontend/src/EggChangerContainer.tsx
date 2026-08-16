import { faEgg } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Group, Stack } from '@mantine/core';
import { useEffect, useState } from 'react';
import { z } from 'zod';
import Button from '@/elements/Button.tsx';
import { ServerCan } from '@/elements/Can.tsx';
import Select from '@/elements/input/Select.tsx';
import Spinner from '@/elements/Spinner.tsx';
import TitleCard from '@/elements/TitleCard';
import { serverEggSchema } from '@/lib/schemas/server/server.ts';
import { useResource } from '@/plugins/useResource.ts';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import { useServerStore } from '@/stores/server.ts';
import getAvailable from './api/getAvailable.ts';
import ChangeEggModal from './modals/ChangeEggModal.tsx';
import { useExtTranslations } from './translations.ts';

export default function EggChangerContainer() {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();

  const uuid = useServerStore((state) => state.server.uuid);
  const egg = useServerStore((state) => state.server.egg);

  const [selectedEgg, setSelectedEgg] = useState<z.infer<typeof serverEggSchema>>(egg);
  const [openModal, setOpenModal] = useState<'changeEgg' | null>(null);

  useEffect(() => {
    setSelectedEgg(egg);
  }, [egg]);

  const { data: eggGroups } = useResource({
    queryKey: ['extension', 'eggchanger', 'available', uuid],
    queryFn: () => getAvailable(uuid),
  });

  return (
    <ServerCan action='settings.egg-changer'>
      <TitleCard
        title={tExt('pages.server.settings.eggchanger.title', {})}
        icon={<FontAwesomeIcon icon={faEgg} />}
        className='h-full order-100'
      >
        <ChangeEggModal
          opened={openModal === 'changeEgg'}
          onClose={() => setOpenModal(null)}
          selectedEgg={selectedEgg}
          selectedEggGroup={eggGroups?.find((g) => g.eggs.some((e) => e.uuid === selectedEgg.uuid)) ?? null}
        />

        {!eggGroups ? (
          <Spinner.Centered />
        ) : eggGroups.length === 0 ? (
          <Stack>{tExt('pages.server.settings.eggchanger.noEggs', {})}</Stack>
        ) : (
          <Stack>
            <Select
              withAsterisk
              label={tExt('pages.server.settings.eggchanger.form.egg', {})}
              placeholder={tExt('pages.server.settings.eggchanger.form.egg', {})}
              value={selectedEgg.uuid}
              data={eggGroups.map((group, index) => ({
                group: group.name,
                items: group.eggs
                  .filter((egg) => !eggGroups.slice(0, index).some((g) => g.eggs.some((e) => e.uuid === egg.uuid)))
                  .map((egg) => ({
                    label: egg.name,
                    value: egg.uuid,
                  })),
              }))}
              onChange={(val) => setSelectedEgg(eggGroups.flatMap((g) => g.eggs).find((e) => e.uuid === val)!)}
              searchable
            />

            <Group mt='auto'>
              <Button onClick={() => setOpenModal('changeEgg')}>{t('common.button.update', {})}</Button>
            </Group>
          </Stack>
        )}
      </TitleCard>
    </ServerCan>
  );
}
