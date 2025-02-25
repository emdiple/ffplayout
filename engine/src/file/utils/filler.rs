use async_walkdir::WalkDir;
use std::path::PathBuf;
use std::sync::Arc;

use tokio_stream::StreamExt;

use lexical_sort::natural_lexical_cmp;
use log::*;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use tokio::sync::Mutex;

use crate::player::utils::{include_file_extension, Media};
use crate::utils::{config::PlayoutConfig, logging::Target};

use super::ABS_PATH_INDICATOR;

pub async fn absolute_fill_filler_list(
    config: &PlayoutConfig,
    fillers: Option<Arc<Mutex<Vec<Media>>>>,
) -> Vec<Media> {
    let id = config.general.channel_id;
    let mut filler_list = vec![];
    // let filler_path = &config.storage.filler_path;

    let raw_filler_path = &config.storage.filler_path.to_string_lossy();
    let filler_sanitized_path = raw_filler_path
        .strip_prefix(ABS_PATH_INDICATOR)
        .unwrap_or(raw_filler_path);
    let filler_sanitized_path = if filler_sanitized_path.starts_with("/") {
        filler_sanitized_path
    } else {
        &format!("/{}", filler_sanitized_path)
    };
    let filler_path = PathBuf::from(filler_sanitized_path);

    if filler_path.is_dir() {
        // let config_clone = config.clone();
        let mut index = 0;
        let mut entries = WalkDir::new(&filler_path);

        while let Some(Ok(entry)) = entries.next().await {
            if entry.path().is_file() && include_file_extension(config, &entry.path()) {
                let mut media = Media::new(index, &entry.path().to_string_lossy(), false).await;

                if fillers.is_none() {
                    if let Err(e) = media.add_probe(false).await {
                        error!(target: Target::file_mail(), channel = id; "{e:?}");
                    };
                }

                filler_list.push(media);
                index += 1;
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
    } else if filler_path.is_file() {
        let mut media = Media::new(0, &config.storage.filler_path.to_string_lossy(), false).await;

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

    filler_list
}
