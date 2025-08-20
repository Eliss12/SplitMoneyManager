#[derive(Debug, Clone)]
pub struct Expenses {
    id: i32,
    username: String,
    amount: f32,
    group_name: String,
    due_date: String,
}

impl Expenses {
    pub fn new(id: i32, username: String, amount: f32, group_name: String, due_date: String) -> Self {
        Self {id, username, amount, group_name, due_date}
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn amount(&self) -> f32 {
        self.amount
    }

    pub fn group_name(&self) -> &str {
        &self.group_name
    }

    pub fn due_date(&self) -> &str {
        &self.due_date
    }
}