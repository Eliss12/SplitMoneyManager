#[derive(Debug, Clone)]
pub struct Group {
    id: i32,
    groupname: String,
    owner_id: i32,
}

impl Group {
    pub fn new(id: i32, groupname: String, owner_id: i32) -> Group {
        Self { id, groupname, owner_id }
    }

    pub fn groupname(&self) -> &str {
        &self.groupname
    }
}