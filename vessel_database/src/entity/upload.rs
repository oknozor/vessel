use crate::entity::Entity;
use crate::UPLOAD_QUEUE;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UploadEntity {
    pub file_name: String,
    pub user_name: String,
    pub ticket: u32,
    pub place_in_queue: u32,
}

impl UploadEntity {
    pub fn new(file_name: String, user_name: String, ticket: u32) -> UploadEntity {
        let mut upload_queue = UPLOAD_QUEUE.lock().unwrap();
        *upload_queue += 1;
        let place_in_queue = *upload_queue;

        UploadEntity {
            file_name,
            user_name,
            ticket,
            place_in_queue,
        }
    }
}

impl Entity for UploadEntity {
    fn get_key(&self) -> Vec<u8> {
        format!("{}@{}", self.user_name, self.file_name)
            .as_bytes()
            .to_vec()
    }

    const COLLECTION: &'static str = "uploads";
}
