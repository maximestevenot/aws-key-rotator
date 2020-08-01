pub mod config;

pub mod connection {
    use anyhow::Result;
    use rusoto_core::{HttpClient, Region};
    use rusoto_core::credential::{AutoRefreshingProvider, ProfileProvider};
    use rusoto_sts::{StsClient, StsSessionCredentialsProvider};

    use crate::aws::config::get_credentials_path;

    async fn get_sts_credentials_provider(profile: &str, mfa_arn: &str) -> Result<StsSessionCredentialsProvider> {
        let profile_provider = ProfileProvider::with_configuration(get_credentials_path().unwrap(), profile);

        let sts_client = StsClient::new_with(HttpClient::new().unwrap(),
                                             profile_provider,
                                             Region::default());

        Ok(StsSessionCredentialsProvider::new(sts_client,
                                              Option::None,
                                              Option::from(mfa_arn.to_string())))
    }

    pub async fn get_credentials_provider(aws_profile: &str,
                                          aws_mfa_arn: &str,
                                          mfa_code: &str) -> Result<AutoRefreshingProvider<StsSessionCredentialsProvider>> {
        let mut sts_provider = get_sts_credentials_provider(aws_profile, aws_mfa_arn).await?;
        sts_provider.set_mfa_code(mfa_code);
        Ok(AutoRefreshingProvider::new(sts_provider)?)
    }
}
