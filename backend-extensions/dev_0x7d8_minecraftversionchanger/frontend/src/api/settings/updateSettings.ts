import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { transformKeysToSnakeCase } from '@/lib/transformers.ts';
import { adminExtensionSettingsSchema } from '../../lib/schemas.ts';

export default async (data: z.infer<typeof adminExtensionSettingsSchema>): Promise<void> => {
  return new Promise((resolve, reject) => {
    axiosInstance
      .put('/api/admin/extensions/dev.0x7d8.minecraftversionchanger/settings', {
        ...transformKeysToSnakeCase(data),
        mcjars_type_order: data.mcjarsTypeOrder,
      })
      .then(() => resolve())
      .catch(reject);
  });
};
