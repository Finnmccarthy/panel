import { axiosInstance } from '@/api/axios.ts';

export default async (uuid: string, path: string, destination: string | null, overwrite = false): Promise<string> => {
  const { data } = await axiosInstance.post(`/api/client/servers/${uuid}/files/copy`, {
    path,
    destination,
    overwrite,
  });
  return data.identifier;
};
