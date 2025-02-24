use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct MediaMap {
    pub video_duration_data: Mutex<HashMap<String, f64>>,
    queue: Mutex<VecDeque<String>>,
    limit: usize,
}

impl MediaMap {
    pub fn create(limit: usize) -> Self {
        Self {
            video_duration_data: Mutex::new(HashMap::with_capacity(limit)),
            queue: Mutex::new(VecDeque::with_capacity(limit)),
            limit,
        }
    }

    pub async fn add_obj(&self, key: String, value: f64) -> Result<(), &'static str> {
        // insert item with FIFO algorithm
        let mut media_duration = self.video_duration_data.lock().await;
        let mut queue = self.queue.lock().await;
        if media_duration.len() >= self.limit {
            if let Some(oldest_key) = queue.pop_front() {
                media_duration.remove(&oldest_key.clone());
            }
        }
        media_duration.insert(key.clone(), value);
        queue.push_back(key);
        Ok(())
    }

    pub async fn get_obj(&self, key: &str) -> Option<f64> {
        let media_duration = self.video_duration_data.lock().await;
        media_duration.get(key).copied()
    }

    pub async fn remove_obj(&self, key: &str) -> Result<(), &'static str> {
        let mut media_duration = self.video_duration_data.lock().await;
        let mut queue = self.queue.lock().await;
        media_duration.remove(key);
        if let Some(index) = queue.iter().position(|x| x == key) {
            queue.remove(index);
        }
        Ok(())
    }

    pub async fn update_obj(&self, old_key: &str, new_key: &str) -> Result<(), &'static str> {
        let mut media_duration = self.video_duration_data.lock().await;
        let mut queue = self.queue.lock().await;

        // Remove old key and get its value, if it exists
        if let Some(value) = media_duration.remove(old_key) {
            // Replace old key in queue if present, or add new key
            match queue.iter().position(|x| x == old_key) {
                Some(index) => queue[index] = new_key.to_string(),
                None => {
                    if queue.len() >= self.limit {
                        if let Some(oldest_key) = queue.pop_front() {
                            media_duration.remove(&oldest_key);
                        }
                    }
                    queue.push_back(new_key.to_string());
                }
            }
            // Insert new key-value pair
            media_duration.insert(new_key.to_string(), value);
        }

        Ok(())
    }
}

pub type SharedMediaMap = Arc<MediaMap>;
