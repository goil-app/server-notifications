use crate::infrastructure::s3::S3UrlSigner;

#[derive(Clone)]
pub struct StorageServiceProvider {
    pub s3_signer: S3UrlSigner,
}

impl StorageServiceProvider {
    pub async fn new() -> Result<Self, String> {
        let s3_signer = S3UrlSigner::new().await?;

        Ok(Self {
            s3_signer,
        })
    }
}

