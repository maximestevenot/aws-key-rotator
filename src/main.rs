use std::io::{stdin, stdout, Write};

use anyhow::{Context, Result};
use inflector::Inflector;
use rusoto_core::credential::AutoRefreshingProvider;
use rusoto_core::request::HttpClient;
use rusoto_core::Region;
use rusoto_iam::{
    AccessKey, AccessKeyMetadata, CreateAccessKeyRequest, DeleteAccessKeyRequest, Iam, IamClient,
    ListAccessKeysRequest, UpdateAccessKeyRequest,
};

use crate::aws::config::AwsConfigurationManager;
use crate::aws::connection::CredentialsProviderFactory;

mod aws;

const INACTIVE: &str = "Inactive";

fn get_answer(prompt: &str) -> Result<String> {
    print!("{}: ", prompt.to_title_case());
    let _r = stdout().flush();

    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .context(format!("Error while reading: '{}'", prompt))?;

    Ok(input.trim_end().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut config_manager = AwsConfigurationManager::new();

    let old_key = config_manager.read_credentials_info();

    let mfa = get_answer("enter your mfa")?;

    let credentials_provider = CredentialsProviderFactory::get_sts_credentials_provider(
        config_manager.aws_profile.as_ref(),
        config_manager.aws_mfa_arn.as_ref(),
        mfa.as_ref(),
    )?;

    let credentials_provider = AutoRefreshingProvider::new(credentials_provider)?;

    let iam_client = IamClient::new_with(
        HttpClient::new().unwrap(),
        credentials_provider,
        Region::default(),
    );

    let existing_keys = get_existing_keys(&mut config_manager, &iam_client).await?;

    delete_inactive_keys(&mut config_manager, &iam_client, existing_keys);
    let created_key = create_new_key(&mut config_manager, &iam_client).await?;
    config_manager.write_credentials_info(&created_key);

    disable_old_key(&config_manager, &old_key, &iam_client);

    Ok(())
}

async fn get_existing_keys(
    config_manager: &AwsConfigurationManager,
    iam_client: &IamClient,
) -> Result<Vec<AccessKeyMetadata>> {
    let existing_keys = iam_client
        .list_access_keys(ListAccessKeysRequest {
            marker: None,
            max_items: None,
            user_name: Some(config_manager.aws_username.clone()),
        })
        .await?
        .access_key_metadata;
    Ok(existing_keys)
}

async fn create_new_key(
    config_manager: &AwsConfigurationManager,
    iam_client: &IamClient,
) -> Result<AccessKey> {
    println!("Creating new key");
    let created_key = iam_client
        .create_access_key(CreateAccessKeyRequest {
            user_name: Some(config_manager.aws_username.clone()),
        })
        .await?
        .access_key;
    Ok(created_key)
}

fn disable_old_key(
    config_manager: &AwsConfigurationManager,
    old_key: &AccessKey,
    iam_client: &IamClient,
) {
    let old_key_id = old_key.access_key_id.clone();
    println!("Disabling {}", old_key_id);

    let _disable_response = iam_client.update_access_key(UpdateAccessKeyRequest {
        access_key_id: old_key_id,
        status: INACTIVE.to_string(),
        user_name: Some(config_manager.aws_username.clone()),
    });
}

fn delete_inactive_keys(
    config_manager: &AwsConfigurationManager,
    iam_client: &IamClient,
    existing_keys: Vec<AccessKeyMetadata>,
) {
    for key in existing_keys {
        let status = key.status.unwrap();
        let key_id = key.access_key_id.expect("AccessKeyId");
        println!("Found {} status {}", key_id, status);

        if status == INACTIVE {
            println!("Deleting {}", key_id);

            let _delete_response = iam_client.delete_access_key(DeleteAccessKeyRequest {
                access_key_id: key_id,
                user_name: Some(config_manager.aws_username.clone()),
            });
        }
    }
}
