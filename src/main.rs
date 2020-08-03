use std::io::{stdin, stdout, Write};

use anyhow::{Context, Result};
use inflector::Inflector;
use rusoto_core::credential::AutoRefreshingProvider;
use rusoto_core::request::HttpClient;
use rusoto_core::Region;
use rusoto_iam::{
    CreateAccessKeyRequest, DeleteAccessKeyRequest, Iam, IamClient, ListAccessKeysRequest,
    UpdateAccessKeyRequest,
};

use crate::aws::{config, connection};

mod aws;

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
    const INACTIVE: &str = "Inactive";

    let (mut aws_credentials, aws_config) = config::get_aws_config_files();

    let parameters = config::read_automation_info(&aws_config);
    let old_key = config::read_credentials_info(&aws_credentials, parameters.aws_profile.as_ref());

    let mfa = get_answer("enter your mfa")?;

    let credentials_provider = connection::get_sts_credentials_provider(
        parameters.aws_profile.as_ref(),
        parameters.aws_mfa_arn.as_ref(),
        mfa.as_ref(),
    )?;

    let credentials_provider = AutoRefreshingProvider::new(credentials_provider)?;

    let iam_client = IamClient::new_with(
        HttpClient::new().unwrap(),
        credentials_provider,
        Region::default(),
    );

    let existing_keys = iam_client
        .list_access_keys(ListAccessKeysRequest {
            marker: None,
            max_items: None,
            user_name: Some(parameters.aws_username.clone()),
        })
        .await?
        .access_key_metadata;

    for key in existing_keys {
        let status = key.status.unwrap();
        let key_id = key.access_key_id.expect("AccessKeyId");
        println!("Found {} status {}", key_id, status);

        if status == INACTIVE {
            println!("Deleting {}", key_id);

            let _delete_response = iam_client.delete_access_key(DeleteAccessKeyRequest {
                access_key_id: key_id,
                user_name: Some(parameters.aws_username.clone()),
            });
        }
    }

    println!("Creating new key");
    let created_key = iam_client
        .create_access_key(CreateAccessKeyRequest {
            user_name: Option::from(parameters.aws_username.clone()),
        })
        .await?
        .access_key;

    config::write_credentials_info(
        &mut aws_credentials,
        parameters.aws_profile.as_ref(),
        &created_key,
    );

    let old_key_id = old_key.access_key_id;
    println!("Disabling {}", old_key_id);

    let _disable_response = iam_client.update_access_key(UpdateAccessKeyRequest {
        access_key_id: old_key_id,
        status: INACTIVE.to_string(),
        user_name: Some(parameters.aws_username.clone()),
    });

    Ok(())
}
