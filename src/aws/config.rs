use std::path::PathBuf;

use anyhow::Result;
use dirs::home_dir;
use ini::Ini;
use rusoto_iam::AccessKey;

#[derive(Clone, Debug)]
pub struct AutomationConfig {
    pub aws_profile: String,
    pub aws_username: String,
    pub aws_mfa_arn: String,
}

pub fn get_config_path() -> Result<PathBuf> {
    get_dir_path().and_then(|it| push_and_return(it, "config"))
}

pub fn get_credentials_path() -> Result<PathBuf> {
    get_dir_path().and_then(|it| push_and_return(it, "credentials"))
}

pub fn get_aws_config_files() -> Result<(Ini, Ini)> {
    let credentials = Ini::load_from_file(get_credentials_path()?.as_path())?;
    let config = Ini::load_from_file(get_config_path()?.as_path())?;

    Ok((credentials, config))
}

pub fn read_credentials_info(credentials_file: &Ini, profile: &str) -> Result<AccessKey> {
    let section = credentials_file.section(Some(profile)).unwrap();
    Ok(AccessKey {
        access_key_id: section.get("aws_access_key_id").unwrap().to_string(),
        secret_access_key: section.get("aws_secret_access_key").unwrap().to_string(),
        create_date: None,
        status: String::default(),
        user_name: String::default(),
    })
}

pub fn write_credentials_info(mut credentials_file: Ini, profile: &str, key: AccessKey) -> Result<()> {
    let section = credentials_file.section_mut(Some(profile)).unwrap();

    println!("Writing {}", key.access_key_id);

    section.insert("aws_access_key_id", key.access_key_id);
    section.insert("aws_secret_access_key", key.secret_access_key);
    credentials_file.write_to_file(get_credentials_path()?.as_path())?;
    Ok(())
}

pub fn read_automation_info(config_file: &Ini) -> AutomationConfig {
    let section = config_file.section(Some("automation")).unwrap();
    AutomationConfig {
        aws_profile: section.get("profile").unwrap().to_string(),
        aws_username: section.get("username").unwrap().to_string(),
        aws_mfa_arn: section.get("mfa_arn").unwrap().to_string(),
    }
}

fn get_dir_path() -> Result<PathBuf> {
    let mut dir_path = home_dir().unwrap();
    dir_path.push(".aws");
    Ok(dir_path)
}

fn push_and_return(mut path_buf: PathBuf, path: &str) -> Result<PathBuf> {
    path_buf.push(path);
    Ok(path_buf)
}
