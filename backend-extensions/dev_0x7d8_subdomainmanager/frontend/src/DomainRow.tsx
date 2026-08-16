import { faList, faPencil, faTrash, faVial } from '@fortawesome/free-solid-svg-icons';
import { useState } from 'react';
import { z } from 'zod';
import { httpErrorToHuman } from '@/api/axios.ts';
import Code from '@/elements/Code.tsx';
import ContextMenu, { ContextMenuToggle } from '@/elements/ContextMenu.tsx';
import ConfirmationModal from '@/elements/modals/ConfirmationModal.tsx';
import { TableData, TableRow } from '@/elements/Table.tsx';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import deleteDomain from './api/domains/deleteDomain.ts';
import testDomain from './api/domains/testDomain.ts';
import { providerLabelMapping } from './lib/enums.ts';
import { domainSchema } from './lib/schemas.ts';
import DomainFormModal from './modals/DomainFormModal.tsx';
import { useExtTranslations } from './translations.ts';

type EggGroup = { group: string; items: { label: string; value: string }[] };

export default function DomainRow({
  domain,
  eggGroups,
  onUpdated,
  onViewSubdomains,
}: {
  domain: z.infer<typeof domainSchema>;
  eggGroups: EggGroup[];
  onUpdated: () => void;
  onViewSubdomains: () => void;
}) {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();
  const { addToast } = useToast();

  const [openModal, setOpenModal] = useState<'edit' | 'delete' | null>(null);
  const [testLoading, setTestLoading] = useState(false);

  const doDelete = async () => {
    await deleteDomain(domain.uuid)
      .then(() => {
        addToast(tExt('pages.admin.configuration.domains.toast.deleted', {}), 'success');
        setOpenModal(null);
        onUpdated();
      })
      .catch((err) => addToast(httpErrorToHuman(err), 'error'));
  };

  const doTest = () => {
    setTestLoading(true);
    testDomain(domain.uuid)
      .then(() => addToast(tExt('pages.admin.configuration.domains.toast.testOk', {}), 'success'))
      .catch((err) => addToast(httpErrorToHuman(err), 'error'))
      .finally(() => setTestLoading(false));
  };

  return (
    <>
      <DomainFormModal
        opened={openModal === 'edit'}
        onClose={() => setOpenModal(null)}
        domain={domain}
        eggGroups={eggGroups}
        onSaved={onUpdated}
      />
      <ConfirmationModal
        title={tExt('pages.admin.configuration.domains.modal.delete.title', {})}
        opened={openModal === 'delete'}
        onClose={() => setOpenModal(null)}
        onConfirmed={doDelete}
      >
        {tExt('pages.admin.configuration.domains.modal.delete.content', { domain: domain.domain }).md()}
      </ConfirmationModal>

      <ContextMenu
        items={[
          {
            type: 'action',
            icon: faVial,
            label: tExt('pages.admin.configuration.domains.button.test', {}),
            onClick: doTest,
            color: 'gray',
            hidden: testLoading,
          },
          {
            type: 'action',
            icon: faList,
            label: tExt('pages.admin.configuration.domains.button.viewSubdomains', {}),
            onClick: onViewSubdomains,
            color: 'gray',
          },
          {
            type: 'action',
            icon: faPencil,
            label: t('common.button.edit', {}),
            onClick: () => setOpenModal('edit'),
            color: 'gray',
          },
          {
            type: 'action',
            icon: faTrash,
            label: t('common.button.delete', {}),
            onClick: () => setOpenModal('delete'),
            color: 'red',
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
              <Code>{domain.domain}</Code>
            </TableData>
            <TableData>{providerLabelMapping[domain.provider] ?? domain.provider}</TableData>
            <TableData>{domain.eggs.length}</TableData>
            <ContextMenuToggle items={items} openMenu={openMenu} />
          </TableRow>
        )}
      </ContextMenu>
    </>
  );
}
