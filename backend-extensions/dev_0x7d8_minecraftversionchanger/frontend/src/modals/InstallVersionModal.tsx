import { faTriangleExclamation } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Stack } from '@mantine/core';
import { useEffect, useMemo, useState } from 'react';
import { httpErrorToHuman } from '@/api/axios.ts';
import Alert from '@/elements/Alert.tsx';
import Button from '@/elements/Button.tsx';
import Select from '@/elements/input/Select.tsx';
import Switch from '@/elements/input/Switch.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { useSearchableResource } from '@/plugins/useSearchableResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import getBuildsForVersionForType from '../api/getBuildsForVersionForType.ts';
import { MinecraftBuild, MinecraftVersion } from '../api/getVersionsForType.ts';
import installVersion from '../api/installVersion.ts';
import { useExtTranslations } from '../translations.ts';

interface Props {
  uuid: string;
  type: string | null;
  version: MinecraftVersion | null;
  typeName: string;
  isVanilla: boolean;
  opened: boolean;
  onClose: () => void;
  onInstalled: () => void;
}

export default function InstallVersionModal({
  uuid,
  type,
  version,
  typeName,
  isVanilla,
  opened,
  onClose,
  onInstalled,
}: Props) {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();
  const { addToast } = useToast();

  const [selectedBuild, setSelectedBuild] = useState<MinecraftBuild | null>(null);
  const [truncate, setTruncate] = useState(false);
  const [acceptEula, setAcceptEula] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (isVanilla) {
      setSelectedBuild(version?.latest ?? null);
    }
  }, [isVanilla, version]);

  const {
    items: builds,
    loading: buildsLoading,
    search: buildsSearch,
    setSearch: setBuildsSearch,
  } = useSearchableResource<MinecraftBuild>({
    queryKey: ['extension', 'minecraftversionchanger', 'builds', type, version?.id] as const,
    fetcher: (search) =>
      type && version
        ? getBuildsForVersionForType(uuid, type, version.id, 1, search)
        : Promise.resolve({ data: [], total: 0, perPage: 0, page: 1 }),
    deps: [type, version?.id],
    canRequest: opened && !!type && !!version,
  });

  useEffect(() => {
    if (isVanilla) return;
    if (builds.length === 0) return;
    if (selectedBuild && builds.some((b) => b.uuid === selectedBuild.uuid)) return;
    const stableBuild = builds.find((b) => !b.experimental);
    setSelectedBuild(stableBuild ?? builds.find((b) => b.experimental) ?? null);
  }, [builds, isVanilla, selectedBuild]);

  const buildSelectData = useMemo(
    () => [
      {
        group: tExt('pages.server.versions.stable', {}),
        items: builds.filter((b) => !b.experimental).map((b) => ({ label: b.name, value: b.uuid })),
      },
      {
        group: tExt('pages.server.versions.experimental', {}),
        items: builds.filter((b) => b.experimental).map((b) => ({ label: b.name, value: b.uuid })),
      },
    ],
    [builds, tExt],
  );

  const doInstall = () => {
    if (!selectedBuild) return;
    setLoading(true);
    installVersion(uuid, {
      buildUuid: selectedBuild.uuid,
      truncateDirectory: truncate,
      acceptEula,
    })
      .then(() => {
        addToast(tExt('pages.server.versions.toast.installed', {}), 'success');
        onInstalled();
      })
      .catch((error) => addToast(httpErrorToHuman(error), 'error'))
      .finally(() => setLoading(false));
  };

  return (
    <Modal
      title={tExt('pages.server.versions.modal.install.title', {
        type: typeName,
        version: version?.id ?? '',
      })}
      opened={opened}
      onClose={onClose}
    >
      <Stack>
        {selectedBuild?.experimental && (
          <Alert icon={<FontAwesomeIcon icon={faTriangleExclamation} />} color='red'>
            {tExt('pages.server.versions.modal.install.alert.experimental', {}).md()}
          </Alert>
        )}

        {!isVanilla && (
          <Select
            value={selectedBuild?.uuid ?? null}
            placeholder={tExt('pages.server.versions.modal.install.form.buildPlaceholder', {})}
            onChange={(value) => setSelectedBuild(builds.find((b) => b.uuid === value) ?? null)}
            searchable
            loading={buildsLoading}
            searchValue={buildsSearch}
            onSearchChange={setBuildsSearch}
            data={buildSelectData}
          />
        )}

        <Switch
          label={t('common.form.truncateDirectory', {})}
          name='truncate'
          checked={truncate}
          onChange={(e) => setTruncate(e.target.checked)}
          disabled={loading}
        />

        <Switch
          label={tExt('pages.server.versions.modal.install.form.acceptEula', {})}
          name='accept_eula'
          description={tExt('pages.server.versions.modal.install.form.acceptEulaDescription', {})}
          checked={acceptEula}
          onChange={(e) => setAcceptEula(e.target.checked)}
          disabled={loading}
        />
      </Stack>

      <ModalFooter>
        <Button color='red' onClick={doInstall} loading={loading} disabled={!selectedBuild}>
          {t('common.button.install', {})}
        </Button>
        <Button variant='default' onClick={onClose}>
          {t('common.button.close', {})}
        </Button>
      </ModalFooter>
    </Modal>
  );
}
