import { z } from 'zod';
import { axiosInstance } from '@/api/axios';
import { parseFromApi } from '@/lib/api-transform.ts';
import { minecraftBuildSchema } from './getVersionsForType.ts';

const installedBuildSchema = z.object({
  build: minecraftBuildSchema,
  latest: minecraftBuildSchema,
});

export default async (uuid: string): Promise<z.infer<typeof installedBuildSchema> | null> => {
  const { data } = await axiosInstance.get(`/api/client/servers/${uuid}/minecraft/versions/installed`);
  return data.build ? parseFromApi(installedBuildSchema, data.build) : null;
};
