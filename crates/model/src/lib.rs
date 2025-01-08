use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct ImageData {
    pub image_id: u32,
    pub foreign_id: u32,
    pub path: String,
    pub user: String,
}
