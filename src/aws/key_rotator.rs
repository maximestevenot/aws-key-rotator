use anyhow::{Context, Result};
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
    pub fn new(mfa_code: &str) -> Result<Self> {
        let config_manager = AwsConfigurationManager::new();

        let credentials_provider = CredentialsProviderFactory::get_sts_credentials_provider(
            config_manager.aws_profile.as_ref(),
            config_manager.aws_mfa_arn.as_ref(),
            mfa_code,
        )?;

        let credentials_provider = AutoRefreshingProvider::new(credentials_provider)?;

        let iam_client =
            IamClient::new_with(HttpClient::new()?, credentials_provider, Region::default());

        Ok(Self {
            config_manager,
            iam_client,
        })
    }

    pub async fn process(&mut self) -> Result<()> {
        let old_key = self.config_manager.read_credentials_info();

        let existing_keys = self.get_existing_keys().await.unwrap_or_default();

        self.delete_inactive_keys(existing_keys).await;

        let created_key: AccessKey = self.create_new_key().await?;

        self.config_manager.write_credentials_info(&created_key);
        self.disable_old_key(&old_key).await?;
        Ok(())
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

    async fn delete_inactive_keys(&self, existing_keys: Vec<AccessKeyMetadata>) -> Option<()> {
        for key in existing_keys {
            let status = key.status?;
            let key_id = key.access_key_id?;
            println!("Found {} status {}", key_id, status);

            if status == INACTIVE {
                println!("Deleting {}", key_id);

                let _delete_response = self
                    .iam_client
                    .delete_access_key(DeleteAccessKeyRequest {
                        access_key_id: key_id,
                        user_name: Some(self.config_manager.aws_username.clone()),
                    })
                    .await
                    .context(format!("Error while deleting {} key", status));
            }
        }
        Some(())
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

    async fn disable_old_key(&self, old_key: &AccessKey) -> Result<()> {
        let old_key_id = old_key.access_key_id.clone();
        println!("Disabling {}", old_key_id);

        let _disable_response = self
            .iam_client
            .update_access_key(UpdateAccessKeyRequest {
                access_key_id: old_key_id,
                status: INACTIVE.to_string(),
                user_name: Some(self.config_manager.aws_username.clone()),
            })
            .await?;
        Ok(())
    }
}
