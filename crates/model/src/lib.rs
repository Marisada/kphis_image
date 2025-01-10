use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageData {
    pub image_id: u32,
    // 0 is no group
    pub foreign_id: u32,
    pub path: String,
    pub title: Option<String>,
    pub user: String,
}

impl PartialEq for ImageData {
    fn eq(&self, other: &Self) -> bool {
        self.image_id == other.image_id
    }
}