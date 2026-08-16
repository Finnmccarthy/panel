import { faDiscord } from '@fortawesome/free-brands-svg-icons';
import {
  faChartArea,
  faDownload,
  faExclamationTriangle,
  faHeart,
  faList,
  faServer,
  faWrench,
} from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { PieChart } from '@mantine/charts';
import { Group, Stack, Title } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { SemVer } from 'semver';
import { z } from 'zod';
import { httpErrorToHuman } from '@/api/axios.ts';
import Alert from '@/elements/Alert.tsx';
import Button from '@/elements/Button.tsx';
import CollapsibleSection from '@/elements/CollapsibleSection.tsx';
import PasswordInput from '@/elements/input/PasswordInput.tsx';
import Switch from '@/elements/input/Switch.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import Spinner from '@/elements/Spinner.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import TitleCard from '@/elements/TitleCard.tsx';
import FormattedTimestamp from '@/elements/time/FormattedTimestamp.tsx';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import get2038Info from './api/settings/get2038Info.ts';
import getDefaultOrder from './api/settings/getDefaultOrder.ts';
import getSettings from './api/settings/getSettings.ts';
import getTotalStatistics from './api/settings/statistics/getTotal.ts';
import updateSettings from './api/settings/updateSettings.ts';
import { adminExtensionSettingsSchema } from './lib/schemas.ts';
import TypeEggAssignmentsInput from './TypeEggAssignmentsInput.tsx';
import TypeOrderInput from './TypeOrderInput.tsx';
import { useExtTranslations } from './translations.ts';

export default function Dev0x7d8MinecraftVersionChangerConfigurationPage() {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();
  const { addToast } = useToast();

  const [loading, setLoading] = useState(false);

  const form = useForm<z.infer<typeof adminExtensionSettingsSchema>>({
    initialValues: {
      mcjarsUrl: '',
      mcjarsApiKey: '',
      mcjarsIconBaseUrl: '',
      mcjarsIconFileExtension: '',
      mcjarsTypeOrder: null,
      typeEggAssignments: [],
      collectInstallationStatistics: false,
    },
    validateInputOnBlur: true,
    validate: zod4Resolver(adminExtensionSettingsSchema),
  });

  const { data: settings } = useResource({
    queryKey: ['extension', 'minecraftversionchanger', 'settings'],
    queryFn: getSettings,
  });
  const { data: defaultOrder } = useResource({
    queryKey: ['extension', 'minecraftversionchanger', 'defaultOrder'],
    queryFn: getDefaultOrder,
  });
  const { data: totalStatistics } = useResource({
    queryKey: ['extension', 'minecraftversionchanger', 'totalStatistics'],
    queryFn: getTotalStatistics,
  });
  const { data: extensionInfo } = useResource({
    queryKey: ['extension', 'minecraftversionchanger', '2038info'],
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
            <TextInput
              withAsterisk
              label={tExt('pages.admin.configuration.settings.form.mcjarsUrl', {})}
              placeholder={tExt('pages.admin.configuration.settings.form.mcjarsUrl', {})}
              description={tExt('pages.admin.configuration.settings.form.mcjarsUrlDescription', {})}
              {...form.getInputProps('mcjarsUrl')}
            />
            <PasswordInput
              label={tExt('pages.admin.configuration.settings.form.mcjarsApiKey', {})}
              placeholder={tExt('pages.admin.configuration.settings.form.mcjarsApiKey', {})}
              description={tExt('pages.admin.configuration.settings.form.mcjarsApiKeyDescription', {})}
              {...form.getInputProps('mcjarsApiKey')}
            />

            <Group grow>
              <TextInput
                withAsterisk
                label={tExt('pages.admin.configuration.settings.form.mcjarsIconBaseUrl', {})}
                placeholder={tExt('pages.admin.configuration.settings.form.mcjarsIconBaseUrl', {})}
                description={tExt('pages.admin.configuration.settings.form.mcjarsIconBaseUrlDescription', {})}
                {...form.getInputProps('mcjarsIconBaseUrl')}
              />
              <TextInput
                withAsterisk
                label={tExt('pages.admin.configuration.settings.form.mcjarsIconFileExtension', {})}
                placeholder={tExt('pages.admin.configuration.settings.form.mcjarsIconFileExtension', {})}
                description={tExt('pages.admin.configuration.settings.form.mcjarsIconFileExtensionDescription', {})}
                {...form.getInputProps('mcjarsIconFileExtension')}
              />
            </Group>

            <CollapsibleSection
              icon={<FontAwesomeIcon icon={faList} />}
              title={tExt('pages.admin.configuration.settings.form.mcjarsTypeOrder', {})}
              enabled={form.values.mcjarsTypeOrder !== null}
              onToggle={(enabled) => form.setFieldValue('mcjarsTypeOrder', enabled ? defaultOrder || {} : null)}
            >
              <TypeOrderInput
                value={form.values.mcjarsTypeOrder || {}}
                allowedTypes={defaultOrder ? Object.values(defaultOrder).flat() : []}
                onChange={(newValue) => form.setFieldValue('mcjarsTypeOrder', newValue)}
              />
            </CollapsibleSection>

            <Switch
              label={tExt('pages.admin.configuration.settings.form.collectInstallationStatistics', {})}
              description={tExt('pages.admin.configuration.settings.form.collectInstallationStatisticsDescription', {})}
              {...form.getInputProps('collectInstallationStatistics', { type: 'checkbox' })}
            />

            <Button type='submit' loading={loading} disabled={!form.isValid()} className='w-fit!'>
              {t('common.button.save', {})}
            </Button>
          </Stack>
        </form>
      </TitleCard>

      <TitleCard
        title={tExt('pages.admin.configuration.typeEggAssignments.title', {})}
        icon={<FontAwesomeIcon icon={faServer} />}
        className='w-full'
      >
        <p className='mb-4'>{tExt('pages.admin.configuration.typeEggAssignments.description', {})}</p>

        <TypeEggAssignmentsInput
          assignments={form.values.typeEggAssignments}
          allTypes={defaultOrder ? Object.values(defaultOrder).flat() : []}
          onChange={(v) => form.setFieldValue('typeEggAssignments', v)}
        />
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

      <TitleCard
        title={tExt('pages.admin.configuration.defaultTypeOrder.title', {})}
        icon={<FontAwesomeIcon icon={faList} />}
        className='w-full'
      >
        <p>{tExt('pages.admin.configuration.defaultTypeOrder.description', {})}</p>

        <div className='mt-4'>
          {!defaultOrder ? (
            <Spinner.Centered />
          ) : (
            <TypeOrderInput value={defaultOrder} allowedTypes={Object.values(defaultOrder).flat()} readOnly />
          )}
        </div>
      </TitleCard>

      <TitleCard
        title={tExt('pages.admin.configuration.statistics.title', {})}
        icon={<FontAwesomeIcon icon={faChartArea} />}
        className='w-full'
      >
        <p>{tExt('pages.admin.configuration.statistics.description', {})}</p>

        <div className='mt-4'>
          {!totalStatistics ? (
            <Spinner.Centered />
          ) : (
            <div className='space-y-4'>
              <p>
                {tExt('pages.admin.configuration.statistics.descriptionAmount', {
                  totalInstallations: totalStatistics.totalInstallations,
                })}
              </p>

              <div className='w-full grid grid-cols-1 md:grid-cols-2 gap-4'>
                <div className='flex flex-col'>
                  <Title order={4} mb='xs' className='text-center'>
                    {tExt('pages.admin.configuration.statistics.byType', {})}
                  </Title>

                  {Object.values(totalStatistics.installationsByType).some((v) => v > 0) ? (
                    <div className='overflow-hidden aspect-square w-full max-w-xs mx-auto'>
                      <PieChart
                        w='100%'
                        h='100%'
                        withTooltip
                        tooltipDataSource='segment'
                        data={Object.entries(totalStatistics.installationsByType).map(([name, value], index) => ({
                          name,
                          value,
                          color: `hsl(${(index * 137.508) % 360} 70% 50%)`,
                        }))}
                      />
                    </div>
                  ) : (
                    <p className='text-center'>{tExt('pages.admin.configuration.statistics.noStatistics', {})}</p>
                  )}
                </div>

                <div className='flex flex-col'>
                  <Title order={4} mb='xs' className='text-center'>
                    {tExt('pages.admin.configuration.statistics.byVersion', {})}
                  </Title>
                  {Object.values(totalStatistics.installationsByVersion).some((v) => v > 0) ? (
                    <div className='overflow-hidden aspect-square w-full max-w-xs mx-auto'>
                      <PieChart
                        w='100%'
                        h='100%'
                        withTooltip
                        tooltipDataSource='segment'
                        data={Object.entries(totalStatistics.installationsByVersion).map(([name, value], index) => ({
                          name,
                          value,
                          color: `hsl(${(index * 137.508) % 360} 70% 50%)`,
                        }))}
                      />
                    </div>
                  ) : (
                    <p className='text-center'>{tExt('pages.admin.configuration.statistics.noStatistics', {})}</p>
                  )}
                </div>
              </div>
            </div>
          )}
        </div>
      </TitleCard>
    </div>
  );
}
