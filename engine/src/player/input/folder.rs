use std::sync::atomic::Ordering;

use log::*;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};

use crate::player::{
    controller::ChannelManager,
    utils::{include_file_extension, time_in_seconds, Media},
};
use crate::utils::{config::PlayoutConfig, logging::Target};

/// Folder Sources
///
/// Like playlist source, we create here a folder list for iterate over it.
#[derive(Debug, Clone)]
pub struct FolderSource {
    manager: ChannelManager,
    current_node: Media,
}

impl FolderSource {
    pub async fn new(config: &PlayoutConfig, manager: ChannelManager) -> Self {
        let id = config.general.channel_id;
        let mut path_list = vec![];
        let mut media_list = vec![];
        let mut index: usize = 0;
        let storage = manager.storage.lock().await.clone();

        if !config.storage.paths.is_empty() && config.general.generate.is_some() {
            path_list.extend(&config.storage.paths);
        } else {
            path_list.push(&config.channel.storage);
        }
        if let Some(dates) = &config.general.generate {
            debug!(target: Target::file_mail(), channel = id;
                "generate: {dates:?}, paths: {path_list:?}"
            );
        }

        for path in &path_list {
            if !storage.is_dir(path).await {
                error!(target: Target::file_mail(), channel = id; "Path not exists: <b><magenta>{path:?}</></b>");
            }

            match storage.walk_dir(path).await {
                Ok(paths) => {
                    for p in paths {
                        if storage.is_file(&p).await && include_file_extension(config, &p) {
                            let fetched_path = storage
                                .fetch_file_path(&p.to_string_lossy())
                                .await
                                .unwrap_or(p.to_string_lossy().to_string());
                            let mut media = Media::new(0, &fetched_path, false).await;

                            let key_path = if p.to_string_lossy().starts_with("/") {
                                p.to_string_lossy().to_string()
                            } else {
                                format!("/{}", p.to_string_lossy())
                            };
                            media.key = storage.sanitized_file_path(&key_path);

                            media_list.push(media);
                        }
                    }
                }
                Err(e) => {
                    error!("{e:?}");
                }
            }
        }

        if media_list.is_empty() {
            error!(target: Target::file_mail(), channel = id;
                "no playable files found under: <b><magenta>{:?}</></b>",
                path_list
            );
        }

        if config.storage.shuffle {
            info!(target: Target::file_mail(), channel = id; "Shuffle files");
            let mut rng = StdRng::from_os_rng();
            media_list.shuffle(&mut rng);
        } else {
            media_list.sort_by(|d1, d2| d1.source.cmp(&d2.source));
        }

        for item in &mut media_list {
            item.index = Some(index);

            index += 1;
        }

        *manager.current_list.lock().await = media_list;

        Self {
            manager,
            current_node: Media::default(),
        }
    }

    pub async fn from_list(manager: &ChannelManager, list: Vec<Media>) -> Self {
        *manager.current_list.lock().await = list;

        Self {
            manager: manager.clone(),
            current_node: Media::default(),
        }
    }

    async fn shuffle(&mut self) {
        let mut rng = StdRng::from_os_rng();
        let mut nodes = self.manager.current_list.lock().await;

        nodes.shuffle(&mut rng);

        for (index, item) in nodes.iter_mut().enumerate() {
            item.index = Some(index);
        }
    }

    async fn sort(&mut self) {
        let mut nodes = self.manager.current_list.lock().await;

        nodes.sort_by(|d1, d2| d1.source.cmp(&d2.source));

        for (index, item) in nodes.iter_mut().enumerate() {
            item.index = Some(index);
        }
    }
}

/// Create iterator for folder source
impl FolderSource {
    pub async fn next(&mut self) -> Option<Media> {
        let config = self.manager.config.lock().await.clone();
        let id = config.general.id;

        if self.manager.current_index.load(Ordering::SeqCst)
            < self.manager.current_list.lock().await.len()
        {
            let i = self.manager.current_index.load(Ordering::SeqCst);
            self.current_node = self.manager.current_list.lock().await[i].clone();
            let _ = self.current_node.add_probe(false).await.ok();
            self.current_node
                .add_filter(&config, &self.manager.filter_chain)
                .await;
            self.current_node.begin = Some(time_in_seconds(&config.channel.timezone));
            self.manager.current_index.fetch_add(1, Ordering::SeqCst);
        } else {
            if config.storage.shuffle {
                if config.general.generate.is_none() {
                    info!(target: Target::file_mail(), channel = id; "Shuffle files");
                }

                self.shuffle().await;
            } else {
                if config.general.generate.is_none() {
                    info!(target: Target::file_mail(), channel = id; "Sort files");
                }

                self.sort().await;
            }

            self.current_node = match self.manager.current_list.lock().await.first() {
                Some(m) => m.clone(),
                None => return None,
            };
            let _ = self.current_node.add_probe(false).await.ok();
            self.current_node
                .add_filter(&config, &self.manager.filter_chain)
                .await;
            self.current_node.begin = Some(time_in_seconds(&config.channel.timezone));
            self.manager.current_index.store(1, Ordering::SeqCst);
        }

        Some(self.current_node.clone())
    }
}
