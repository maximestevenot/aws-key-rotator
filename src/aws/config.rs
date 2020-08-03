use std::path::PathBuf;

use dirs::home_dir;
use ini::Ini;
use rusoto_iam::AccessKey;

#[derive(Clone, Debug)]
pub struct AutomationConfig {
    pub aws_profile: String,
    pub aws_username: String,
    pub aws_mfa_arn: String,
}

pub fn get_aws_config_files() -> (Ini, Ini) {
    let credentials = get_credentials_path()
        .and_then(|it| Ini::load_from_file(it.as_path()).ok())
        .expect("Error while reading credentials file");

    let config = get_config_path()
        .and_then(|it| Ini::load_from_file(it.as_path()).ok())
        .expect("Error while reading config file");

    (credentials, config)
}

pub fn get_config_path() -> Option<PathBuf> {
    get_dir_path().map(|it| it.join(PathBuf::from("config")))
}

pub fn get_credentials_path() -> Option<PathBuf> {
    get_dir_path().map(|it| it.join(PathBuf::from("credentials")))
}

fn get_dir_path() -> Option<PathBuf> {
    home_dir().map(|it| it.join(PathBuf::from(".aws")))
}

pub fn read_credentials_info(credentials_file: &Ini, profile: &str) -> AccessKey {
    AccessKey {
        access_key_id: read_property(credentials_file, profile, "aws_access_key_id"),
        secret_access_key: read_property(credentials_file, profile, "aws_secret_access_key"),
        create_date: None,
        status: String::default(),
        user_name: String::default(),
    }
}

pub fn write_credentials_info(credentials_file: &mut Ini, profile: &str, key: &AccessKey) {
    println!("Writing {}", key.access_key_id);

    let section = credentials_file.section_mut(Some(profile));
    if let Some(it) = section {
        it.insert("aws_access_key_id", key.access_key_id.clone());
        it.insert("aws_secret_access_key", key.secret_access_key.clone())
    }

    get_credentials_path()
        .and_then(|it| credentials_file.write_to_file(it.as_path()).ok())
        .expect("Error while writing credentials file")
}

pub fn read_automation_info(config_file: &Ini) -> AutomationConfig {
    let section = "automation";
    AutomationConfig {
        aws_profile: read_property(config_file, section, "profile"),
        aws_username: read_property(config_file, section, "username"),
        aws_mfa_arn: read_property(config_file, section, "mfa_arn"),
    }
}

fn read_property(config_file: &Ini, section: &str, key: &str) -> String {
    config_file
        .section(Some(section))
        .and_then(|it| it.get(key))
        .map(|it| it.to_string())
        .unwrap_or_else(|| panic!("Field '{}' is missing in section '{}'", section, key))
}
