import { ModalProps, Stack } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { z } from 'zod';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import Switch from '@/elements/input/Switch.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { serverEggSchema } from '@/lib/schemas/server/server';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import { useServerStore } from '@/stores/server.ts';
import { EggGroup } from '../api/getAvailable.ts';
import switchEgg from '../api/switchEgg.ts';
import { serverSettingsChangeEggSchema } from '../lib/schemas.ts';
import { useExtTranslations } from '../translations.ts';

interface Props extends ModalProps {
  selectedEgg: z.infer<typeof serverEggSchema>;
  selectedEggGroup: EggGroup | null;
}

export default function ChangeEggModal({ opened, onClose, selectedEgg, selectedEggGroup }: Props) {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();
  const { addToast } = useToast();
  const { server, updateServer } = useServerStore();
  const navigate = useNavigate();

  const [loading, setLoading] = useState(false);

  const form = useForm<z.infer<typeof serverSettingsChangeEggSchema>>({
    initialValues: {
      eggUuid: selectedEgg.uuid,
      updateStartup: false,
      reinstall: false,
      truncateDirectory: false,
    },
    validateInputOnBlur: true,
    validate: zod4Resolver(serverSettingsChangeEggSchema),
  });

  const doChangeEgg = () => {
    setLoading(true);

    switchEgg(server.uuid, form.values)
      .then(() => {
        if (form.values.reinstall) {
          addToast(tExt('pages.server.settings.eggchanger.modal.toast.changedWithReinstall', {}), 'success');
          navigate(`/server/${server.uuidShort}`);
          updateServer({ status: 'installing' });
        } else {
          addToast(tExt('pages.server.settings.eggchanger.modal.toast.changed', {}), 'success');
        }

        updateServer({ egg: selectedEgg });
        onClose();
      })
      .catch((msg) => {
        addToast(httpErrorToHuman(msg), 'error');
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    form.setValues((v) => ({
      ...v,
      eggUuid: selectedEgg.uuid,
    }));
  }, [selectedEgg.uuid]);

  useEffect(() => {
    if (selectedEggGroup) {
      form.setValues((v) => ({
        ...v,
        updateStartup: selectedEggGroup.forceUpdateStartup,
        reinstall: selectedEggGroup.forceReinstall,
        truncateDirectory: selectedEggGroup.forceReinstallTruncateFiles,
      }));
    }
  }, [selectedEggGroup]);

  return (
    <Modal title={tExt('pages.server.settings.eggchanger.modal.title', {})} onClose={onClose} opened={opened}>
      <form onSubmit={form.onSubmit(() => doChangeEgg())}>
        <Stack>
          {tExt('pages.server.settings.eggchanger.modal.content', { eggName: selectedEgg.name }).md()}

          <Switch
            label={tExt('pages.server.settings.eggchanger.modal.form.updateStartup', {})}
            description={tExt('pages.server.settings.eggchanger.modal.form.updateStartupDescription', {})}
            name='updateStartup'
            disabled={selectedEggGroup?.forceUpdateStartup}
            {...form.getInputProps('updateStartup', { type: 'checkbox' })}
          />
          <Switch
            label={tExt('pages.server.settings.eggchanger.modal.form.reinstall', {})}
            description={tExt('pages.server.settings.eggchanger.modal.form.reinstallDescription', {})}
            name='reinstall'
            disabled={selectedEggGroup?.forceReinstall}
            {...form.getInputProps('reinstall', { type: 'checkbox' })}
          />
          <Switch
            label={t('common.form.truncateDirectory', {})}
            name='truncate'
            disabled={selectedEggGroup?.forceReinstallTruncateFiles}
            {...form.getInputProps('truncateDirectory', { type: 'checkbox' })}
          />
        </Stack>

        <ModalFooter>
          <Button
            color={form.values.truncateDirectory ? 'red' : 'blue'}
            type='submit'
            loading={loading}
            disabled={!form.isValid()}
          >
            {t('common.button.update', {})}
          </Button>
          <Button variant='default' onClick={onClose}>
            {t('common.button.close', {})}
          </Button>
        </ModalFooter>
      </form>
    </Modal>
  );
}
