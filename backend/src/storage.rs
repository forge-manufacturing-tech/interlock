use std::path::PathBuf;
use std::sync::Arc;
use object_store::{local::LocalFileSystem, ObjectStore};
use tempfile::tempdir;
use std::sync::OnceLock;

static STORAGE: OnceLock<Arc<dyn ObjectStore>> = OnceLock::new();

pub fn get_storage() -> Arc<dyn ObjectStore> {
    STORAGE.get_or_init(|| {
        let path = std::env::var("STORAGE_PATH").map(PathBuf::from).unwrap_or_else(|_| {
            let tmp = tempdir().expect("failed to create temp dir");
            let path = tmp.into_path(); 
            println!("Storage defaulting to temporary directory: {:?}", path);
            path
        });
        
        if !path.exists() {
            std::fs::create_dir_all(&path).expect("failed to create storage directory");
        }
        
        Arc::new(LocalFileSystem::new_with_prefix(path).expect("failed to initialize storage"))
    }).clone()
}
