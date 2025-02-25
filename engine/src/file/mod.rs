use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
};

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utils::ABS_PATH_INDICATOR;

mod local;
mod s3;
pub mod utils;

use crate::file::{utils::filler, utils::media_map::SharedMediaMap};
use crate::player::utils::Media;
use crate::utils::{config::PlayoutConfig, errors::ServiceError};

use s3::S3_INDICATOR;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PathObject {
    pub source: String,
    parent: Option<String>,
    parent_folders: Option<Vec<String>>,
    folders: Option<Vec<String>>,
    files: Option<Vec<VideoFile>>,
    #[serde(default)]
    pub folders_only: bool,
    #[serde(default)]
    pub recursive: bool,
}

impl PathObject {
    fn new(source: String, parent: Option<String>) -> Self {
        Self {
            source,
            parent,
            parent_folders: Some(vec![]),
            folders: Some(vec![]),
            files: Some(vec![]),
            folders_only: false,
            recursive: false,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MoveObject {
    source: String,
    target: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VideoFile {
    name: String,
    duration: f64,
}

#[derive(Clone, Debug)]
pub enum StorageType {
    Local,
    S3,
}

#[derive(Clone, Debug)]
pub enum StorageBackend {
    Local(local::LocalStorage),
    S3(s3::S3Storage),
}

impl StorageBackend {
    pub fn interpreted_file_path(&self, path: &str) -> String {
        match self {
            StorageBackend::Local(storage) => storage.interpreted_file_path(path),
            StorageBackend::S3(storage) => storage.interpreted_file_path(path),
        }
    }

    pub fn sanitized_file_path(&self, path: &str) -> String {
        match self {
            StorageBackend::Local(storage) => storage.sanitized_file_path(path),
            StorageBackend::S3(storage) => storage.sanitized_file_path(path),
        }
    }

    pub fn echo_log(&self) {
        match self {
            StorageBackend::Local(storage) => storage.echo_log(),
            StorageBackend::S3(storage) => storage.echo_log(),
        }
    }

    pub async fn fetch_file_path(&self, file_path: &str) -> Result<String, ServiceError> {
        match self {
            StorageBackend::Local(storage) => storage.fetch_file_path(file_path).await,
            StorageBackend::S3(storage) => storage.fetch_file_path(file_path).await,
        }
    }

    pub async fn browser(
        &self,
        path_obj: &PathObject,
        dur_data: web::Data<SharedMediaMap>,
    ) -> Result<PathObject, ServiceError> {
        match self {
            StorageBackend::Local(storage) => storage.browser(path_obj, dur_data).await,
            StorageBackend::S3(storage) => storage.browser(path_obj, dur_data).await,
        }
    }

    pub async fn mkdir(&self, path_obj: &PathObject) -> Result<(), ServiceError> {
        match self {
            StorageBackend::Local(storage) => storage.mkdir(path_obj).await,
            StorageBackend::S3(storage) => storage.mkdir(path_obj).await,
        }
    }

    pub async fn rename(
        &self,
        move_object: &MoveObject,
        duration: web::Data<SharedMediaMap>,
    ) -> Result<MoveObject, ServiceError> {
        match self {
            StorageBackend::Local(storage) => storage.rename(move_object, duration).await,
            StorageBackend::S3(storage) => storage.rename(move_object, duration).await,
        }
    }

    pub async fn remove(
        &self,
        source_path: &str,
        duration: web::Data<SharedMediaMap>,
        recursive: bool,
    ) -> Result<(), ServiceError> {
        match self {
            StorageBackend::Local(storage) => {
                storage.remove(source_path, duration, recursive).await
            }
            StorageBackend::S3(storage) => storage.remove(source_path, duration, recursive).await,
        }
    }

    pub async fn upload(
        &self,
        payload: Multipart,
        path: &Path,
        is_abs: bool,
    ) -> Result<(), ServiceError> {
        match self {
            StorageBackend::Local(storage) => storage.upload(payload, path, is_abs).await,
            StorageBackend::S3(storage) => storage.upload(payload, path, is_abs).await,
        }
    }

    pub async fn watchman(
        &mut self,
        config: PlayoutConfig,
        is_alive: Arc<AtomicBool>,
        sources: Arc<Mutex<Vec<Media>>>,
    ) {
        match self {
            StorageBackend::Local(storage) => storage.watchman(config, is_alive, sources).await,
            StorageBackend::S3(storage) => storage.watchman(config, is_alive, sources).await,
        }
    }

    pub async fn stop_watch(&mut self) {
        match self {
            StorageBackend::Local(storage) => storage.stop_watch().await,
            StorageBackend::S3(storage) => storage.stop_watch().await,
        }
    }

    pub async fn fill_filler_list(
        &mut self,
        config: &PlayoutConfig,
        fillers: Option<Arc<Mutex<Vec<Media>>>>,
    ) -> Vec<Media> {
        let filler_path = &config.storage.filler;
        if filler_path.starts_with(ABS_PATH_INDICATOR) {
            filler::absolute_fill_filler_list(config, fillers).await
        } else {
            match self {
                StorageBackend::Local(storage) => storage.fill_filler_list(config, fillers).await,
                StorageBackend::S3(storage) => storage.fill_filler_list(config, fillers).await,
            }
        }
    }

    pub async fn copy_assets(&self) -> Result<(), std::io::Error> {
        match self {
            StorageBackend::Local(storage) => storage.copy_assets().await,
            StorageBackend::S3(storage) => storage.copy_assets().await,
        }
    }

    pub async fn is_dir<P: AsRef<Path>>(&self, input: P) -> bool {
        match self {
            StorageBackend::Local(storage) => storage.is_dir(input).await,
            StorageBackend::S3(storage) => storage.is_dir(input).await,
        }
    }

    pub async fn is_file<P: AsRef<Path>>(&self, input: P) -> bool {
        match self {
            StorageBackend::Local(storage) => storage.is_file(input).await,
            StorageBackend::S3(storage) => storage.is_file(input).await,
        }
    }

    pub async fn walk_dir<P: AsRef<Path>>(&self, input: P) -> Result<Vec<PathBuf>, ServiceError> {
        match self {
            StorageBackend::Local(storage) => storage.walk_dir(input).await,
            StorageBackend::S3(storage) => storage.walk_dir(input).await,
        }
    }

    pub async fn open_media(
        &self,
        _req: &HttpRequest,
        path_obj: &str,
    ) -> Result<HttpResponse, ServiceError> {
        match self {
            StorageBackend::Local(storage) => storage.open_media(_req, path_obj).await,
            StorageBackend::S3(storage) => storage.open_media(_req, path_obj).await,
        }
    }
}

trait Storage {
    fn path_prefix_generator(&self) -> String;
    fn interpreted_file_path(&self, path: &str) -> String;
    fn sanitized_file_path(&self, path: &str) -> String;
    fn echo_log(&self);
    async fn fetch_file_path(&self, file_path: &str) -> Result<String, ServiceError>;
    async fn browser(
        &self,
        path_obj: &PathObject,
        dur_data: web::Data<SharedMediaMap>,
    ) -> Result<PathObject, ServiceError>;
    async fn mkdir(&self, path_obj: &PathObject) -> Result<(), ServiceError>;
    async fn rename(
        &self,
        move_object: &MoveObject,
        duration: web::Data<SharedMediaMap>,
    ) -> Result<MoveObject, ServiceError>;
    async fn remove(
        &self,
        source_path: &str,
        duration: web::Data<SharedMediaMap>,
        recursive: bool,
    ) -> Result<(), ServiceError>;
    async fn upload(&self, data: Multipart, path: &Path, is_abs: bool) -> Result<(), ServiceError>;
    async fn watchman(
        &mut self,
        config: PlayoutConfig,
        is_alive: Arc<AtomicBool>,
        sources: Arc<Mutex<Vec<Media>>>,
    );
    async fn stop_watch(&mut self);
    async fn fill_filler_list(
        &mut self,
        config: &PlayoutConfig,
        fillers: Option<Arc<Mutex<Vec<Media>>>>,
    ) -> Vec<Media>;
    async fn copy_assets(&self) -> Result<(), std::io::Error>;
    async fn is_dir<P: AsRef<Path>>(&self, input: P) -> bool;
    async fn is_file<P: AsRef<Path>>(&self, input: P) -> bool;
    async fn walk_dir<P: AsRef<Path>>(&self, input: P) -> Result<Vec<PathBuf>, ServiceError>;
    async fn open_media(
        &self,
        _req: &HttpRequest,
        file_path: &str,
    ) -> Result<HttpResponse, ServiceError>;
}

pub fn select_storage_type<S: AsRef<std::ffi::OsStr>>(path: S) -> StorageType {
    let path_str = path.as_ref().to_string_lossy().to_lowercase();

    if path_str.starts_with(S3_INDICATOR) {
        return StorageType::S3;
    }

    StorageType::Local
}

pub async fn init_storage(
    storage_type: StorageType,
    root: PathBuf,
    extensions: Vec<String>,
) -> StorageBackend {
    match storage_type {
        StorageType::Local => {
            StorageBackend::Local(local::LocalStorage::new(root, extensions).await)
        }
        StorageType::S3 => StorageBackend::S3(s3::S3Storage::new(root, extensions).await),
    }
}

/// Normalize absolut path
///
/// This function takes care, that it is not possible to break out from root_path.
pub fn norm_abs_path(
    root_path: &Path,
    input_path: &str,
) -> Result<(PathBuf, String, String), ServiceError> {
    let path_relative = RelativePath::new(&root_path.to_string_lossy())
        .normalize()
        .to_string()
        .replace("../", "");
    let path_suffix = root_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut source_relative = RelativePath::new(input_path)
        .normalize()
        .to_string()
        .replace("../", "");

    if input_path.starts_with(&*root_path.to_string_lossy())
        || source_relative.starts_with(&path_relative)
    {
        source_relative = source_relative
            .strip_prefix(&path_relative)
            .and_then(|s| s.strip_prefix('/'))
            .unwrap_or_default()
            .to_string();
    } else {
        source_relative = source_relative
            .strip_prefix(&path_suffix)
            .and_then(|s| s.strip_prefix('/'))
            .unwrap_or(&source_relative)
            .to_string();
    }

    let path = &root_path.join(&source_relative);

    Ok((path.clone(), path_suffix, source_relative))
}
