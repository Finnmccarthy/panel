import { z } from 'zod';
import { axiosInstance } from '@/api/axios';
import { parseFromApi } from '@/lib/api-transform.ts';
import { serverEggSchema } from '@/lib/schemas/server/server';

export const eggGroupSchema = z.object({
  name: z.string(),
  nameTranslations: z.record(z.string(), z.string()),
  eggs: z.array(serverEggSchema),
  forceUpdateStartup: z.boolean(),
  forceReinstall: z.boolean(),
  forceReinstallTruncateFiles: z.boolean(),
});

export type EggGroup = z.infer<typeof eggGroupSchema>;

export default async (uuid: string): Promise<EggGroup[]> => {
  const { data } = await axiosInstance.get(`/api/client/servers/${uuid}/settings/egg-changer/available`);
  return parseFromApi(z.array(eggGroupSchema), data.egg_groups);
};
