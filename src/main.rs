use std::io::{stdin, stdout, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use dirs::home_dir;
use inflector::Inflector;
use ini::Ini;
use rusoto_core::{Client, credential::ProfileProvider, Region};
use rusoto_core::request::HttpClient;
use rusoto_iam::{AccessKey, CreateAccessKeyRequest, Iam, IamClient, ListAccessKeysError, ListAccessKeysResponse};

fn aws_credentials_file_path() -> PathBuf {
    let mut filename = home_dir().unwrap();
    filename.push(".aws");
    filename.push("credentials");
    filename
}

async fn get_answer(prompt: &str) -> Result<String> {
    print!("{}: ", prompt.to_title_case());
    let _r = stdout().flush();

    let mut input = String::new();
    stdin().read_line(&mut input).context(format!("Error while reading: '{}'", prompt))?;

    Ok(input)
}

fn get_aws_key_from_file(profile: &str) -> AccessKey {
    let conf = Ini::load_from_file(aws_credentials_file_path().as_path()).unwrap();
    let section = conf.section(Some(profile)).unwrap();
    AccessKey {
        access_key_id: String::from(section.get("aws_access_key_id").unwrap()),
        secret_access_key: String::from(section.get("aws_secret_access_key").unwrap()),
        create_date: None,
        status: String::default(),
        user_name: String::default(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let aws_profile = "central";
    let aws_username = "maxime.stevenot@soprabanking.com";
    let _mfa = get_answer("mfa").await?;

    let _old_key = get_aws_key_from_file(aws_profile);

    // let req = CreateAccessKeyRequest {
    //     user_name: Option::Some(String::from(aws_username)),
    // };

    let profile_provider = ProfileProvider::with_configuration(aws_credentials_file_path(), aws_profile);
    let client = IamClient::new_with(HttpClient::new().unwrap(),
                                     profile_provider,
                                     Region::default());

    let result = client.list_access_keys(rusoto_iam::ListAccessKeysRequest {
        marker: None,
        max_items: None,
        user_name: Option::Some(String::from(aws_username)),
    }).await?;

    println!("{:?}", result);

    Ok(())
}

