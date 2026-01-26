use std::path::PathBuf;
use std::sync::Arc;
use object_store::{local::LocalFileSystem, ObjectStore};
use object_store::gcp::GoogleCloudStorageBuilder;
use std::sync::OnceLock;

static STORAGE: OnceLock<Arc<dyn ObjectStore>> = OnceLock::new();

pub fn get_storage() -> Arc<dyn ObjectStore> {
    STORAGE.get_or_init(|| {
        if let Ok(bucket) = std::env::var("GOOGLE_CLOUD_BUCKET") {
            println!("Storage configuring Google Cloud Storage with bucket: {}", bucket);
            let mut builder = GoogleCloudStorageBuilder::new().with_bucket_name(bucket);

            if let Ok(creds_path) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
                builder = builder.with_service_account_path(creds_path);
            }

            let gcs = builder.build().expect("Failed to create GCS client");
            return Arc::new(gcs);
        }

        let path = std::env::var("STORAGE_PATH").map(PathBuf::from).unwrap_or_else(|_| {
            let path = std::env::current_dir()
                .expect("failed to get current dir")
                .join("storage");
            println!("Storage defaulting to persistent directory: {:?}", path);
            path
        });
        
        if !path.exists() {
            std::fs::create_dir_all(&path).expect("failed to create storage directory");
        }
        
        Arc::new(LocalFileSystem::new_with_prefix(path).expect("failed to initialize storage"))
    }).clone()
}
