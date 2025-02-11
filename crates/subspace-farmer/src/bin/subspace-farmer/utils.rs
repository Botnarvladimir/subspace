use std::fs;
use std::path::PathBuf;

pub(crate) fn get_path(custom_path: Option<PathBuf>) -> PathBuf {
    // set storage path
    let path = custom_path.unwrap_or_else(|| {
        dirs::data_local_dir()
            .expect("Can't find local data directory, needs to be specified explicitly")
            .join("subspace-farmer")
    });

    if !path.exists() {
        fs::create_dir_all(&path).unwrap_or_else(|error| {
            panic!("Failed to create data directory {:?}: {:?}", path, error)
        });
    }

    path
}
