use super::ServerUrlItem;
use crate::{
    constants::{SERVER_URL_BUCKET_DEFAULT_ITEM_ID, SERVER_URL_BUCKET_MAX_ITEMS},
    types::profile::UID,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ServerUrlBucket {
    /// User ID
    pub uid: UID,
    /// [`HashMap`] Key is the [`ServerUrlItem`]`.id`.
    pub items: HashMap<String, ServerUrlItem>,
}

impl ServerUrlBucket {
    /// Create a new [`ServerUrlBucket`] with the base URL inserted.
    pub fn new(uid: UID, base_url: Url) -> Self {
        let mut items = HashMap::new();

        let server_url_item = ServerUrlItem {
            id: SERVER_URL_BUCKET_DEFAULT_ITEM_ID.to_string(),
            url: base_url.clone(),
            mtime: Self::current_timestamp() as i64,
            selected: true,
        };

        // Use the item's id as the key in the HashMap
        items.insert(server_url_item.id.clone(), server_url_item);

        ServerUrlBucket { uid, items }
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    pub fn merge_bucket(&mut self, bucket: ServerUrlBucket) {
        if self.uid == bucket.uid {
            self.merge_items(bucket.items.into_values().collect());
        }
    }

    pub fn merge_items(&mut self, items: Vec<ServerUrlItem>) {
        for new_item in items.into_iter() {
            match self.items.get_mut(&new_item.id) {
                Some(item) => {
                    *item = new_item;
                }
                None => {
                    if self.items.len() < SERVER_URL_BUCKET_MAX_ITEMS {
                        self.items.insert(new_item.id.to_owned(), new_item);
                    } else {
                        let oldest_item_id_option = self
                            .items
                            .values()
                            .filter(|item| item.id != SERVER_URL_BUCKET_DEFAULT_ITEM_ID)
                            .min_by_key(|item| item.mtime)
                            .map(|item| item.id.clone());

                        if let Some(oldest_item_id) = oldest_item_id_option {
                            if new_item.mtime > self.items[&oldest_item_id].mtime {
                                self.items.remove(&oldest_item_id);
                                self.items.insert(new_item.id.to_owned(), new_item);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn edit_item(&mut self, id: &str, new_url: Url) -> Result<(), String> {
        if let Some(item) = self.items.get_mut(id) {
            item.url = new_url;
            item.mtime = Self::current_timestamp() as i64;
            Ok(())
        } else {
            Err("Item not found".to_string())
        }
    }

    /// Delete an item by its ID
    pub fn delete_item(&mut self, id: &str) -> Result<(), String> {
        if id == SERVER_URL_BUCKET_DEFAULT_ITEM_ID {
            return Err("Cannot remove the base URL item.".to_string());
        }
        if self.items.remove(id).is_some() {
            Ok(())
        } else {
            Err("Item not found".to_string())
        }
    }

    pub fn select_item(&mut self, id: &str) -> Result<(), String> {
        if let Some(current_selected_item) = self.items.values_mut().find(|item| item.selected) {
            current_selected_item.selected = false;
        }
    
        if let Some(new_selected_item) = self.items.get_mut(id) {
            new_selected_item.selected = true;
            Ok(())
        } else {
            Err("Item not found".to_string())
        }
    }

    pub fn selected_item(&self) -> Option<&ServerUrlItem> {
        self.items.values().find(|item| item.selected)
    }

    pub fn selected_item_url(&self) -> Option<Url> {
        self.selected_item().map(|item| item.url.clone())
    }
}
