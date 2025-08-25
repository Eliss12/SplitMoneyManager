#[derive(Debug, Clone)]
pub struct Group {
    id: i32,
    groupname: String,
}

impl Group {
    pub fn new(id: i32, groupname: String) -> Group {
        Self { id, groupname }
    }

    pub fn groupname(&self) -> &str {
        &self.groupname
    }

    pub fn id(&self) -> i32 {
        self.id
    }
}