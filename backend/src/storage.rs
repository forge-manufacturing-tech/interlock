use std::path::PathBuf;
use std::sync::Arc;
use object_store::{local::LocalFileSystem, ObjectStore};
use tempfile::tempdir;
use std::sync::OnceLock;

static STORAGE: OnceLock<Arc<dyn ObjectStore>> = OnceLock::new();

pub fn get_storage() -> Arc<dyn ObjectStore> {
    STORAGE.get_or_init(|| {
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
