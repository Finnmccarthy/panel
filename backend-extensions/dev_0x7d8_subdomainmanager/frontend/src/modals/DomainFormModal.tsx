import { Divider, ModalProps, Stack } from '@mantine/core';
import { UseFormReturnType, useForm } from '@mantine/form';
import { zod4Resolver } from 'mantine-form-zod-resolver';
import { useEffect, useState } from 'react';
import { z } from 'zod';
import { httpErrorToHuman } from '@/api/axios.ts';
import Button from '@/elements/Button.tsx';
import MultiSelectGroup from '@/elements/input/MultiSelectGroup.tsx';
import PasswordInput from '@/elements/input/PasswordInput.tsx';
import Select from '@/elements/input/Select.tsx';
import TagsInput from '@/elements/input/TagsInput.tsx';
import TextArea from '@/elements/input/TextArea.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { Modal, ModalFooter } from '@/elements/modals/Modal.tsx';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import createDomain from '../api/domains/createDomain.ts';
import updateDomain from '../api/domains/updateDomain.ts';
import { providerLabelMapping } from '../lib/enums.ts';
import { domainCreateUpdateFormSchema, domainFormSchema, domainSchema } from '../lib/schemas.ts';
import { useExtTranslations } from '../translations.ts';

type EggGroup = { group: string; items: { label: string; value: string }[] };

const DEFAULT_API_DATA = JSON.stringify(
  {
    posts: [
      {
        type: 'A',
        name: '{subdomain}.{domain}',
        content: '{ip_alias_forceip}',
        comment: '{comment}',
        ttl: 120,
      },
      {
        type: 'SRV',
        name: '_minecraft._tcp.{subdomain}.{domain}',
        data: {
          priority: 0,
          weight: 5,
          port: '{port}',
          target: '{subdomain}.{domain}',
        },
        comment: '{comment}',
        ttl: 120,
      },
    ],
  },
  null,
  2,
);

function fromDomain(domain?: z.infer<typeof domainSchema>): z.infer<typeof domainCreateUpdateFormSchema> {
  const cfg = (domain?.providerConfig ?? {}) as Record<string, unknown>;
  return {
    provider: 'cloudflare',
    eggs: domain?.eggs ?? [],
    domain: domain?.domain ?? '',
    disallowedSubdomainsRegexes: domain?.disallowedSubdomainsRegexes ?? [],
    cloudflare_token: (cfg.token as string) ?? '',
    cloudflare_zone_id: (cfg.zone_id as string) ?? '',
    cloudflare_api_data: JSON.stringify(cfg.api_data ?? JSON.parse(DEFAULT_API_DATA), null, 2),
  };
}

function toDomainForm(values: z.infer<typeof domainCreateUpdateFormSchema>): z.infer<typeof domainFormSchema> {
  return {
    provider: values.provider,
    eggs: values.eggs,
    domain: values.domain,
    disallowedSubdomainsRegexes: values.disallowedSubdomainsRegexes,
    providerConfig: {
      token: values.cloudflare_token,
      zone_id: values.cloudflare_zone_id,
      api_data: JSON.parse(values.cloudflare_api_data),
    },
  };
}

function CloudflareFields({ form }: { form: UseFormReturnType<z.infer<typeof domainCreateUpdateFormSchema>> }) {
  const { t } = useExtTranslations();

  return (
    <>
      <PasswordInput
        withAsterisk
        label={t('pages.admin.configuration.domains.form.cloudflareToken', {})}
        description={t('pages.admin.configuration.domains.form.cloudflareTokenDescription', {})}
        placeholder='cf_...'
        key={form.key('cloudflare_token')}
        {...form.getInputProps('cloudflare_token')}
      />
      <TextInput
        withAsterisk
        label={t('pages.admin.configuration.domains.form.cloudflareZoneId', {})}
        description={t('pages.admin.configuration.domains.form.cloudflareZoneIdDescription', {})}
        placeholder='a1b2c3d4...'
        key={form.key('cloudflare_zone_id')}
        {...form.getInputProps('cloudflare_zone_id')}
      />
      <TextArea
        withAsterisk
        label={t('pages.admin.configuration.domains.form.apiDataTemplate', {})}
        description={t('pages.admin.configuration.domains.form.apiDataTemplateDescription', {
          placeholders: 'subdomain, domain, ip, ip_alias, ip_alias_forceip, port, comment',
        })}
        rows={8}
        styles={{ input: { fontFamily: 'monospace', fontSize: 12 } }}
        key={form.key('cloudflare_api_data')}
        {...form.getInputProps('cloudflare_api_data')}
      />
    </>
  );
}

const providerFieldsMapping: Record<
  string,
  React.FC<{ form: UseFormReturnType<z.infer<typeof domainCreateUpdateFormSchema>> }>
> = {
  cloudflare: CloudflareFields,
};

interface Props extends ModalProps {
  domain?: z.infer<typeof domainSchema>;
  eggGroups: EggGroup[];
  onSaved: () => void;
}

export default function DomainFormModal({ domain, eggGroups, onSaved, ...modalProps }: Props) {
  const { t } = useTranslations();
  const { t: tExt } = useExtTranslations();
  const { addToast } = useToast();
  const [loading, setLoading] = useState(false);

  const form = useForm<z.infer<typeof domainCreateUpdateFormSchema>>({
    initialValues: fromDomain(),
    validateInputOnBlur: true,
    validate: zod4Resolver(domainCreateUpdateFormSchema),
  });

  useEffect(() => {
    if (modalProps.opened) {
      form.setValues(fromDomain(domain));
      form.resetDirty();
    }
  }, [domain, modalProps.opened]);

  const doSubmit = () => {
    setLoading(true);

    const payload = toDomainForm(form.values);
    const promise = domain ? updateDomain(domain.uuid, payload) : createDomain(payload);

    promise
      .then(() => {
        addToast(
          tExt(
            domain
              ? 'pages.admin.configuration.domains.toast.updated'
              : 'pages.admin.configuration.domains.toast.created',
            {},
          ),
          'success',
        );
        onSaved();
        modalProps.onClose();
      })
      .catch((err) => addToast(httpErrorToHuman(err), 'error'))
      .finally(() => setLoading(false));
  };

  const providerLabel = providerLabelMapping[form.values.provider] ?? form.values.provider;
  const ProviderFields = providerFieldsMapping[form.values.provider] ?? null;

  return (
    <Modal
      title={tExt(
        domain
          ? 'pages.admin.configuration.domains.modal.edit.title'
          : 'pages.admin.configuration.domains.modal.create.title',
        {},
      )}
      size='lg'
      {...modalProps}
    >
      <form onSubmit={form.onSubmit(() => doSubmit())}>
        <Stack>
          <Select
            withAsterisk
            label={tExt('pages.admin.configuration.domains.form.provider', {})}
            data={Object.entries(providerLabelMapping).map(([value, label]) => ({ value, label }))}
            key={form.key('provider')}
            {...form.getInputProps('provider')}
          />

          <TextInput
            withAsterisk
            label={tExt('pages.admin.configuration.domains.form.domain', {})}
            placeholder='example.com'
            key={form.key('domain')}
            {...form.getInputProps('domain')}
          />

          <MultiSelectGroup
            label={tExt('pages.admin.configuration.domains.form.eggs', {})}
            placeholder='Select eggs'
            data={eggGroups}
            searchable
            key={form.key('eggs')}
            {...form.getInputProps('eggs')}
          />

          <TagsInput
            label={tExt('pages.admin.configuration.domains.form.regexes', {})}
            description={tExt('pages.admin.configuration.domains.form.regexesDescription', {})}
            placeholder={tExt('pages.admin.configuration.domains.form.regexesPlaceholder', {})}
            key={form.key('disallowedSubdomainsRegexes')}
            {...form.getInputProps('disallowedSubdomainsRegexes')}
          />

          {ProviderFields && (
            <>
              <Divider label={providerLabel} labelPosition='left' />
              <ProviderFields form={form} />
            </>
          )}
        </Stack>

        <ModalFooter>
          <Button type='submit' loading={loading} disabled={!form.isValid()}>
            {domain ? t('common.button.save', {}) : t('common.button.create', {})}
          </Button>
          <Button variant='default' onClick={modalProps.onClose}>
            {t('common.button.close', {})}
          </Button>
        </ModalFooter>
      </form>
    </Modal>
  );
}
