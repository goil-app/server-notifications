use aws_config::Region;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;
use std::sync::Arc;

/// Servicio para firmar URLs de S3
#[derive(Clone)]
pub struct S3UrlSigner {
    client: Arc<S3Client>,
    bucket_name: String,
}

impl S3UrlSigner {
    /// Crea una nueva instancia del firmador de URLs de S3
    pub async fn new() -> Result<Self, String> {
        let bucket_name = std::env::var("PUBLIC_BUCKET")
            .map_err(|_| "PUBLIC_BUCKET environment variable not set")?;

        let region_str = std::env::var("AWS_REGION")
            .unwrap_or_else(|_| "eu-west-3".to_string());
        let region = Region::new(region_str);

        // Configurar credenciales desde variables de entorno
        // Temporalmente establecer las variables con nombres estándar de AWS
        let access_key = std::env::var("AWS_ACCESS_KEY")
            .map_err(|_| "AWS_ACCESS_KEY environment variable not set")?;
        let secret_key = std::env::var("AWS_SECRET_KEY")
            .map_err(|_| "AWS_SECRET_KEY environment variable not set")?;

        // Establecer las variables con los nombres estándar que aws-config espera
        std::env::set_var("AWS_ACCESS_KEY_ID", &access_key);
        std::env::set_var("AWS_SECRET_ACCESS_KEY", &secret_key);

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region)
            .load()
            .await;

        let client = Arc::new(S3Client::new(&config));

        Ok(Self {
            client,
            bucket_name,
        })
    }

    /// Normaliza el path de la imagen para que coincida con la estructura en S3
    /// Convierte "notification/image/..." a "notifications/images/..."
    fn normalize_key(&self, key: &str) -> String {
        // Si la key ya es una URL completa, retornarla sin modificar
        if key.starts_with("http://") || key.starts_with("https://") {
            return key.to_string();
        }

        // Log el key original
        println!("[S3UrlSigner::normalize_key] Original key: '{}'", key);

        // Transformar "notification/image/" a "notifications/images/"
        // Usar replace para manejar todos los casos
        let normalized = if key.starts_with("notification/image/") {
            format!("notifications/images/{}", &key[19..]) // "notification/image/".len() = 19
        } else if key.starts_with("notification/images/") {
            format!("notifications/images/{}", &key[20..]) // "notification/images/".len() = 20
        } else if key.starts_with("notifications/image/") {
            format!("notifications/images/{}", &key[21..]) // "notifications/image/".len() = 21
        } else {
            key.to_string() // Ya está bien o tiene otro formato
        };

        if key != normalized {
            println!("[S3UrlSigner::normalize_key] Normalized: '{}' -> '{}'", key, normalized);
        } else {
            println!("[S3UrlSigner::normalize_key] No normalization needed: '{}'", key);
        }

        normalized
    }

    /// Firma una URL de S3 para un objeto específico
    /// Equivalente a getSignedUrl con GetObjectCommand en TypeScript
    /// 
    /// # Argumentos
    /// * `key` - La clave del objeto en S3 (ej: "notifications/images/image.png" o "notification/image/image.png")
    /// * `expires_in` - Duración en segundos para la validez de la URL (default: 600 = 10 minutos)
    /// 
    /// # Retorna
    /// Una URL firmada de S3 o un error
    pub async fn sign_url(&self, key: &str, expires_in: u64) -> Result<String, String> {
        // Normalizar la key antes de firmarla
        let normalized_key = self.normalize_key(key);
        
        println!("[S3UrlSigner::sign_url] Bucket: '{}', Key: '{}', Expires: {}s", 
                 self.bucket_name, normalized_key, expires_in);

        // Construir la configuración de presigning (equivalente a { expiresIn: params.Expires } en TS)
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(expires_in))
            .map_err(|e| format!("Error creating presigning config: {}", e))?;

        // Crear el request de presigned URL (equivalente a new GetObjectCommand en TS)
        let request = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(&normalized_key)
            .presigned(presigning_config)
            .await
            .map_err(|e| format!("Error generating presigned URL for bucket '{}', key '{}': {}", 
                                 self.bucket_name, normalized_key, e))?;

        let signed_url = request.uri().to_string();
        println!("[S3UrlSigner::sign_url] Generated signed URL (full): {}", signed_url);
        
        Ok(signed_url)
    }

    /// Firma múltiples URLs de S3
    pub async fn sign_urls(&self, keys: &[String], expires_in: u64) -> Result<Vec<String>, String> {
        let mut signed_urls = Vec::new();

        for key in keys {
            let signed_url = self.sign_url(key, expires_in).await?;
            signed_urls.push(signed_url);
        }

        Ok(signed_urls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Ignorar en CI/CD ya que requiere credenciales AWS
    async fn test_sign_url() {
        // Este test requiere variables de entorno configuradas
        if let Ok(signer) = S3UrlSigner::new().await {
            let key = "notifications/images/test.png";
            let result = signer.sign_url(key, 600).await;
            assert!(result.is_ok());
            let url = result.unwrap();
            assert!(url.contains("https://"));
            assert!(url.contains("X-Amz-Signature"));
        }
    }
}

