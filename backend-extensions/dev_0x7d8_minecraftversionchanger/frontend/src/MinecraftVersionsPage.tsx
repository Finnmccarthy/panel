import {
  faArrowLeft,
  faExclamationTriangle,
  faRefresh,
  faSearch,
  faSkull,
  faTriangleExclamation,
} from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Title } from '@mantine/core';
import { useEffect, useMemo, useState } from 'react';
import { useLocation } from 'react-router';
import Alert from '@/elements/Alert.tsx';
import Badge from '@/elements/Badge.tsx';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import ServerContentContainer from '@/elements/containers/ServerContentContainer.tsx';
import Divider from '@/elements/Divider.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import Spinner from '@/elements/Spinner.tsx';
import { Pagination } from '@/elements/Table.tsx';
import Tooltip from '@/elements/Tooltip.tsx';
import { useResource } from '@/plugins/useResource.ts';
import { useSearchablePaginatedTable } from '@/plugins/useSearchablePaginatedTable.ts';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import { useServerStore } from '@/stores/server.ts';
import getInstalled from './api/getInstalled.ts';
import getTypes, { MinecraftVersionProviderType } from './api/getTypes.ts';
import getVersionsForType, { MinecraftBuild, MinecraftVersion } from './api/getVersionsForType.ts';
import InstallVersionModal from './modals/InstallVersionModal.tsx';
import { useExtTranslations } from './translations.ts';

function VersionList({
  uuid,
  type,
  typeMeta,
  installed,
  installedType,
  onBack,
  onSelect,
}: {
  uuid: string;
  type: string;
  typeMeta?: MinecraftVersionProviderType;
  installed: { build: MinecraftBuild; latest: MinecraftBuild } | null;
  installedType: MinecraftVersionProviderType | null;
  onBack: () => void;
  onSelect: (version: MinecraftVersion) => void;
}) {
  const { t } = useTranslations();
  const { t: tExt, tItem: tItemExt } = useExtTranslations();
  const [data, setData] = useState<Pagination<MinecraftVersion>>();

  const { loading, search, setSearch, setPage } = useSearchablePaginatedTable({
    queryKey: ['extension', 'minecraftversionchanger', 'versions', type] as const,
    fetcher: (page, search) => getVersionsForType(uuid, type, page, search || undefined),
    setStoreData: setData,
    deps: [type],
    modifyParams: false,
  });

  return (
    <>
      <div className='mb-3 flex gap-2'>
        <Button variant='light' leftSection={<FontAwesomeIcon icon={faArrowLeft} />} onClick={onBack}>
          {t('common.button.back', {})}
        </Button>
        <TextInput
          className='flex-1'
          placeholder={t('common.input.search', {})}
          leftSection={<FontAwesomeIcon icon={faSearch} />}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>

      {!data || loading ? (
        <Spinner.Centered />
      ) : (
        <>
          {data.total > data.perPage && <Pagination data={data} onPageSelect={setPage} m='xs' />}

          <div className='w-full grid grid-cols-[repeat(auto-fill,minmax(20rem,1fr))] gap-2'>
            {data.data.map((version) => (
              <Card
                key={version.id}
                className='transition-all p-3 rounded-md w-full min-w-[20rem] select-none flex flex-row! justify-between items-center'
                hoverable
                onClick={() => onSelect(version)}
                leftStripeClassName={
                  (installed?.build.versionId ?? installed?.build.projectVersionId) === version.id &&
                  installedType?.identifier === type
                    ? installed?.build.uuid !== installed?.latest.uuid
                      ? 'bg-yellow-500'
                      : 'bg-green-500'
                    : undefined
                }
              >
                <img src={typeMeta?.icon} className='rounded object-cover w-10 h-10 mr-2' />
                <div className='flex flex-row pl-2 justify-between w-full'>
                  <div className='h-full w-full'>
                    <div className='grid grid-cols-5'>
                      <Title
                        order={4}
                        className='wrap-break-word w-auto h-auto text-white/90 text-xl col-span-3 flex flex-col'
                      >
                        {version.id}
                        <Badge color={version.type === 'RELEASE' ? 'blue' : 'yellow'}>
                          {tExt(`pages.server.versions.${version.type === 'RELEASE' ? 'release' : 'snapshot'}`, {})}
                        </Badge>
                      </Title>
                      <p className='my-auto col-span-2 text-right pr-1'>{tItemExt('build', version.builds)}</p>
                    </div>
                  </div>
                </div>
              </Card>
            ))}
          </div>

          <Pagination data={data} onPageSelect={setPage} m='xs' />
        </>
      )}
    </>
  );
}

export default function MinecraftVersionsPage() {
  const { t: tExt, tItem: tItemExt } = useExtTranslations();
  const location = useLocation();
  const uuid = useServerStore((state) => state.server.uuid);

  const [type, setType] = useState<string>();
  const [selectedVersion, setSelectedVersion] = useState<MinecraftVersion | null>(null);
  const [displayedVersion, setDisplayedVersion] = useState<MinecraftVersion | null>(null);

  useEffect(() => {
    if (selectedVersion) setDisplayedVersion(selectedVersion);
  }, [selectedVersion]);

  const { data: types } = useResource({
    queryKey: ['extension', 'minecraftversionchanger', 'types', uuid],
    queryFn: () => getTypes(uuid),
  });

  const { data: installed, refetch: refetchInstalled } = useResource({
    queryKey: ['extension', 'minecraftversionchanger', 'installed', uuid],
    queryFn: () => getInstalled(uuid),
  });

  const allTypes = useMemo(() => Object.values(types ?? {}).reduce((acc, val) => acc.concat(val), []), [types]);

  const installedType = useMemo(
    () => (installed ? (allTypes.find((t) => t.identifier === installed.build.type) ?? null) : null),
    [installed, allTypes],
  );

  const typeMeta = useMemo(() => allTypes.find((t) => t.identifier === type), [allTypes, type]);

  useEffect(() => {
    if (allTypes.length === 0) return;
    const params = new URLSearchParams(location.search);
    setType(allTypes.find((t) => t.identifier === params.get('type')?.toUpperCase())?.identifier ?? undefined);
  }, [allTypes, location]);

  useEffect(() => {
    const searchParams = new URLSearchParams(location.search);
    if (!type) {
      searchParams.delete('type');
    } else {
      searchParams.set('type', type.toUpperCase());
    }
    window.history.pushState(
      {},
      '',
      `${location.pathname}${searchParams?.toString() ? '?' : ''}${searchParams?.toString()}`,
    );
  }, [type]);

  if (!types || installed === undefined) {
    return <Spinner.Centered />;
  }

  return (
    <ServerContentContainer title={tExt('pages.server.versions.title', {})}>
      <InstallVersionModal
        uuid={uuid}
        type={type ?? null}
        version={displayedVersion}
        typeName={typeMeta?.name ?? ''}
        isVanilla={type?.toUpperCase() === 'VANILLA'}
        opened={!!selectedVersion}
        onClose={() => setSelectedVersion(null)}
        onInstalled={() => {
          refetchInstalled();
          setSelectedVersion(null);
        }}
      />

      {installed && installedType && (
        <>
          <Card
            className='w-full flex flex-row! justify-between items-center'
            leftStripeClassName={installed.build.uuid !== installed.latest.uuid ? 'bg-yellow-500' : 'bg-green-500'}
          >
            <img src={installedType.icon} className='rounded object-cover select-none w-16 h-16 mr-2' />
            <div className='flex flex-row pl-2 justify-between w-full'>
              <div className='flex flex-col h-full justify-between w-full'>
                <div className='flex flex-col'>
                  <Title order={3} className='wrap-break-word w-auto h-auto text-xl text-white/90'>
                    {tExt('pages.server.versions.alert.currentlyRunning', { type: installedType.name })}
                    {installedType.experimental && (
                      <Tooltip label={tExt('pages.server.versions.tooltip.experimental', {})}>
                        <span className='ml-2 text-yellow-500'>
                          <FontAwesomeIcon icon={faExclamationTriangle} />
                        </span>
                      </Tooltip>
                    )}
                    {installedType.deprecated && (
                      <Tooltip label={tExt('pages.server.versions.tooltip.deprecated', {})}>
                        <span className='ml-2 text-red-500'>
                          <FontAwesomeIcon icon={faSkull} />
                        </span>
                      </Tooltip>
                    )}
                  </Title>
                  {installed.build.versionId ? (
                    <p>
                      {tExt('pages.server.versions.alert.installedVersion', { version: installed.build.versionId })}
                    </p>
                  ) : (
                    <p>
                      {tExt('pages.server.versions.alert.installedProjectVersion', {
                        projectVersion: installed.build.projectVersionId ?? '',
                      })}
                    </p>
                  )}
                  {installed.build.type !== 'VANILLA' && (
                    <p>{tExt('pages.server.versions.alert.installedBuild', { build: installed.build.name })}</p>
                  )}
                </div>
              </div>
            </div>
          </Card>

          {installed.build.uuid !== installed.latest.uuid && (
            <Alert icon={<FontAwesomeIcon icon={faTriangleExclamation} />} color='yellow' className='my-2'>
              <div className='flex flex-row items-center justify-between w-full'>
                <p>
                  {tExt('pages.server.versions.alert.outdatedVersion', {
                    type: installedType.name,
                    version: installed.build.versionId ?? installed.build.projectVersionId ?? '',
                    build: installed.latest.name,
                  }).md()}
                </p>
                <Button
                  leftSection={<FontAwesomeIcon icon={faRefresh} />}
                  color='yellow'
                  variant='light'
                  onClick={() => setType(installed.build.type)}
                  disabled={type === installed.build.type}
                >
                  {tExt('pages.server.versions.button.viewVersions', {})}
                </Button>
              </div>
            </Alert>
          )}
        </>
      )}

      <div className='w-full flex flex-col'>
        {!type ? (
          Object.entries(types).map(([category, categoryTypes]) => (
            <div key={category}>
              <Divider label={category} labelPosition='left' className='my-4' />

              <div className='w-full grid grid-cols-[repeat(auto-fill,minmax(20rem,1fr))] gap-2'>
                {categoryTypes.map((t) => (
                  <Card
                    key={t.identifier}
                    onClick={() => setType(t.identifier)}
                    hoverable
                    className='transition-all w-full min-w-[20rem] select-none flex flex-row! justify-between items-center'
                    leftStripeClassName={
                      installedType?.identifier === t.identifier
                        ? installed?.build.uuid !== installed?.latest.uuid
                          ? 'bg-yellow-500'
                          : 'bg-green-500'
                        : undefined
                    }
                  >
                    <img src={t.icon} className='rounded object-cover w-16 h-16 mr-2' />
                    <div className='flex flex-row pl-2 justify-between w-full'>
                      <div className='flex flex-col h-full justify-between w-full'>
                        <div className='flex flex-col'>
                          <Title
                            order={4}
                            className='wrap-break-word w-auto h-auto text-xl flex flex-row text-white/80'
                          >
                            {t.name}
                            {t.experimental && (
                              <Tooltip label={tExt('pages.server.versions.tooltip.experimental', {})}>
                                <span className='ml-2 text-yellow-500'>
                                  <FontAwesomeIcon icon={faExclamationTriangle} />
                                </span>
                              </Tooltip>
                            )}
                            {t.deprecated && (
                              <Tooltip label={tExt('pages.server.versions.tooltip.deprecated', {})}>
                                <span className='ml-2 text-red-500'>
                                  <FontAwesomeIcon icon={faSkull} />
                                </span>
                              </Tooltip>
                            )}
                          </Title>
                          {t.versions.minecraft > 0 ? (
                            <p>{tItemExt('version', t.versions.minecraft)}</p>
                          ) : (
                            <p>{tItemExt('projectVersion', t.versions.project)}</p>
                          )}
                          <p>{tItemExt('build', t.builds)}</p>
                        </div>
                      </div>
                    </div>
                  </Card>
                ))}
              </div>
            </div>
          ))
        ) : (
          <div className='mb-2'>
            <Divider label={typeMeta?.name} labelPosition='left' className='my-4' />

            <VersionList
              uuid={uuid}
              type={type}
              typeMeta={typeMeta}
              installed={installed}
              installedType={installedType}
              onBack={() => setType(undefined)}
              onSelect={setSelectedVersion}
            />
          </div>
        )}
      </div>
    </ServerContentContainer>
  );
}
