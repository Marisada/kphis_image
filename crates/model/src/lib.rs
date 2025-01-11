use serde_derive::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageData {
    pub image_id: u32,
    // 0 is no group
    pub foreign_id: u32,
    pub path: String,
    pub title: Option<String>,
    pub user: String,
}

impl ImageData {
    pub fn from_rc_ref(rc_ref: &Rc<Self>) -> Self {
        Self {
            image_id: rc_ref.image_id,
            foreign_id: rc_ref.foreign_id,
            path: rc_ref.path.clone(),
            title: rc_ref.title.clone(),
            user: rc_ref.user.clone(),
        }
    }
}

impl PartialEq for ImageData {
    fn eq(&self, other: &Self) -> bool {
        self.image_id == other.image_id
    }
}