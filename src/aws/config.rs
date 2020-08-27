use std::path::PathBuf;

use dirs::home_dir;
use ini::Ini;
use rusoto_iam::AccessKey;

const AUTOMATION_SECTION: &str = "automation";

pub struct AwsConfigurationManager {
    credentials_file: Ini,
    pub aws_profile: String,
    pub aws_username: String,
    pub aws_mfa_arn: String,
}

impl AwsConfigurationManager {
    pub fn new() -> Self {
        let credentials_file = Self::get_credentials_path()
            .and_then(|it| Ini::load_from_file(it.as_path()).ok())
            .expect("Error while reading credentials file");

        let configuration_file = Self::get_config_path()
            .and_then(|it| Ini::load_from_file(it.as_path()).ok())
            .expect("Error while reading config file");

        let aws_profile = Self::read_property(&configuration_file, AUTOMATION_SECTION, "profile");
        let aws_username = Self::read_property(&configuration_file, AUTOMATION_SECTION, "username");
        let aws_mfa_arn = Self::read_property(&configuration_file, AUTOMATION_SECTION, "mfa_arn");

        Self {
            credentials_file,
            aws_profile,
            aws_username,
            aws_mfa_arn,
        }
    }

    pub fn get_config_path() -> Option<PathBuf> {
        Self::get_dir_path().map(|it| it.join(PathBuf::from("config")))
    }

    pub fn get_credentials_path() -> Option<PathBuf> {
        Self::get_dir_path().map(|it| it.join(PathBuf::from("credentials")))
    }

    pub fn read_credentials_info(&self) -> AccessKey {
        AccessKey {
            access_key_id: Self::read_property(
                &self.credentials_file,
                self.aws_profile.as_str(),
                "aws_access_key_id",
            ),
            secret_access_key: Self::read_property(
                &self.credentials_file,
                self.aws_profile.as_str(),
                "aws_secret_access_key",
            ),
            create_date: None,
            status: String::default(),
            user_name: String::default(),
        }
    }

    pub fn write_credentials_info(&mut self, key: &AccessKey) {
        println!("Writing {}", key.access_key_id);

        let section = self
            .credentials_file
            .section_mut(Some(self.aws_profile.as_str()));
        if let Some(it) = section {
            it.insert("aws_access_key_id", &key.access_key_id);
            it.insert("aws_secret_access_key", &key.secret_access_key)
        }

        Self::get_credentials_path()
            .and_then(|it| self.credentials_file.write_to_file(it.as_path()).ok())
            .expect("Error while writing credentials file")
    }

    fn read_property(file: &Ini, section: &str, key: &str) -> String {
        file.section(Some(section))
            .and_then(|it| it.get(key))
            .map(|it| it.to_string())
            .unwrap_or_else(|| panic!("Field '{}' is missing in section '{}'", section, key))
    }

    fn get_dir_path() -> Option<PathBuf> {
        home_dir().map(|it| it.join(PathBuf::from(".aws")))
    }
}
