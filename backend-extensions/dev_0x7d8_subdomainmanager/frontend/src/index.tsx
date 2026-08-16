import { faGlobe } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Extension, type ExtensionContext } from 'shared';
import { z } from 'zod';
import { insertFieldsAfter } from '@/elements/form-engine/index.ts';
import Dev0x7d8SubdomainManagerConfigurationPage from './ConfigurationPage.tsx';
import SubdomainManagerPage from './SubdomainManagerPage.tsx';
import { getExtTranslations } from './translations.ts';

class Dev0x7d8SubdomainManagerExtension extends Extension {
  public cardConfigurationPage: React.FC | null = Dev0x7d8SubdomainManagerConfigurationPage;
  public cardComponent: React.FC | null = null;

  public initialize(ctx: ExtensionContext): void {
    ctx.extensionRegistry.enterRoutes((routes) =>
      routes.addServerRoute({
        name: () => getExtTranslations().t('pages.server.subdomains.title', {}),
        icon: faGlobe,
        path: '/subdomains',
        element: SubdomainManagerPage,
      }),
    );

    ctx.extensionRegistry.enterForms((forms) => {
      for (const formId of ['admin.servers.create', 'admin.servers.update'] as const) {
        forms.extend(formId, {
          zodShape: { featureLimits: z.object({ subdomains: z.number().min(0) }) },
          initialValues: { featureLimits: { subdomains: 0 } },
          transform: (fields) =>
            insertFieldsAfter(fields, 'featureLimits.schedules', {
              type: 'number',
              name: 'featureLimits.subdomains',
              label: () => getExtTranslations().t('pages.admin.servers.featureLimits.subdomains', {}),
              required: true,
              props: { min: 0, placeholder: '0' },
            }),
        });
      }
    });

    ctx.extensionRegistry.permissionIcons.addServerPermissionIcon('subdomains', <FontAwesomeIcon icon={faGlobe} />);
  }
}

export default new Dev0x7d8SubdomainManagerExtension();
