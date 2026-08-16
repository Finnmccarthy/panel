import { faDiscord } from '@fortawesome/free-brands-svg-icons';
import { faDownload, faExclamationTriangle, faGlobe, faHeart, faPlus } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Text } from '@mantine/core';
import { useState } from 'react';
import { SemVer } from 'semver';
import { z } from 'zod';
import getAllEggs from '@/api/admin/nests/getAllEggs.ts';
import Alert from '@/elements/Alert.tsx';
import Button from '@/elements/Button.tsx';
import Spinner from '@/elements/Spinner.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import TitleCard from '@/elements/TitleCard.tsx';
import FormattedTimestamp from '@/elements/time/FormattedTimestamp.tsx';
import { useResource } from '@/plugins/useResource.ts';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import getDomains from './api/domains/getDomains.ts';
import get2038Info from './api/settings/get2038Info.ts';
import DomainRow from './DomainRow.tsx';
import DomainSubdomainsPanel from './DomainSubdomainsPanel.tsx';
import { domainSchema } from './lib/schemas.ts';
import DomainFormModal from './modals/DomainFormModal.tsx';
import { useExtTranslations } from './translations.ts';

type EggGroup = { group: string; items: { label: string; value: string }[] };

export default function Dev0x7d8SubdomainManagerConfigurationPage() {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();

  const [createOpened, setCreateOpened] = useState(false);
  const [selectedDomain, setSelectedDomain] = useState<z.infer<typeof domainSchema> | null>(null);

  const { data: domains, invalidate: invalidateDomains } = useResource({
    queryKey: ['extension', 'subdomainmanager', 'domains'],
    queryFn: getDomains,
  });

  const { data: extensionInfo } = useResource({
    queryKey: ['extension', 'subdomainmanager', '2038info'],
    queryFn: get2038Info,
  });

  const { data: eggGroups = [] } = useResource<EggGroup[]>({
    queryKey: ['extension', 'subdomainmanager', 'eggs'],
    queryFn: async () => {
      const nests = await getAllEggs();
      return nests.map((nest) => ({
        group: nest.nest.name,
        items: nest.eggs.map((egg) => ({ label: egg.name, value: egg.uuid })),
      }));
    },
  });

  return (
    <>
      <DomainFormModal
        opened={createOpened}
        onClose={() => setCreateOpened(false)}
        eggGroups={eggGroups}
        onSaved={invalidateDomains}
      />

      <div className='md:columns-2 gap-4 space-y-4'>
        <TitleCard
          title={tExt('pages.admin.configuration.domains.title', {})}
          icon={<FontAwesomeIcon icon={faGlobe} />}
          className='w-full'
          rightSection={
            <Button
              leftSection={<FontAwesomeIcon icon={faPlus} />}
              size='xs'
              ml='auto'
              onClick={() => setCreateOpened(true)}
            >
              {tExt('pages.admin.configuration.domains.button.add', {})}
            </Button>
          }
        >
          {!domains ? (
            <Spinner.Centered />
          ) : domains.length === 0 ? (
            <Text c='dimmed' ta='center' py='sm' size='sm'>
              {tExt('pages.admin.configuration.domains.empty', {})}
            </Text>
          ) : (
            <Table
              columns={[
                tExt('pages.admin.configuration.domains.table.columns.domain', {}),
                tExt('pages.admin.configuration.domains.table.columns.provider', {}),
                tExt('pages.admin.configuration.domains.table.columns.eggs', {}),
                '',
              ]}
            >
              {domains.map((domain) => (
                <DomainRow
                  key={domain.uuid}
                  domain={domain}
                  eggGroups={eggGroups}
                  onUpdated={invalidateDomains}
                  onViewSubdomains={() => setSelectedDomain(domain)}
                />
              ))}
            </Table>
          )}
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
                      {Object.values(extensionInfo.providers).map((provider) => (
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

      {selectedDomain && (
        <DomainSubdomainsPanel
          key={selectedDomain.uuid}
          domain={selectedDomain}
          onClose={() => setSelectedDomain(null)}
        />
      )}
    </>
  );
}
