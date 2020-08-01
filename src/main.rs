use std::io::{stdin, stdout, Write};

use anyhow::{Context, Result};
use inflector::Inflector;
use rusoto_core::Region;
use rusoto_core::request::HttpClient;
use rusoto_iam::{Iam, IamClient};

use crate::aws::{config, connection};

mod aws;

async fn get_answer(prompt: &str) -> Result<String> {
    print!("{}: ", prompt.to_title_case());
    let _r = stdout().flush();

    let mut input = String::new();
    stdin().read_line(&mut input).context(format!("Error while reading: '{}'", prompt))?;

    Ok(input.trim_end().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let (aws_credentials, aws_config) = config::get_aws_config_files()?;

    let parameters = config::read_automation_info(aws_config);
    let _old_key = config::read_credentials_info(aws_credentials, &*parameters.aws_profile);

    let mfa = get_answer("enter your mfa").await?;

    let mut credentials_provider = connection::get_aws_credentials_provider(&*parameters.aws_profile, &*parameters.aws_mfa_arn);
    credentials_provider.set_mfa_code(mfa);

    let client = IamClient::new_with(HttpClient::new().unwrap(),
                                     credentials_provider,
                                     Region::default());

    let result = client.list_access_keys(rusoto_iam::ListAccessKeysRequest {
        marker: None,
        max_items: None,
        user_name: Option::Some(String::from(parameters.aws_username)),
    }).await?;

    println!("{:?}", result);

    Ok(())
}

