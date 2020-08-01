pub mod config;

pub mod connection {
    use rusoto_core::{HttpClient, Region};
    use rusoto_core::credential::ProfileProvider;
    use rusoto_sts::{StsClient, StsSessionCredentialsProvider};

    use crate::aws::config::get_credentials_path;

    pub fn get_aws_credentials_provider(aws_profile: &str, aws_mfa_arn: &str) -> StsSessionCredentialsProvider {
        let profile_provider = ProfileProvider::with_configuration(get_credentials_path().unwrap(),
                                                                   aws_profile);
        let sts_client = StsClient::new_with(HttpClient::new().unwrap(),
                                             profile_provider,
                                             Region::default());

        StsSessionCredentialsProvider::new(sts_client,
                                           Option::None,
                                           Option::from(String::from(aws_mfa_arn)),
        )
    }
}
