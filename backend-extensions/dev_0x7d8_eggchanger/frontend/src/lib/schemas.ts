import { z } from 'zod';

export const adminExtensionSettingsEggGroup = z.object({
  name: z.string().min(1).max(255),
  nameTranslations: z.record(z.string(), z.string().min(1).max(255)),
  eggs: z.array(z.uuid()),

  forceUpdateStartup: z.boolean(),
  forceReinstall: z.boolean(),
  forceReinstallTruncateFiles: z.boolean(),

  reassignAllocations: z.boolean(),

  affectedEggs: z.array(z.uuid()),
});

export const adminExtensionSettingsSchema = z.object({
  eggGroups: z.array(adminExtensionSettingsEggGroup),
});

export const serverSettingsChangeEggSchema = z.object({
  eggUuid: z.uuid(),

  updateStartup: z.boolean(),
  reinstall: z.boolean(),
  truncateDirectory: z.boolean(),
});
