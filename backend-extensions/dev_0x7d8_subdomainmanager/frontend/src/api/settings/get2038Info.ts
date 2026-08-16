import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';

const extensionInfoSchema = z.object({
  product: z.object({
    version: z.string(),
  }),
  providers: z.record(z.string(), z.object({ name: z.string(), link: z.string() })),
  changelogs: z.record(z.string(), z.object({ content: z.string(), created: z.coerce.date() })),
});

const response2038Schema = z.object({
  extension: extensionInfoSchema,
  version: z.string(),
});

export type ExtensionInfo = z.infer<typeof extensionInfoSchema> & { currentVersion: string };

export default async (): Promise<ExtensionInfo> => {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.0x7d8.subdomainmanager/2038');
  const { extension, version } = parseFromApi(response2038Schema, data);
  return { ...extension, currentVersion: version };
};
