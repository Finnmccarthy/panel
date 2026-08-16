#![allow(clippy::default_constructed_unit_structs)]
#![allow(unused_imports)]

use shared::extensions::{ConstructedExtension, distr::MetadataToml};
use std::sync::Arc;

pub fn list() -> Vec<ConstructedExtension> {
    vec![
        ConstructedExtension {
            metadata_toml: MetadataToml {
                package_name: "dev.0x7d8.eggchanger".to_string(),
                name: "Egg Changer".to_string(),
                panel_version: semver::VersionReq::parse(">=1.1.0").unwrap(),
                license_text: None,
            },
            package_name: "dev.0x7d8.eggchanger",
            description: "Allow users to change the egg/nest of their servers at any time.",
            authors: &["0x7d8"],
            version: semver::Version::parse("1.1.0").unwrap(),
            extension: Arc::new(dev_0x7d8_eggchanger::ExtensionStruct::default()),
        },
        ConstructedExtension {
            metadata_toml: MetadataToml {
                package_name: "dev.0x7d8.minecraftversionchanger".to_string(),
                name: "Minecraft Version Changer".to_string(),
                panel_version: semver::VersionReq::parse(">=1.1.0").unwrap(),
                license_text: None,
            },
            package_name: "dev.0x7d8.minecraftversionchanger",
            description: "Minecraft Version Changer allows you to adjust your minecraft servers version instantly.",
            authors: &["0x7d8"],
            version: semver::Version::parse("1.2.0").unwrap(),
            extension: Arc::new(dev_0x7d8_minecraftversionchanger::ExtensionStruct::default()),
        },
        ConstructedExtension {
            metadata_toml: MetadataToml {
                package_name: "dev.0x7d8.subdomainmanager".to_string(),
                name: "Subdomain Manager".to_string(),
                panel_version: semver::VersionReq::parse(">=1.1.0").unwrap(),
                license_text: None,
            },
            package_name: "dev.0x7d8.subdomainmanager",
            description: "Allow creating Subdomains in the Dashboard using predefined domains easily with an intuitive UI.",
            authors: &["0x7d8"],
            version: semver::Version::parse("1.1.0").unwrap(),
            extension: Arc::new(dev_0x7d8_subdomainmanager::ExtensionStruct::default()),
        },
    ]
}
