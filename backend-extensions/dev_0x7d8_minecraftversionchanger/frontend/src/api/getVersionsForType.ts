import { z } from 'zod';
import { axiosInstance } from '@/api/axios';
import { parsePaginationFromApi } from '@/lib/api-transform.ts';

export const minecraftBuildSchema = z.object({
  uuid: z.string(),
  type: z.string(),
  experimental: z.boolean(),
  name: z.string(),
  versionId: z.string().nullable(),
  projectVersionId: z.string().nullable(),
  created: z.string().nullable(),
});

// `type` is proxied verbatim from the mcjars API, so it is not constrained to a known set here
export const minecraftVersionSchema = z.object({
  id: z.string(),
  type: z.string(),
  supported: z.boolean(),
  java: z.number(),
  builds: z.number(),
  created: z.string(),
  latest: minecraftBuildSchema,
});

export type MinecraftBuild = z.infer<typeof minecraftBuildSchema>;
export type MinecraftVersion = z.infer<typeof minecraftVersionSchema>;

export default async (
  uuid: string,
  type: string,
  page: number,
  search?: string,
): Promise<Pagination<MinecraftVersion>> => {
  const { data } = await axiosInstance.get(
    `/api/client/servers/${uuid}/minecraft/versions/types/${type.toUpperCase()}`,
    { params: { page, search } },
  );
  return parsePaginationFromApi(minecraftVersionSchema, data.versions);
};
