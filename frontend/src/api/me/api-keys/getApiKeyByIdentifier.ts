import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { userApiKeySchema } from '@/lib/schemas/user/apiKeys.ts';

export default async (identifier: string): Promise<z.infer<typeof userApiKeySchema>> => {
  const { data } = await axiosInstance.get(`/api/client/account/api-keys/identifier/${identifier}`);
  return parseFromApi(userApiKeySchema, data.api_key);
};
