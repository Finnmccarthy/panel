import { faDiscord } from '@fortawesome/free-brands-svg-icons';
import { faDownload, faExclamationTriangle, faHeart, faWrench } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Stack } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { SemVer } from 'semver';
import { z } from 'zod';
import { httpErrorToHuman } from '@/api/axios.ts';
import Alert from '@/elements/Alert.tsx';
import Button from '@/elements/Button.tsx';
import Spinner from '@/elements/Spinner.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import TitleCard from '@/elements/TitleCard.tsx';
import FormattedTimestamp from '@/elements/time/FormattedTimestamp.tsx';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import get2038Info from './api/settings/get2038Info.ts';
import getSettings from './api/settings/getSettings.ts';
import updateSettings from './api/settings/updateSettings.ts';
import ExtensionSettingsEditor from './ExtensionSettingsEditor.tsx';
import { adminExtensionSettingsSchema } from './lib/schemas.ts';
import { useExtTranslations } from './translations.ts';

export default function Dev0x7d8EggChangerConfigurationPage() {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();
  const { addToast } = useToast();

  const [loading, setLoading] = useState(false);

  const form = useForm<z.infer<typeof adminExtensionSettingsSchema>>({
    initialValues: {
      eggGroups: [],
    },
    validateInputOnBlur: true,
    validate: zod4Resolver(adminExtensionSettingsSchema),
  });

  const { data: settings } = useResource({
    queryKey: ['extension', 'eggchanger', 'settings'],
    queryFn: getSettings,
  });
  const { data: extensionInfo } = useResource({
    queryKey: ['extension', 'eggchanger', '2038info'],
    queryFn: get2038Info,
  });

  useEffect(() => {
    if (settings) form.setValues({ ...settings });
  }, [settings]);

  const doSave = () => {
    setLoading(true);

    updateSettings(form.values)
      .then(() => {
        addToast(tExt('pages.admin.configuration.settings.toast.updated', {}), 'success');
      })
      .catch((err) => {
        addToast(httpErrorToHuman(err), 'error');
      })
      .finally(() => setLoading(false));
  };

  return (
    <div className='md:columns-2 gap-4 space-y-4'>
      <TitleCard
        title={tExt('pages.admin.configuration.settings.title', {})}
        icon={<FontAwesomeIcon icon={faWrench} />}
        className='w-full'
      >
        <form onSubmit={form.onSubmit(() => doSave())}>
          <Stack>
            <ExtensionSettingsEditor form={form} />

            <Button type='submit' loading={loading} disabled={!form.isValid()} className='w-fit!'>
              {t('common.button.save', {})}
            </Button>
          </Stack>
        </form>
      </TitleCard>

      <TitleCard
        title={tExt('pages.admin.configuration.thankYou.title', {})}
        icon={<FontAwesomeIcon icon={faHeart} />}
        className='w-full'
      >
        <p>{tExt('pages.admin.configuration.thankYou.description', {})}</p>

        {!extensionInfo ? (
          <Spinner.Centered />
        ) : (
          <>
            {new SemVer(extensionInfo.currentVersion).compare(extensionInfo.product.version) === -1 && (
              <Alert icon={<FontAwesomeIcon icon={faExclamationTriangle} />} mt='md' color='yellow'>
                <div className='flex flex-col'>
                  {tExt('pages.admin.configuration.thankYou.alert.newVersion', {})}

                  <div className='mt-2 flex flex-col md:flex-row space-x-2 w-full'>
                    {Object.values(extensionInfo?.providers || {}).map((provider) => (
                      <a
                        key={provider.name}
                        href={provider.link}
                        target='_blank'
                        rel='noopener noreferrer'
                        className='grow'
                      >
                        <Button leftSection={<FontAwesomeIcon icon={faDownload} />} className='w-full!'>
                          {tExt('pages.admin.configuration.thankYou.button.downloadFrom', {
                            version: extensionInfo.product.version,
                            provider: provider.name,
                          })}
                        </Button>
                      </a>
                    ))}
                  </div>
                </div>
              </Alert>
            )}
          </>
        )}

        <div className='mt-4' hidden={Object.keys(extensionInfo?.changelogs || {}).length === 0}>
          <Table
            columns={[
              tExt('pages.admin.configuration.thankYou.table.columns.version', {}),
              tExt('pages.admin.configuration.thankYou.table.columns.changelog', {}),
              t('common.table.columns.created', {}),
            ]}
            loading={!extensionInfo}
          >
            {Object.entries(extensionInfo?.changelogs || {}).map(([version, changelog]) => (
              <TableRow key={version}>
                <TableData>{version}</TableData>
                <TableData className='whitespace-pre-wrap'>{changelog.content}</TableData>
                <TableData>
                  <FormattedTimestamp timestamp={changelog.created} />
                </TableData>
              </TableRow>
            ))}
          </Table>
        </div>

        <a href='https://discord.2038.buzz' target='_blank' rel='noopener noreferrer'>
          <Button leftSection={<FontAwesomeIcon icon={faDiscord} />} mt='md'>
            {tExt('pages.admin.configuration.thankYou.button.joinDiscord', {})}
          </Button>
        </a>
      </TitleCard>
    </div>
  );
}
