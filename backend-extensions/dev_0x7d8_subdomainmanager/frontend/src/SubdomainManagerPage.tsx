import { faPencil, faPlus, faTrash } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { useState } from 'react';
import { z } from 'zod';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import { ServerCan } from '@/elements/Can.tsx';
import Code from '@/elements/Code.tsx';
import ConditionalTooltip from '@/elements/ConditionalTooltip.tsx';
import ContextMenu, { ContextMenuToggle } from '@/elements/ContextMenu.tsx';
import ServerContentContainer from '@/elements/containers/ServerContentContainer.tsx';
import ConfirmationModal from '@/elements/modals/ConfirmationModal.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import FormattedTimestamp from '@/elements/time/FormattedTimestamp.tsx';
import { formatAllocation } from '@/lib/server.ts';
import { useServerCan } from '@/plugins/usePermissions.ts';
import { useResource } from '@/plugins/useResource.ts';
import { useSearchablePaginatedTable } from '@/plugins/useSearchablePaginatedTable.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import { useServerStore } from '@/stores/server.ts';
import deleteSubdomain from './api/deleteSubdomain.ts';
import getServerDomains from './api/getServerDomains.ts';
import getSubdomains from './api/getSubdomains.ts';
import { serverWithSubdomainsFeatureLimitsSchema, subdomainEntrySchema } from './lib/schemas.ts';
import SubdomainCreateModal from './modals/SubdomainCreateModal.tsx';
import SubdomainUpdateModal from './modals/SubdomainUpdateModal.tsx';
import { useExtTranslations } from './translations.ts';

function SubdomainRow({
  subdomain,
  serverUuid,
  onUpdated,
}: {
  subdomain: z.infer<typeof subdomainEntrySchema>;
  serverUuid: string;
  onUpdated: () => void;
}) {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();
  const { addToast } = useToast();

  const [openModal, setOpenModal] = useState<'update' | 'delete' | null>(null);

  const doDelete = async () => {
    await deleteSubdomain(serverUuid, subdomain.uuid)
      .then(() => {
        addToast(tExt('pages.server.subdomains.toast.deleted', {}), 'success');
        setOpenModal(null);
        onUpdated();
      })
      .catch((err) => {
        addToast(httpErrorToHuman(err), 'error');
      });
  };

  return (
    <>
      <SubdomainUpdateModal
        opened={openModal === 'update'}
        onClose={() => setOpenModal(null)}
        serverUuid={serverUuid}
        subdomain={subdomain}
        onSuccess={() => {
          setOpenModal(null);
          onUpdated();
        }}
      />
      <ConfirmationModal
        title={tExt('pages.server.subdomains.modal.delete.title', {})}
        opened={openModal === 'delete'}
        onClose={() => setOpenModal(null)}
        onConfirmed={doDelete}
      >
        {tExt('pages.server.subdomains.modal.delete.content', {
          subdomain: subdomain.subdomain,
          domain: subdomain.domain.domain,
        }).md()}
      </ConfirmationModal>

      <ContextMenu
        items={[
          {
            type: 'action',
            icon: faPencil,
            label: t('common.button.edit', {}),
            onClick: () => setOpenModal('update'),
            color: 'gray',
            canAccess: useServerCan('subdomains.update'),
          },
          {
            type: 'action',
            icon: faTrash,
            label: t('common.button.delete', {}),
            onClick: () => setOpenModal('delete'),
            color: 'red',
            canAccess: useServerCan('subdomains.delete'),
          },
        ]}
      >
        {({ items, openMenu }) => (
          <TableRow
            onContextMenu={(e) => {
              e.preventDefault();
              openMenu(e.pageX, e.pageY);
            }}
          >
            <TableData>
              <Code>
                {subdomain.subdomain}.{subdomain.domain.domain}
              </Code>
            </TableData>
            <TableData>
              {subdomain.allocation ? (
                <Code>{formatAllocation(subdomain.allocation)}</Code>
              ) : (
                <>{t('common.server.noAllocation', {})}</>
              )}
            </TableData>
            <TableData>
              <FormattedTimestamp timestamp={subdomain.created} />
            </TableData>
            <ContextMenuToggle items={items} openMenu={openMenu} />
          </TableRow>
        )}
      </ContextMenu>
    </>
  );
}

export default function SubdomainManagerPage() {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();
  const server = useServerStore((s) => s.server);
  const uuid = server.uuid;
  const subdomainLimit = serverWithSubdomainsFeatureLimitsSchema.safeParse(server).data?.featureLimits.subdomains ?? 0;

  const [createOpened, setCreateOpened] = useState(false);
  const [subdomains, setSubdomains] = useState<Pagination<z.infer<typeof subdomainEntrySchema>>>();

  const { loading, search, setSearch, setPage, refetch } = useSearchablePaginatedTable({
    queryKey: ['extension', 'subdomainmanager', 'subdomains', uuid],
    fetcher: (page, search) => getSubdomains(uuid, page, search || undefined),
    setStoreData: setSubdomains,
    modifyParams: false,
  });

  const { data: domains = [] } = useResource({
    queryKey: ['extension', 'subdomainmanager', 'server-domains', uuid],
    queryFn: () => getServerDomains(uuid),
  });

  const atLimit = (subdomains?.total ?? 0) >= subdomainLimit;
  const noDomains = domains.length === 0;

  return (
    <ServerContentContainer
      title={tExt('pages.server.subdomains.title', {})}
      subtitle={tExt('pages.server.subdomains.subtitle', { current: subdomains?.total ?? 0, max: subdomainLimit })}
      search={search}
      setSearch={setSearch}
      contentRight={
        <ServerCan action='subdomains.create'>
          <ConditionalTooltip
            enabled={atLimit || noDomains}
            label={
              noDomains
                ? tExt('pages.server.subdomains.tooltip.noDomains', {})
                : tExt('pages.server.subdomains.tooltip.limitReached', { max: subdomainLimit })
            }
          >
            <Button
              leftSection={<FontAwesomeIcon icon={faPlus} />}
              onClick={() => setCreateOpened(true)}
              disabled={atLimit || noDomains}
            >
              {t('common.button.create', {})}
            </Button>
          </ConditionalTooltip>
        </ServerCan>
      }
    >
      <SubdomainCreateModal
        opened={createOpened}
        onClose={() => setCreateOpened(false)}
        serverUuid={uuid}
        domains={domains}
        onSuccess={refetch}
      />

      <Table
        columns={[
          tExt('pages.server.subdomains.table.columns.domain', {}),
          tExt('common.form.allocation', {}),
          t('common.table.columns.created', {}),
          '',
        ]}
        loading={loading}
        pagination={subdomains}
        onPageSelect={setPage}
      >
        {subdomains?.data.map((sub) => (
          <SubdomainRow key={sub.uuid} subdomain={sub} serverUuid={uuid} onUpdated={refetch} />
        ))}
      </Table>
    </ServerContentContainer>
  );
}
