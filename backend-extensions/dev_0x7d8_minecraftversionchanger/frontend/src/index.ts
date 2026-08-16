import { faTag } from '@fortawesome/free-solid-svg-icons';
import { Extension, ExtensionContext } from 'shared';
import Dev0x7d8MinecraftVersionChangerConfigurationPage from './ConfigurationPage.tsx';
import MinecraftVersionsPage from './MinecraftVersionsPage.tsx';
import { getExtTranslations } from './translations.ts';

class MinecraftVersionsExtension extends Extension {
  public cardConfigurationPage: React.FC | null = Dev0x7d8MinecraftVersionChangerConfigurationPage;
  public cardComponent: React.FC | null = null;

  public initialize(ctx: ExtensionContext): void {
    ctx.extensionRegistry.enterRoutes((routes) =>
      routes.addServerRoute({
        name: () => getExtTranslations().t('pages.server.versions.title', {}),
        icon: faTag,
        path: '/minecraft/versions',
        element: MinecraftVersionsPage,
      }),
    );
  }
}

export default new MinecraftVersionsExtension();
