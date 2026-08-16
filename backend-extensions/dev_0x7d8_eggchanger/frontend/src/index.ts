import { Extension, ExtensionContext } from 'shared';
import Dev0x7d8EggChangerConfigurationPage from './ConfigurationPage.tsx';
import EggChangerContainer from './EggChangerContainer.tsx';

class EggChangerExtension extends Extension {
  public cardConfigurationPage: React.FC | null = Dev0x7d8EggChangerConfigurationPage;
  public cardComponent: React.FC | null = null;

  public initialize(ctx: ExtensionContext): void {
    ctx.extensionRegistry.pages.server.settings.enterSettingContainers((containers) =>
      containers.appendComponent(EggChangerContainer),
    );
  }
}

export default new EggChangerExtension();
