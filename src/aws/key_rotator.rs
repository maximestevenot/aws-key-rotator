use anyhow::Result;
use rusoto_core::credential::AutoRefreshingProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_iam::{
    AccessKey, AccessKeyMetadata, CreateAccessKeyRequest, DeleteAccessKeyRequest, Iam, IamClient,
    ListAccessKeysRequest, UpdateAccessKeyRequest,
};

use crate::aws::config::AwsConfigurationManager;
use crate::aws::connection::CredentialsProviderFactory;

const INACTIVE: &str = "Inactive";

pub struct AwsKeyRotator {
    config_manager: AwsConfigurationManager,
    iam_client: IamClient,
}

impl AwsKeyRotator {
    pub fn new(mfa_code: &str) -> Self {
        let config_manager = AwsConfigurationManager::new();

        let credentials_provider = CredentialsProviderFactory::get_sts_credentials_provider(
            config_manager.aws_profile.as_ref(),
            config_manager.aws_mfa_arn.as_ref(),
            mfa_code.as_ref(),
        )
        .expect("Error while getting STS Credentials Provider");

        let credentials_provider = AutoRefreshingProvider::new(credentials_provider)
            .expect("Error while getting Auto Refreshing Provider");

        let iam_client = IamClient::new_with(
            HttpClient::new().unwrap(),
            credentials_provider,
            Region::default(),
        );

        Self {
            config_manager,
            iam_client,
        }
    }

    pub async fn process(&mut self) {
        let old_key = self.config_manager.read_credentials_info();

        let existing_keys = self.get_existing_keys().await.unwrap_or(Vec::default());

        self.delete_inactive_keys(existing_keys);

        let created_key = self
            .create_new_key()
            .await
            .expect("Error while creating new key");

        self.config_manager.write_credentials_info(&created_key);

        self.disable_old_key(&old_key);
    }

    async fn get_existing_keys(&self) -> Result<Vec<AccessKeyMetadata>> {
        let existing_keys = self
            .iam_client
            .list_access_keys(ListAccessKeysRequest {
                marker: None,
                max_items: None,
                user_name: Some(self.config_manager.aws_username.clone()),
            })
            .await?
            .access_key_metadata;
        Ok(existing_keys)
    }

    fn delete_inactive_keys(&self, existing_keys: Vec<AccessKeyMetadata>) {
        for key in existing_keys {
            let status = key.status.unwrap();
            let key_id = key.access_key_id.expect("AccessKeyId");
            println!("Found {} status {}", key_id, status);

            if status == INACTIVE {
                println!("Deleting {}", key_id);

                let _delete_response = self.iam_client.delete_access_key(DeleteAccessKeyRequest {
                    access_key_id: key_id,
                    user_name: Some(self.config_manager.aws_username.clone()),
                });
            }
        }
    }

    async fn create_new_key(&self) -> Result<AccessKey> {
        println!("Creating new key");
        let created_key = self
            .iam_client
            .create_access_key(CreateAccessKeyRequest {
                user_name: Some(self.config_manager.aws_username.clone()),
            })
            .await?
            .access_key;
        Ok(created_key)
    }

    fn disable_old_key(&self, old_key: &AccessKey) {
        let old_key_id = old_key.access_key_id.clone();
        println!("Disabling {}", old_key_id);

        let _disable_response = self.iam_client.update_access_key(UpdateAccessKeyRequest {
            access_key_id: old_key_id,
            status: INACTIVE.to_string(),
            user_name: Some(self.config_manager.aws_username.clone()),
        });
    }
}
