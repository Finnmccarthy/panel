import { faGlobe, faTimes } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { useState } from 'react';
import { z } from 'zod';
import { getEmptyPaginationSet } from '@/api/axios.ts';
import ActionIcon from '@/elements/ActionIcon.tsx';
import Code from '@/elements/Code.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import Table, { TableData, TableRow } from '@/elements/Table.tsx';
import TableLink from '@/elements/TableLink.tsx';
import TitleCard from '@/elements/TitleCard.tsx';
import FormattedTimestamp from '@/elements/time/FormattedTimestamp.tsx';
import { formatAllocation } from '@/lib/server.ts';
import { useSearchablePaginatedTable } from '@/plugins/useSearchablePaginatedTable.ts';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import getAdminDomainSubdomains from './api/domains/getAdminDomainSubdomains.ts';
import { adminSubdomainEntrySchema, domainSchema } from './lib/schemas.ts';
import { useExtTranslations } from './translations.ts';

function AdminSubdomainRow({
  subdomain,
  domainName,
}: {
  subdomain: z.infer<typeof adminSubdomainEntrySchema>;
  domainName: string;
}) {
  const { t } = useTranslations();

  return (
    <TableRow>
      <TableData>
        <Code>
          {subdomain.subdomain}.{domainName}
        </Code>
      </TableData>
      <TableData>
        {subdomain.server ? (
          <TableLink to={`/admin/servers/${subdomain.server.uuid}`}>
            <Code>{subdomain.server.name}</Code>
          </TableLink>
        ) : (
          <>{t('common.na', {})}</>
        )}
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
    </TableRow>
  );
}

export default function DomainSubdomainsPanel({
  domain,
  onClose,
}: {
  domain: z.infer<typeof domainSchema>;
  onClose: () => void;
}) {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();

  const [subdomains, setSubdomains] = useState<Pagination<z.infer<typeof adminSubdomainEntrySchema>>>(
    getEmptyPaginationSet(),
  );

  const { loading, search, setSearch, setPage } = useSearchablePaginatedTable({
    queryKey: ['extension', 'subdomainmanager', 'admin-domain-subdomains', domain.uuid],
    fetcher: (page, search) => getAdminDomainSubdomains(domain.uuid, page, search || undefined),
    setStoreData: setSubdomains,
    modifyParams: false,
  });

  return (
    <TitleCard
      title={tExt('pages.admin.configuration.domains.subdomains.title', { domain: domain.domain })}
      icon={<FontAwesomeIcon icon={faGlobe} />}
      className='w-full mt-4'
      rightSection={
        <div className='flex items-center gap-2 ml-auto'>
          <TextInput
            placeholder={t('common.input.search', {})}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            size='xs'
            w={200}
          />
          <ActionIcon variant='subtle' onClick={onClose}>
            <FontAwesomeIcon icon={faTimes} />
          </ActionIcon>
        </div>
      }
    >
      <Table
        columns={[
          tExt('pages.admin.configuration.domains.subdomains.table.columns.subdomain', {}),
          tExt('pages.admin.configuration.domains.subdomains.table.columns.server', {}),
          tExt('common.form.allocation', {}),
          t('common.table.columns.created', {}),
        ]}
        loading={loading}
        pagination={subdomains}
        onPageSelect={setPage}
      >
        {subdomains.data.map((subdomain) => (
          <AdminSubdomainRow key={subdomain.uuid} subdomain={subdomain} domainName={domain.domain} />
        ))}
      </Table>
    </TitleCard>
  );
}
