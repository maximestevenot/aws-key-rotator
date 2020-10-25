use anyhow::Result;
use rusoto_core::credential::ProfileProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_sts::{StsClient, StsSessionCredentialsProvider};

use crate::aws::config::AwsConfigurationManager;

pub struct CredentialsProviderFactory;

impl CredentialsProviderFactory {
    pub fn get_sts_credentials_provider(
        profile: &str,
        mfa_arn: &str,
        mfa_code: &str,
    ) -> Result<StsSessionCredentialsProvider> {

        let mut profile_provider = ProfileProvider::new()?;
        if let Some(credentials) = AwsConfigurationManager::get_credentials_path() {
            profile_provider = ProfileProvider::with_configuration(
                credentials,
                profile,
            );
        }
        

        let sts_client = StsClient::new_with(
            HttpClient::new()?,
            profile_provider,
            Region::default(),
        );

        let mut sts_provider =
            StsSessionCredentialsProvider::new(sts_client, None, Some(mfa_arn.to_string()));
        sts_provider.set_mfa_code(mfa_code);
        Ok(sts_provider)
    }
}
