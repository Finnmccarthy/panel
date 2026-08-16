import { axiosInstance } from '@/api/axios';

interface Data {
  buildUuid: string;
  truncateDirectory: boolean;
  acceptEula: boolean;
}

export default async (uuid: string, data: Data): Promise<void> => {
  await axiosInstance.post(`/api/client/servers/${uuid}/minecraft/versions/install`, {
    build_uuid: data.buildUuid,
    truncate_directory: data.truncateDirectory,
    accept_eula: data.acceptEula,
  });
};
