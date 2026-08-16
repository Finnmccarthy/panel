import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { adminExtensionSettingsSchema } from '../../lib/schemas.ts';

export default async (): Promise<z.infer<typeof adminExtensionSettingsSchema>> => {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.0x7d8.eggchanger/settings');
  return parseFromApi(adminExtensionSettingsSchema, data.settings);
};
