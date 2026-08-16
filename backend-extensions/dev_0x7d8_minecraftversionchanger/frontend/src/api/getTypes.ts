import { z } from 'zod';
import { axiosInstance } from '@/api/axios';
import { parseFromApi } from '@/lib/api-transform.ts';

const minecraftTypeSchema = z.object({
  name: z.string(),
  icon: z.string(),
  color: z.string(),
  homepage: z.string(),
  description: z.string(),
  experimental: z.boolean(),
  deprecated: z.boolean(),
  builds: z.number(),
  versions: z.object({
    minecraft: z.number(),
    project: z.number(),
  }),
});

// Both record levels are keyed by identifiers from the mcjars API, so their keys pass through verbatim
const responseSchema = z.object({
  types: z.record(z.string(), z.record(z.string(), minecraftTypeSchema)),
});

// `identifier` is not part of the response body - it is the key each type is stored under
export type MinecraftVersionProviderType = z.infer<typeof minecraftTypeSchema> & { identifier: string };

export default async (uuid: string): Promise<Record<string, MinecraftVersionProviderType[]>> => {
  const { data } = await axiosInstance.get(`/api/client/servers/${uuid}/minecraft/versions/types`);
  const { types } = parseFromApi(responseSchema, data);
  return Object.fromEntries(
    Object.entries(types).map(([group, groupTypes]) => [
      group,
      Object.entries(groupTypes).map(([identifier, type]) => ({ ...type, identifier })),
    ]),
  );
};
