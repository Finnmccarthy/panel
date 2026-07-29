import { axiosInstance } from '@/api/axios.ts';

export default async (uuid: string, revisionId: number, file: string): Promise<string> => {
  const { data } = await axiosInstance.get(`/api/client/servers/${uuid}/files/revisions/${revisionId}`, {
    params: { file },
    responseType: 'text',
  });
  return data;
};
