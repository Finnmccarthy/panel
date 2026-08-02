import { faChevronDown, faCode, faServer } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { useState } from 'react';
import Button from '@/elements/Button.tsx';
import { ServerCan } from '@/elements/Can.tsx';
import ContextMenu from '@/elements/ContextMenu.tsx';
import { openUrl } from '@/lib/url.ts';
import { useServerCan } from '@/plugins/usePermissions.ts';
import { useAuth } from '@/providers/AuthProvider.tsx';
import { useFileManager } from '@/providers/FileManagerProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import { useServerStore } from '@/stores/server.ts';
import SftpDetailsModal from './modals/SftpDetailsModal.tsx';

export default function FileConnectButton({ file }: { file?: string }) {
  const { t } = useTranslations();
  const { user } = useAuth();
  const server = useServerStore((state) => state.server);
  const vscodeUriScheme = useFileManager((state) => state.vscodeUriScheme);
  const canSftp = useServerCan('files.sftp');
  const [sftpDetailsOpen, setSftpDetailsOpen] = useState(false);

  const sftpUrl = `sftp://${user!.username}.${server.uuidShort}@${server.sftpHost}:${server.sftpPort}`;

  const vscodeUrl =
    `${vscodeUriScheme}://calagopus.calagopus/open?origin=${encodeURIComponent(window.location.origin)}&server=${server.uuid}&create_path=/account/api-keys/create&console=1` +
    (file ? `&file=${encodeURIComponent(file)}` : '');

  return (
    <>
      <ServerCan action='files.sftp'>
        <SftpDetailsModal opened={sftpDetailsOpen} onClose={() => setSftpDetailsOpen(false)} />
      </ServerCan>
      <ServerCan action='files.read-content'>
        <ContextMenu
          menuProps={{ position: 'bottom-start' }}
          items={[
            {
              type: 'action',
              icon: faServer,
              label: t('pages.server.files.button.connectSftp', {}),
              onClick: (e) => {
                if (e.shiftKey) {
                  openUrl(sftpUrl);
                } else {
                  setSftpDetailsOpen(true);
                }
              },
              color: 'gray',
              canAccess: canSftp,
            },
            {
              type: 'action',
              icon: faCode,
              label: t('pages.server.files.button.connectVscode', {}),
              onClick: () => openUrl(vscodeUrl),
              color: 'gray',
            },
          ]}
        >
          {({ openMenu }) => (
            <Button
              variant='outline'
              leftSection={<FontAwesomeIcon icon={faServer} />}
              rightSection={<FontAwesomeIcon icon={faChevronDown} />}
              onClick={(e) => {
                e.stopPropagation();
                const rect = e.currentTarget.getBoundingClientRect();
                openMenu(rect.left, rect.bottom);
              }}
            >
              {t('pages.server.files.button.connect', {})}
            </Button>
          )}
        </ContextMenu>
      </ServerCan>
    </>
  );
}
