import { defineTranslations } from 'shared';

const translations = defineTranslations({
  items: {},
  translations: {
    common: {
      form: {
        allocation: 'Allocation',
        allocationGroupPrimary: 'Primary',
        allocationGroupOther: 'Other',
      },
    },
    pages: {
      server: {
        subdomains: {
          title: 'Subdomains',
          subtitle: '{current} of {max} subdomains created.',
          toast: {
            created: 'Subdomain created.',
            updated: 'Subdomain updated.',
            deleted: 'Subdomain deleted.',
          },
          tooltip: {
            limitReached: 'This server is limited to {max} subdomains.',
            noDomains: 'No domains are configured for this egg.',
          },
          table: {
            columns: {
              domain: 'Domain',
            },
          },
          modal: {
            create: { title: 'Create Subdomain' },
            update: { title: 'Update Subdomain' },
            delete: {
              title: 'Delete Subdomain',
              content:
                'Are you sure you want to delete **{subdomain}.{domain}**? The DNS records will also be removed.',
            },
          },
          form: {
            subdomain: 'Subdomain',
            domain: 'Domain',
          },
        },
      },
      admin: {
        servers: {
          featureLimits: {
            subdomains: 'Subdomains',
          },
        },
        configuration: {
          domains: {
            title: 'Domains',
            empty: 'No domains configured.',
            button: {
              add: 'Add Domain',
              test: 'Test',
              edit: 'Edit',
              viewSubdomains: 'View Subdomains',
            },
            toast: {
              created: 'Domain created.',
              updated: 'Domain updated.',
              deleted: 'Domain deleted.',
              testOk: 'Provider credentials are valid and have write access.',
            },
            table: {
              columns: {
                domain: 'Domain',
                provider: 'Provider',
                eggs: 'Eggs',
              },
            },
            tooltip: {
              test: 'Test credentials',
              edit: 'Edit domain',
              delete: 'Delete domain',
            },
            subdomains: {
              title: 'Subdomains - {domain}',
              empty: 'No subdomains for this domain.',
              table: {
                columns: {
                  subdomain: 'Subdomain',
                  server: 'Server',
                },
              },
            },
            modal: {
              create: { title: 'Create Domain' },
              edit: { title: 'Edit Domain' },
              delete: {
                title: 'Delete Domain',
                content:
                  'Are you sure you want to delete **{domain}**? All associated server subdomains will also be deleted.',
              },
            },
            form: {
              provider: 'Provider',
              eggs: 'Eligible Eggs',
              domain: 'Domain',
              regexes: 'Disallowed Subdomain Patterns',
              regexesDescription: 'Regex patterns that subdomains must not match.',
              regexesPlaceholder: '^admin$',
              cloudflareToken: 'API Token',
              cloudflareTokenDescription: 'Must have dns_records:write permission on the zone.',
              cloudflareZoneId: 'Zone ID',
              cloudflareZoneIdDescription: 'Found in the Cloudflare dashboard overview for your domain.',
              apiDataTemplate: 'DNS Record Template',
              apiDataTemplateDescription:
                'JSON template for the Cloudflare batch DNS API. Available placeholders: {placeholders}.',
            },
          },
          thankYou: {
            title: 'Thank you!',
            description:
              'Thank you for installing the Subdomain Manager extension! If you have any questions or issues, please reach out on Discord.',
            alert: {
              newVersion: 'A new version of the Subdomain Manager extension is available!',
            },
            table: {
              columns: {
                version: 'Version',
                changelog: 'Changelog',
              },
            },
            button: {
              joinDiscord: 'Join our Discord',
              downloadFrom: 'Download {version} from {provider}',
            },
          },
        },
      },
    },
  },
});

export const useExtTranslations = translations.useTranslations.bind(translations);
export const getExtTranslations = translations.getTranslations.bind(translations);

export default translations;
