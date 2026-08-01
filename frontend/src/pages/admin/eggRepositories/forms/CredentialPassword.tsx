import { UseFormReturnType } from '@mantine/form';
import { useEffect } from 'react';
import { z } from 'zod';
import { type FieldDef, FormEngine } from '@/elements/form-engine/index.ts';
import PasswordInput from '@/elements/input/PasswordInput.tsx';
import TextInput from '@/elements/input/TextInput.tsx';
import { adminEggRepositoryCredentialsPasswordSchema } from '@/lib/schemas/admin/eggRepositories.ts';
import { useTranslations } from '@/providers/TranslationProvider.tsx';

type CredentialsForm = UseFormReturnType<{
  credentials: z.infer<typeof adminEggRepositoryCredentialsPasswordSchema>;
}>;

export default function CredentialPassword({ form }: { form: CredentialsForm }) {
  const { t } = useTranslations();

  useEffect(() => {
    form.setValues({
      credentials: {
        type: 'password',
        username: form.values.credentials.username ?? '',
        password: form.values.credentials.password ?? '',
      },
    });
  }, []);

  const fields: FieldDef<{ credentials: z.infer<typeof adminEggRepositoryCredentialsPasswordSchema> }>[] = [
    {
      type: 'custom',
      name: 'username',
      render: (f) => (
        <TextInput
          withAsterisk
          label={t('common.form.username', {})}
          key={f.key('credentials.username')}
          {...f.getInputProps('credentials.username')}
        />
      ),
    },
    {
      type: 'custom',
      name: 'password',
      render: (f) => (
        <PasswordInput
          withAsterisk
          label={t('pages.admin.eggRepositories.tabs.general.page.form.password', {})}
          key={f.key('credentials.password')}
          {...f.getInputProps('credentials.password')}
        />
      ),
    },
  ];

  return <FormEngine id='admin.eggRepositories.credentialPassword' form={form} fields={fields} className='mt-4' />;
}
