use anyhow::Result;
use rusoto_core::credential::ProfileProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_sts::{StsClient, StsSessionCredentialsProvider};

use crate::aws::config::get_credentials_path;

pub fn get_sts_credentials_provider(
    profile: &str,
    mfa_arn: &str,
    mfa_code: &str,
) -> Result<StsSessionCredentialsProvider> {
    let profile_provider =
        ProfileProvider::with_configuration(get_credentials_path().unwrap(), profile);

    let sts_client = StsClient::new_with(
        HttpClient::new().unwrap(),
        profile_provider,
        Region::default(),
    );

    let mut sts_provider =
        StsSessionCredentialsProvider::new(sts_client, None, Some(mfa_arn.to_string()));
    sts_provider.set_mfa_code(mfa_code);
    Ok(sts_provider)
}
