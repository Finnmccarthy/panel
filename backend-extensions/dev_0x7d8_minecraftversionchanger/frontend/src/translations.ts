import { defineEnglishItem, defineTranslations } from 'shared';

const translations = defineTranslations({
  items: {
    version: defineEnglishItem('Version', 'Versions'),
    projectVersion: defineEnglishItem('Project version', 'Project versions'),
    build: defineEnglishItem('Build', 'Builds'),
  },
  translations: {
    elements: {
      typeOrderInput: {
        error: {
          empty: 'Key cannot be empty',
          duplicate: 'Key already exists',
        },
        tooltip: {
          removeEntry: 'Remove entry',
        },
        noTypes: 'No types',
        noEntries: 'No entries',
        selectTypes: 'Select types...',
        newKey: 'New key...',
      },
    },
    pages: {
      admin: {
        configuration: {
          settings: {
            title: 'Settings',
            toast: {
              updated: 'Settings updated.',
            },
            form: {
              mcjarsUrl: 'MCJars URL',
              mcjarsUrlDescription: 'The base URL for the MCJars API.',
              mcjarsApiKey: 'MCJars API Key',
              mcjarsApiKeyDescription: 'Your MCJars API key. You can get an API key from https://mcjars.app.',
              mcjarsIconBaseUrl: 'MCJars Icon Base URL',
              mcjarsIconBaseUrlDescription: 'The base URL for MCJars icons. For example: https://mcjars.app/icons/',
              mcjarsIconFileExtension: 'MCJars Icon File Extension',
              mcjarsIconFileExtensionDescription:
                'The file extension for MCJars icons, excluding the dot. For example: png',
              mcjarsTypeOrder: 'Override MCJars Version Type Order',
              collectInstallationStatistics: 'Collect Installation Statistics',
              collectInstallationStatisticsDescription:
                "Make the extension collect local installation statistics. This data is only collected locally and is not sent to any server. It is used to provide you with insights about Users' Minecraft installations.",
            },
          },
          thankYou: {
            title: 'Thank you!',
            description:
              'Thank you for installing the Minecraft Version Changer extension! If you have any questions, feedback or issues, please feel free to reach out to us on our Discord server.',
            alert: {
              newVersion:
                'A new version of the Minecraft Version Changer extension is available! Please update to the latest version to get the best experience.',
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
          typeEggAssignments: {
            title: 'Type Egg Assignments',
            description:
              "When a user installs a version of one of the selected types, the server's egg will automatically be changed to the configured egg.",
            noEntries: 'No assignments configured.',
            button: {
              addAssignment: 'Add Assignment',
            },
            form: {
              affectedTypes: 'Affected Types',
              affectedTypesPlaceholder: 'Select types...',
              nest: 'Nest',
              egg: 'Egg',
            },
            tooltip: {
              removeAssignment: 'Remove Assignment',
            },
          },
          defaultTypeOrder: {
            title: 'Default Version Type Order',
            description:
              'This is the default order of version types as defined by MCJars. It is used to determine the order of version types in the version list.',
          },
          statistics: {
            title: 'Installation Statistics',
            description: 'This section provides insights about the Minecraft installations of your users.',
            descriptionAmount: 'Total installations: {totalInstallations}',
            byType: 'By Type',
            byVersion: 'By Version',
            noStatistics: 'No statistics available.',
          },
        },
      },
      server: {
        versions: {
          title: 'Versions',
          toast: {
            installed: 'Version installed successfully.',
          },
          tooltip: {
            experimental: 'Experimental',
            deprecated: 'Deprecated',
          },
          button: {
            viewVersions: 'View Versions',
          },
          alert: {
            currentlyRunning: 'Currently running {type}',
            installedVersion: 'Installed version: {version}',
            installedProjectVersion: 'Installed project version: {projectVersion}',
            installedBuild: 'Installed build: {build}',
            outdatedVersion:
              'Your server is currently running an outdated version of {type} {version}. The latest build is {build}.',
          },
          modal: {
            install: {
              title: 'Install {type} ({version})',
              alert: {
                experimental:
                  'This version is experimental and may contain bugs or other issues. Please make sure to create a backup of your server before installing this version.',
              },
              form: {
                buildPlaceholder: 'Select a build...',
                acceptEula: 'Do you want to accept the Minecraft EULA?',
                acceptEulaDescription:
                  'By enabling this option you confirm that you have read and accept the Minecraft EULA. (https://minecraft.net/eula)',
              },
            },
          },
          stable: 'Stable',
          experimental: 'Experimental',
          release: 'Release',
          snapshot: 'Snapshot',
        },
      },
    },
  },
});

export const useExtTranslations = translations.useTranslations.bind(translations);
export const getExtTranslations = translations.getTranslations.bind(translations);

export default translations;
