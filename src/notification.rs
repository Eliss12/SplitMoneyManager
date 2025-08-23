#[derive(Debug, Clone)]
pub struct Notification {
    id: i32,
    message: String,
}

impl Notification {
    pub fn new(id: i32, message: String) -> Notification {
        Notification { id, message }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}