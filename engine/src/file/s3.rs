use std::io::Write;
use std::{
    collections::HashSet,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use actix_multipart::Multipart;
use actix_web::HttpRequest;
use actix_web::{http::StatusCode, web, HttpResponse};
use futures_util::TryStreamExt as _;
use lexical_sort::{natural_lexical_cmp, PathSort};
use log::*;

use rand::{distr::Alphanumeric, rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use regex::Regex;
use tokio::{sync::Mutex, task::JoinHandle};

use crate::file::{utils::media_map::SharedMediaMap, MoveObject, PathObject, Storage, VideoFile};
use crate::player::utils::{include_file_extension, probe::MediaProbe, Media};
use crate::utils::{config::PlayoutConfig, errors::ServiceError, logging::Target};

use aws_config::Region;
use aws_sdk_s3::{
    presigning::PresigningConfig,
    types::{CompletedMultipartUpload, CompletedPart},
    Client,
};

pub const S3_INDICATOR: &str = "s3://";
pub const S3_DEFAULT_PRESIGNEDURL_EXP: f64 = 3600.0 * 24.0;
// pub const S3_MAX_KEYS: i32 = 50000;

#[derive(Clone, Debug)]
pub struct S3Storage {
    pub root: PathBuf,
    original_root: PathBuf,
    pub extensions: Vec<String>,
    endpoint: String,
    bucket: String,
    client: Client,
    pub watch_handler: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl S3Storage {
    pub async fn new(root: PathBuf, extensions: Vec<String>) -> Self {
        let (credentials, bucket, endpoint_url) = s3_parse_string(&root.to_string_lossy())
            .unwrap_or_else(|e| panic!("Invalid S3 schema!: {}", e));
        let credentials_cloned = credentials.clone();
        let endpoint_url_cloned = endpoint_url.clone();
        Self {
            root: PathBuf::new(),
            original_root: root,
            extensions,
            endpoint: endpoint_url.clone(),
            bucket,
            client: {
                let shared_provider =
                    aws_sdk_s3::config::SharedCredentialsProvider::new(credentials_cloned);
                let config = aws_config::from_env()
                    .region(Region::new("us-east-1")) // Dummy default region, will added if needed!
                    .credentials_provider(shared_provider)
                    .load()
                    .await;

                let s3_config = aws_sdk_s3::config::Builder::from(&config)
                    .endpoint_url(&endpoint_url_cloned)
                    .force_path_style(true)
                    .build();

                aws_sdk_s3::Client::from_conf(s3_config)
            },
            watch_handler: Arc::new(Mutex::new(None)),
        }
    }

    /// Watches an S3 bucket for changes and updates the `sources` list accordingly.
    ///
    /// # Arguments
    /// - `config`: The playout configuration, including channel ID and file extension filters.
    /// - `is_alive`: Atomic boolean to control the loop's lifetime.
    /// - `sources`: Shared list of `Media` objects, protected by a mutex.
    /// - `s3_client`: AWS S3 client for interacting with the bucket.
    /// - `bucket_name`: Name of the S3 bucket to monitor.
    pub async fn watch_s3(
        &self,
        config: PlayoutConfig,
        is_alive: Arc<AtomicBool>,
        sources: Arc<Mutex<Vec<Media>>>,
    ) {
        let id = config.general.channel_id;
        let mut previous_keys = HashSet::new();
        let bucket_name = &self.bucket.clone();
        let s3_client = &self.client.clone();

        info!(target: Target::file_mail(), channel = id;
            "Monitoring S3 bucket: <b><magenta>{}</></b>",
            bucket_name
        );

        while is_alive.load(Ordering::SeqCst) {
            let resp = s3_client.list_objects_v2().bucket(bucket_name).send().await;

            match resp {
                Ok(output) => {
                    let current_keys: HashSet<String> = output
                        .contents
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|obj| obj.key.clone())
                        .collect();

                    // Detect added objects
                    let added = current_keys.difference(&previous_keys);
                    for key in added {
                        if include_file_extension(&config, Path::new(key)) {
                            let fetched_path =
                                &self.fetch_file_path(key).await.unwrap_or(key.to_string());
                            let media =
                                Media::new(id.try_into().unwrap(), fetched_path, false).await;
                            sources.lock().await.push(media);
                            info!(target: Target::file_mail(), channel = id;
                                "Added S3 object: <b><magenta>{}</></b>",
                                &key
                            );
                        }
                    }

                    // Detect removed objects
                    let removed = previous_keys.difference(&current_keys);
                    for key in removed {
                        sources.lock().await.retain(|x| x.source != *key);
                        info!(target: Target::file_mail(), channel = id;
                            "Removed S3 object: <b><magenta>{}</></b>",
                            key
                        );
                    }

                    previous_keys = current_keys;
                }
                Err(e) => {
                    error!(target: Target::file_mail(), channel = id;
                        "Error listing S3 objects: {:?}", e
                    );
                }
            }

            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }

    async fn s3_get_object(
        &self,
        object_key: &str,
        expires_in: u64,
    ) -> Result<String, ServiceError> {
        let expires_in = Duration::from_secs(expires_in);
        let presigned_request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(object_key)
            .presigned(
                PresigningConfig::expires_in(expires_in)
                    .map_err(|_| ServiceError::InternalServerError)?,
            )
            .await
            .map_err(|e| ServiceError::BadRequest(format!("Invalid S3 config!: {}", e)))?;

        Ok(presigned_request.uri().to_string())
    }

    fn s3_rename(source_path: &str, target_path: &str) -> Result<MoveObject, ServiceError> {
        let source_name = source_path.rsplit('/').next().unwrap_or(source_path);
        let target_name = target_path.rsplit('/').next().unwrap_or(target_path);

        Ok(MoveObject {
            source: source_name.to_string(),
            target: target_name.to_string(),
        })
    }

    async fn s3_copy_object(
        source_object: &str,
        destination_object: &str,
        bucket: &str,
        client: &aws_sdk_s3::Client,
    ) -> Result<(), ServiceError> {
        // let clean_source = s3_path(source_object)?;
        // let clean_target = s3_path(destination_object)?;
        let source_key = format!("{bucket}/{source_object}");
        client
            .copy_object()
            .copy_source(&source_key)
            .bucket(bucket)
            .key(destination_object)
            .send()
            .await
            .map_err(|e| ServiceError::Conflict(format!("Failed to copy object!: {}", e)))?;
        Ok(())
    }

    async fn s3_delete_prefix(
        source_path: &str,
        bucket: &str,
        s3_client: &Client,
        recursive: bool,
    ) -> Result<(), ServiceError> {
        let (clean_path, parent_path) = s3_path(source_path)?;
        let delimiter = '/';
        let parent_list_resp = s3_client // list of objects and prefix in parent path
            .list_objects_v2()
            .bucket(bucket)
            .prefix(&parent_path)
            .delimiter(delimiter)
            // .max_keys(S3_MAX_KEYS)
            .send()
            .await
            .map_err(|e| ServiceError::BadRequest(format!("Invalid S3 config!: {}", e)))?;
        for prefix in parent_list_resp.common_prefixes() {
            // detele prefix
            if let Some(prefix) = prefix.prefix() {
                if prefix == source_path {
                    if recursive {
                        // recursive deleting
                        let target_fld_list_resp = s3_client
                            .list_objects_v2()
                            .bucket(bucket)
                            .prefix(&clean_path)
                            // .max_keys(S3_MAX_KEYS)
                            .send()
                            .await
                            .map_err(|_| ServiceError::InternalServerError)?;
                        for objs in target_fld_list_resp.contents() {
                            if let Some(obj) = objs.key() {
                                s3_client
                                    .delete_object()
                                    .bucket(bucket)
                                    .key(obj)
                                    .send()
                                    .await
                                    .map_err(|_| {
                                        ServiceError::BadRequest("Source does not exists!".into())
                                    })?;
                            }
                        }
                    } else {
                        // non-recursive deleting
                        let target_fld_list_resp = s3_client
                            .list_objects_v2()
                            .bucket(bucket)
                            .prefix(&clean_path)
                            .delimiter(delimiter)
                            // .max_keys(S3_MAX_KEYS)
                            .send()
                            .await
                            .map_err(|_e| ServiceError::InternalServerError)?;
                        for objs in target_fld_list_resp.contents() {
                            if let Some(obj) = objs.key() {
                                s3_client
                                    .delete_object()
                                    .bucket(bucket)
                                    .key(obj)
                                    .send()
                                    .await
                                    .map_err(|_| {
                                        ServiceError::BadRequest("Source does not exists!".into())
                                    })?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn s3_delete_object(
        source_path: &str,
        bucket: &str,
        s3_client: &Client,
    ) -> Result<(), ServiceError> {
        s3_client
            .delete_object()
            .bucket(bucket)
            .key(source_path)
            .send()
            .await
            .map_err(|e| ServiceError::Conflict(format!("Failed to remove object!: {}", e)))?;

        Ok(())
    }

    async fn s3_rename_object(
        source_object: &str,
        destination_object: &str,
        bucket: &str,
        client: &aws_sdk_s3::Client,
        duration: web::Data<SharedMediaMap>,
    ) -> Result<(), ServiceError> {
        Self::s3_copy_object(source_object, destination_object, bucket, client).await?;
        Self::s3_delete_object(source_object, bucket, client).await?;
        duration
            .update_obj(source_object, destination_object)
            .await?;

        Ok(())
    }

    /// **Check if Path is an S3 Object**
    ///
    /// Verifies whether the given path corresponds to an object in the specified S3 bucket.
    async fn s3_is_object(&self, path: &str) -> Result<bool, ServiceError> {
        let mut is_object = false;
        let (clean_path, parent_path) = s3_path(path)?;
        let delimiter = '/';
        let parent_list_resp = &self
            .client // list of objects and prefix in parent path
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&parent_path)
            .delimiter(delimiter)
            // .max_keys(S3_MAX_KEYS)
            .send()
            .await
            .map_err(|e| ServiceError::BadRequest(format!("Invalid S3 config!: {}", e)))?;
        for object in parent_list_resp.contents() {
            if let Some(key) = object.key() {
                if key == clean_path {
                    is_object = true;
                }
            }
        }
        Ok(is_object)
    }

    /// **Check if Path is an S3 Prefix**
    ///
    /// Checks if the given path corresponds to a prefix in the specified S3 bucket.
    async fn s3_is_prefix(&self, path: &str) -> Result<bool, ServiceError> {
        let mut is_prefix = false;
        let (clean_path, parent_path) = s3_path(path)?;
        let delimiter = '/';
        let parent_list_resp = &self
            .client // list of objects and prefix in parent path
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&parent_path)
            .delimiter(delimiter)
            // .max_keys(S3_MAX_KEYS)
            .send()
            .await
            .map_err(|e| ServiceError::BadRequest(format!("Invalid S3 config!: {}", e)))?;
        for prefix in parent_list_resp.common_prefixes() {
            if let Some(prefix) = prefix.prefix() {
                if prefix == clean_path {
                    is_prefix = true;
                }
            }
        }
        Ok(is_prefix)
    }
}

impl Storage for S3Storage {
    /// Generate proper prefix key for objects' paths.
    fn path_prefix_generator(&self) -> String {
        format!("[S3:]/{}", &self.bucket)
    }
    /// Returns a sanitized file path by normalizing the root and prefixing with "s3:/{bucket}/".
    fn sanitized_file_path(&self, path: &str) -> String {
        let root = self.original_root.to_string_lossy().to_string();
        let normalized_root = Regex::new(r"/+").unwrap().replace_all(&root, "/");
        let staged_path = path
            .strip_prefix(&normalized_root.to_string())
            .unwrap_or(path)
            .to_string();
        let path_prefix = &self.path_prefix_generator();
        format!("{}{}", path_prefix, staged_path) // baked_path
    }

    ///Interpret json media' source address to engine readble version
    fn interpreted_file_path(&self, path: &str) -> String {
        let path_prefix = &self.path_prefix_generator();
        path.strip_prefix(path_prefix).unwrap_or(path).to_string()
    }
    fn echo_log(&self) {
        info!(
            "<blue>S3 Storage initialized at '{}/{}/'",
            &self.endpoint, &self.bucket
        );
    }
    async fn fetch_file_path(&self, file_path: &str) -> Result<String, ServiceError> {
        let (cleaned_root_prefix, _) = s3_path(&self.original_root.to_string_lossy())?;
        let validated_file_path = file_path
            .strip_prefix(&cleaned_root_prefix)
            .unwrap_or(file_path);
        self.s3_get_object(validated_file_path, S3_DEFAULT_PRESIGNEDURL_EXP as u64)
            .await
    }
    async fn browser(
        &self,
        path_obj: &PathObject,
        dur_data: web::Data<SharedMediaMap>,
    ) -> Result<PathObject, ServiceError> {
        // let s3_obj_dur = duration;
        let media_duration = dur_data;
        let mut parent_folders = vec![];
        let bucket = &self.bucket;
        let path = path_obj.source.clone();
        let delimiter = '/'; // should be a single character
        let (prefix, parent_path) = s3_path(&path_obj.source)?;
        let s3_client = &self.client;
        let mut obj = PathObject::new(path.clone(), Some(bucket.clone()));
        obj.folders_only = path_obj.folders_only;

        if (prefix != parent_path && !path_obj.folders_only)
            || (!prefix.is_empty() && (parent_path.is_empty()))
        {
            let childs_resp = s3_client
                .list_objects_v2()
                .bucket(bucket)
                .prefix(&parent_path)
                .delimiter(delimiter)
                // .max_keys(S3_MAX_KEYS)
                .send()
                .await
                .map_err(|_e| ServiceError::InternalServerError)?;

            for prefix in childs_resp.common_prefixes() {
                if let Some(prefix) = prefix.prefix() {
                    let child = prefix.split(delimiter).nth_back(1).unwrap_or("");
                    parent_folders.push(child.to_string());
                }
            }
            parent_folders.path_sort(natural_lexical_cmp);

            obj.parent_folders = Some(parent_folders);
        }

        let list_resp = s3_client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(&prefix)
            .delimiter(delimiter)
            // .max_keys(S3_MAX_KEYS)
            .send()
            .await
            .map_err(|_| ServiceError::InternalServerError)?;

        let mut folders: Vec<String> = vec![];
        let mut files: Vec<String> = vec![];

        for prefix in list_resp.common_prefixes() {
            if let Some(prefix) = prefix.prefix() {
                let fldrs = prefix.split(delimiter).nth_back(1).unwrap_or(prefix);
                folders.push(fldrs.to_string());
            }
        }

        for objs in list_resp.contents() {
            if let Some(obj) = objs.key() {
                if s3_obj_extension_checker(obj, &self.extensions) {
                    let fls = obj.strip_prefix(bucket).unwrap_or(obj);
                    files.push(fls.to_string());
                }
            }
        }

        files.path_sort(natural_lexical_cmp);
        folders.path_sort(natural_lexical_cmp);

        let mut media_files = vec![];
        for file in files {
            let s3file_presigned_url = self
                .s3_get_object(&file, S3_DEFAULT_PRESIGNEDURL_EXP as u64)
                .await?;
            let name = file.strip_prefix(&prefix).unwrap_or(&file).to_string();
            if let Some(stored_dur) = media_duration.get_obj(&file).await {
                let video = VideoFile {
                    name,
                    duration: stored_dur,
                };
                media_files.push(video);
            } else {
                match MediaProbe::new(&s3file_presigned_url).await {
                    Ok(probe) => {
                        let duration = probe.format.duration.unwrap_or_default();
                        media_duration.add_obj(file, duration).await?;
                        let video = VideoFile { name, duration };
                        media_files.push(video);
                    }
                    Err(e) => error!("{e:?}"),
                };
            }
        }

        obj.folders = Some(folders);
        obj.files = Some(media_files);
        Ok(obj)
    }
    async fn mkdir(&self, path_obj: &PathObject) -> Result<(), ServiceError> {
        let bucket: &str = &self.bucket;
        let (folder_path, _) = &s3_path(&path_obj.source)?;
        let none_file = format!("{}.ignore", folder_path); // it should be made to validate the new folder's existence
        let client = &self.client;

        let body = aws_sdk_s3::primitives::ByteStream::from(Vec::new()); // to not consume bytes!
        client
            .put_object()
            .bucket(bucket)
            .key(&none_file)
            .body(body)
            .send()
            .await
            .map_err(|_e| ServiceError::InternalServerError)?;
        Ok(())
    }
    async fn rename(
        &self,
        move_object: &MoveObject,
        duration: web::Data<SharedMediaMap>,
    ) -> Result<MoveObject, ServiceError> {
        let bucket = &self.bucket.clone();
        let client = &self.client.clone();

        let obj_names = Self::s3_rename(&move_object.source, &move_object.target).unwrap();
        if !Self::s3_is_prefix(self, &move_object.source).await? {
            Self::s3_rename_object(
                &move_object.source,
                &move_object.target,
                bucket,
                client,
                duration,
            )
            .await?;
        }

        Ok(MoveObject {
            source: obj_names.source.to_string(),
            target: obj_names.target.to_string(),
        })
    }
    async fn remove(
        &self,
        source_path: &str,
        duration: web::Data<SharedMediaMap>,
        recursive: bool,
    ) -> Result<(), ServiceError> {
        let bucket = &self.bucket;
        let client = &self.client;
        let (clean_path, _) = s3_path(source_path)?;

        if Self::s3_is_prefix(self, &clean_path).await? {
            Self::s3_delete_prefix(&clean_path, bucket, client, recursive).await?;
        } else {
            Self::s3_delete_object(&clean_path, bucket, client).await?;
            duration.remove_obj(&clean_path).await?;
        }

        Ok(())
    }
    async fn upload(
        &self,
        mut data: Multipart,
        path: &Path,
        _is_abs: bool,
    ) -> Result<(), ServiceError> {
        let mut upload_id: Option<String> = None;
        let mut key: Option<String> = None;
        let mut completed_parts: Vec<CompletedPart> = Vec::new();
        let mut part_number = 1;
        let mut s3_upload_permit = false;
        let path = path.to_string_lossy();

        while let Some(mut field) = data.try_next().await.map_err(|e| e.to_string())? {
            let content_disposition = field
                .content_disposition()
                .ok_or("No content disposition")?;
            debug!("{content_disposition}");

            let mut rng = rand::rng();

            let rand_string: String = (&mut rng)
                .sample_iter(Alphanumeric)
                .take(20)
                .map(char::from)
                .collect();

            let filename = content_disposition
                .get_filename()
                .map_or_else(|| rand_string, sanitize_filename::sanitize);

            let filepath = format!("{path}{filename}");

            if upload_id.is_none() {
                let create_multipart_upload_output = &self
                    .client
                    .create_multipart_upload()
                    .bucket(&self.bucket)
                    .key(&filepath)
                    .send()
                    .await
                    .map_err(|e| format!("Failed to initiate multipart upload: {}", e))?;

                upload_id = create_multipart_upload_output
                    .upload_id()
                    .map(ToString::to_string);
                // .map(|id| id.to_string());

                key = Some(filepath.clone());
            }
            let mut f = web::block(|| std::io::Cursor::new(Vec::new()))
                .await
                .map_err(|e| e.to_string())?;
            loop {
                match field.try_next().await {
                    Ok(Some(chunk)) => {
                        f = web::block(move || f.write_all(&chunk).map(|_| f))
                            .await
                            .map_err(|e| e.to_string())??;
                        s3_upload_permit = true;
                    }
                    Ok(None) => break,
                    Err(e) => {
                        if e.to_string().contains("stream is incomplete") {
                            info!("Incomplete stream for part, continuing multipart upload: {e}");

                            tokio::fs::remove_file(filepath).await?;
                        }

                        return Err(e.into());
                    }
                }
            }
            let body_bytes = actix_web::web::Bytes::from(f.into_inner());

            let upload_part_output = &self
                .client
                .upload_part()
                .bucket(&self.bucket)
                .key(key.as_ref().unwrap())
                .upload_id(upload_id.as_ref().unwrap())
                .part_number(part_number)
                .body(body_bytes.into())
                .send()
                .await
                .map_err(|e| format!("Failed to upload part: {}", e))?;

            completed_parts.push(
                CompletedPart::builder()
                    .e_tag(upload_part_output.e_tag().unwrap())
                    .part_number(part_number)
                    .build(),
            );
            part_number += 1;
        }
        if s3_upload_permit {
            let completed_multipart_upload = CompletedMultipartUpload::builder()
                .set_parts(Some(completed_parts))
                .build();

            self.client
                .complete_multipart_upload()
                .bucket(&self.bucket)
                .key(key.as_ref().unwrap())
                .upload_id(upload_id.as_ref().unwrap())
                .multipart_upload(completed_multipart_upload)
                .send()
                .await
                .map_err(|e| format!("Failed to complete multipart upload: {}", e))?;
        }
        Ok(())
    }

    async fn watchman(
        &mut self,
        config: PlayoutConfig,
        is_alive: Arc<AtomicBool>,
        sources: Arc<Mutex<Vec<Media>>>,
    ) {
        let s3_storage = self.clone();
        let task = tokio::spawn(async move {
            s3_storage.watch_s3(config, is_alive, sources).await;
        });

        *self.watch_handler.lock().await = Some(task);
    }

    async fn stop_watch(&mut self) {
        let mut watch_handler = self.watch_handler.lock().await;

        if let Some(handler) = watch_handler.as_mut() {
            handler.abort();
        }
    }

    async fn open_media(
        &self,
        _req: &HttpRequest,
        file_path: &str,
    ) -> Result<HttpResponse, ServiceError> {
        let obj_key = file_path.strip_prefix(&self.bucket).unwrap_or(file_path);
        let expires_in = S3_DEFAULT_PRESIGNEDURL_EXP as u64;

        let obj_url = self.s3_get_object(obj_key, expires_in).await?;

        // Redirect to the pre-signed S3 URL
        Ok(HttpResponse::build(StatusCode::FOUND)
            .append_header(("Location", obj_url))
            .finish())
    }

    async fn fill_filler_list(
        &mut self,
        config: &PlayoutConfig,
        fillers: Option<Arc<Mutex<Vec<Media>>>>,
    ) -> Vec<Media> {
        let bucket = &self.bucket;
        let client = &self.client;

        let id = config.general.channel_id;
        let mut filler_list = vec![];
        let filler_path = &config.storage.filler_path;
        let mut index = 0;

        if self.is_dir(filler_path.to_str().unwrap()).await {
            let mut obj_list_resp = client
                .list_objects_v2()
                .bucket(bucket)
                .prefix(filler_path.to_str().unwrap())
                // .max_keys(S3_MAX_KEYS)
                .into_paginator()
                .send();

            while let Some(result) = obj_list_resp.next().await {
                match result {
                    Ok(output) => {
                        for object in output.contents() {
                            let obj_key = object.key().unwrap_or("Unknown");
                            let presigned_url = self
                                .s3_get_object(obj_key, S3_DEFAULT_PRESIGNEDURL_EXP as u64)
                                .await
                                .unwrap_or(obj_key.to_string());
                            if include_file_extension(config, Path::new(obj_key)) {
                                let mut media = Media::new(index, &presigned_url, false).await;
                                if fillers.is_none() {
                                    if let Err(e) = media.add_probe(false).await {
                                        error!(target: Target::file_mail(), channel = id; "{e:?}");
                                    };
                                }
                                filler_list.push(media);
                                index += 1;
                            }
                        }
                    }
                    Err(err) => {
                        error!("{err:?}");
                    }
                }
            }
            if config.storage.shuffle {
                let mut rng = StdRng::from_os_rng();
                filler_list.shuffle(&mut rng);
            } else {
                filler_list.sort_by(|d1, d2| natural_lexical_cmp(&d1.source, &d2.source));
            }

            for (index, item) in filler_list.iter_mut().enumerate() {
                item.index = Some(index);
            }

            if let Some(f) = fillers.as_ref() {
                f.lock().await.clone_from(&filler_list);
            }
        } else
        // if s3_utils::s3_is_object(filler_path.to_str().unwrap(), bucket, client)
        //     .await
        //     .unwrap_or(false)
        {
            let presigned_url = self
                .s3_get_object(
                    filler_path.to_str().unwrap(),
                    S3_DEFAULT_PRESIGNEDURL_EXP as u64,
                )
                .await
                .unwrap_or(filler_path.to_string_lossy().to_string());
            let mut media = Media::new(0, &presigned_url, false).await;

            if fillers.is_none() {
                if let Err(e) = media.add_probe(false).await {
                    error!(target: Target::file_mail(), channel = id; "{e:?}");
                };
            }

            filler_list.push(media);

            if let Some(f) = fillers.as_ref() {
                f.lock().await.clone_from(&filler_list);
            }
        }

        vec![]
    }

    async fn copy_assets(&self) -> Result<(), std::io::Error> {
        Ok(())
    }

    async fn is_dir<P: AsRef<Path>>(&self, input: P) -> bool {
        if input.as_ref().to_string_lossy() == self.original_root.to_string_lossy() {
            return true;
        }
        (self.s3_is_prefix(&input.as_ref().to_string_lossy()).await).unwrap_or_default()
    }

    async fn is_file<P: AsRef<Path>>(&self, input: P) -> bool {
        (self.s3_is_object(&input.as_ref().to_string_lossy()).await).unwrap_or_default()
    }

    /// Asynchronously collects all S3 object paths under a prefix.
    ///
    /// # Parameters
    /// - `input`: S3 prefix to list objects.
    ///
    /// # Returns
    /// - `Ok(Vec<PathBuf>)`: Paths of all objects and prefixes.
    /// - `Err(ServiceError)`: On S3 request error.
    async fn walk_dir<P: AsRef<Path>>(&self, input: P) -> Result<Vec<PathBuf>, ServiceError> {
        let input = input.as_ref();
        let (cleaned_root_prefix, _) = s3_path(&self.original_root.to_string_lossy())?;
        let validated_file_path = input
            .strip_prefix(&cleaned_root_prefix)
            .unwrap_or(input)
            .to_string_lossy();
        let bucket = &self.bucket;
        let client = &self.client;
        let mut objects = Vec::new();

        let mut list_objs = client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(validated_file_path)
            // .max_keys(S3_MAX_KEYS)
            .into_paginator()
            .send();

        while let Some(result) = list_objs.next().await {
            match result {
                Ok(output) => {
                    for object in output.contents() {
                        if let Some(obj_key) = object.key() {
                            objects.push(PathBuf::from(obj_key));
                        }
                    }
                    for prefix in output.common_prefixes() {
                        if let Some(prefix_key) = prefix.prefix() {
                            objects.push(PathBuf::from(prefix_key));
                        }
                    }
                }
                Err(err) => {
                    error!("{err:?}");
                    return Err(ServiceError::Conflict(err.to_string()));
                }
            }
        }

        Ok(objects)
    }
}

/// **S3 String Parser**
///
/// ## Purpose
/// Parses S3 configuration details from a provided string, extracting the bucket name, endpoint URL, and AWS credentials.
///
/// ## Input Format
/// The input string must follow this format:
/// `s3://{bucket_name}/:{endpoint_url}/:{access_key}/:{secret_key}`
///
/// - **`bucket_name`**: The name of the S3 bucket.
/// - **`endpoint_url`**: The URL of the S3 endpoint. If missing `http://` or `https://`, `http://` is automatically added.
/// - **`access_key`**: The AWS or S3 access key.
/// - **`secret_key`**: The AWS or S3 secret key.
///
/// ## Example
/// ```rust
/// let s3_string = "s3://my_bucket/:http://example.com/:my_access_key/:my_secret_key";
/// match s3_parse_string(s3_string) {
///     Ok((credentials, bucket_name, endpoint_url)) => {
///         assert_eq!(bucket_name, "my_bucket");
///         assert_eq!(endpoint_url, "http://example.com");
///         assert_eq!(credentials.access_key_id(), "my_access_key");
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// ## Returns
/// A `Result` containing:
/// - **`aws_sdk_s3::config::Credentials`**: AWS credentials.
/// - **`String`**: The S3 bucket name.
/// - **`String`**: The endpoint URL.
///
/// Returns an error if the string does not match the expected format.
///
/// ## Errors
/// - Returns `std::io::Error` if the input is invalid.
///
/// ## Use Case
/// Use this function to parse S3 configurations from strings, such as environment variables or config files.
pub fn s3_parse_string(
    s3_str: &str,
) -> Result<(aws_sdk_s3::config::Credentials, String, String), std::io::Error> {
    let pattern = format!(r"{}([^/]+)/:(.*?)/:([^/]+)/:([^/]+)", S3_INDICATOR);

    let re = Regex::new(&pattern)
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "Failed to compile regex"))?;
    // Match the input string against the regex
    if let Some(captures) = re.captures(s3_str) {
        let access_key = captures[3].to_string();
        let secret_key = captures[4].to_string();
        let mut endpoint = captures[2].to_string();

        if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            endpoint = format!("http://{}", endpoint);
        }

        Ok((
            aws_sdk_s3::config::Credentials::new(access_key, secret_key, None, None, "None"), // Credential
            captures[1].to_string(), // bucket-name
            endpoint,                // endpoint-url
        ))
    } else {
        Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "Input S3 string does not match the expected format: {}",
                s3_str
            ),
        ))
    }
}

/// **S3 Path Preparer**
///
/// Cleans and validates an input path for S3 compatibility, ensuring proper formatting.
///
/// ## Parameters
/// - **`input_path: &str`**: The raw input path to be processed.
///
/// ## Returns
/// - **`Result<(String, String), ServiceError>`**:
///   - **`clean_path`**: The sanitized path.
///   - **`clean_parent_path`**: The sanitized parent path.
///
/// ## Notes
/// - Removes redundant slashes and ensures the path ends with a `/` where appropriate.
/// - Returns an empty string for invalid or empty paths.
pub fn s3_path(input_path: &str) -> Result<(String, String), ServiceError> {
    fn s3_clean_path(input_path: &str) -> Result<String, ServiceError> {
        let re = Regex::new("//+").unwrap(); // Matches one or more '/'
        let none_redundant_path = re.replace_all(input_path, "/");
        let clean_path = if !none_redundant_path.is_empty() && none_redundant_path != "/" {
            if input_path.ends_with("/")
                || Path::new(&none_redundant_path.to_string())
                    .extension()
                    .is_some()
            {
                none_redundant_path.trim_start_matches("/").to_string()
            } else {
                format!("{}/", none_redundant_path.trim_start_matches("/"))
            }
        } else {
            String::new()
        };
        Ok(clean_path)
    }
    let clean_path = s3_clean_path(input_path)?;
    let clean_parent_path = s3_clean_path(&format!(
        "{}/",
        clean_path
            .rsplit('/')
            .skip(2)
            .collect::<Vec<&str>>()
            .iter()
            .rev()
            .cloned()
            .collect::<Vec<&str>>()
            .join("/")
    ))?;

    Ok((clean_path, clean_parent_path))
}

fn s3_obj_extension_checker(obj_name: &str, extensions: &[String]) -> bool {
    extensions.iter().any(|ext| obj_name.ends_with(ext))
}
