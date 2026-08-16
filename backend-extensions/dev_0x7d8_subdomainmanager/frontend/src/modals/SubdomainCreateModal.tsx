import { ModalProps, Stack } from '@mantine/core';
import { useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { z } from 'zod';
import { httpErrorToHuman } from '@/api/axios.ts';
import getAllocations from '@/api/server/allocations/getAllocations.ts';
import Button from '@/elements/Button.tsx';
import Select from '@/elements/input/Select.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { serverAllocationSchema } from '@/lib/schemas/server/allocations.ts';
import { formatAllocation } from '@/lib/server.ts';
import { useResource } from '@/plugins/useResource.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import createSubdomain from '../api/createSubdomain.ts';
import { subdomainFormSchema } from '../lib/schemas.ts';
import { useExtTranslations } from '../translations.ts';

interface Props extends ModalProps {
  serverUuid: string;
  domains: { uuid: string; domain: string }[];
  onSuccess: () => void;
}

function buildAllocationData(
  allocations: z.infer<typeof serverAllocationSchema>[],
  labelPrimary: string,
  labelOther: string,
) {
  const primary = allocations.filter((a) => a.isPrimary);
  const others = allocations.filter((a) => !a.isPrimary);

  if (primary.length === 0) {
    return allocations.map((a) => ({ value: a.uuid, label: formatAllocation(a) }));
  }

  return [
    { group: labelPrimary, items: primary.map((a) => ({ value: a.uuid, label: formatAllocation(a) })) },
    ...(others.length > 0
      ? [{ group: labelOther, items: others.map((a) => ({ value: a.uuid, label: formatAllocation(a) })) }]
      : []),
  ];
}

export default function SubdomainCreateModal({ serverUuid, domains, onSuccess, ...modalProps }: Props) {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();
  const { addToast } = useToast();
  const [loading, setLoading] = useState(false);

  const { data: allocationsPage } = useResource({
    queryKey: ['server', serverUuid, 'allocations'],
    queryFn: () => getAllocations(serverUuid, 1),
    enabled: modalProps.opened,
  });
  const allocations = allocationsPage?.data ?? [];

  const form = useForm<z.infer<typeof subdomainFormSchema>>({
    initialValues: { subdomain: '', domainUuid: domains[0]?.uuid ?? '', allocationUuid: '' },
    validateInputOnBlur: true,
    validate: zod4Resolver(subdomainFormSchema),
  });

  useEffect(() => {
    if (!modalProps.opened) return;
    const primaryAlloc = allocations.find((a) => a.isPrimary);
    form.setValues({
      subdomain: '',
      domainUuid: domains[0]?.uuid ?? '',
      allocationUuid: primaryAlloc?.uuid ?? '',
    });
  }, [modalProps.opened, allocations]);

  const doSubmit = () => {
    setLoading(true);
    createSubdomain(serverUuid, form.values.subdomain, form.values.domainUuid, form.values.allocationUuid)
      .then(() => {
        addToast(tExt('pages.server.subdomains.toast.created', {}), 'success');
        onSuccess();
        modalProps.onClose();
      })
      .catch((err) => addToast(httpErrorToHuman(err), 'error'))
      .finally(() => setLoading(false));
  };

  return (
    <Modal title={tExt('pages.server.subdomains.modal.create.title', {})} {...modalProps}>
      <form onSubmit={form.onSubmit(() => doSubmit())}>
        <Stack>
          <div className='grid grid-cols-2 gap-4'>
            <TextInput
              withAsterisk
              label={tExt('pages.server.subdomains.form.subdomain', {})}
              placeholder='myserver'
              key={form.key('subdomain')}
              {...form.getInputProps('subdomain')}
            />
            <Select
              withAsterisk
              label={tExt('pages.server.subdomains.form.domain', {})}
              data={domains.map((domain) => ({ value: domain.uuid, label: domain.domain }))}
              key={form.key('domainUuid')}
              {...form.getInputProps('domainUuid')}
            />
          </div>

          <Select
            withAsterisk
            label={tExt('common.form.allocation', {})}
            data={buildAllocationData(
              allocations,
              tExt('common.form.allocationGroupPrimary', {}),
              tExt('common.form.allocationGroupOther', {}),
            )}
            key={form.key('allocationUuid')}
            {...form.getInputProps('allocationUuid')}
          />
        </Stack>

        <ModalFooter>
          <Button type='submit' loading={loading} disabled={!form.isValid()}>
            {t('common.button.create', {})}
          </Button>
          <Button variant='default' onClick={modalProps.onClose}>
            {t('common.button.close', {})}
          </Button>
        </ModalFooter>
      </form>
    </Modal>
  );
}
