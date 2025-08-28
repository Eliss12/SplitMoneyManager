#[derive(Debug, Clone)]
pub struct User {
    id: i32,
    username: String,
    email: String,
    loyal_payer: bool,
}

impl User {

    pub fn new(id: i32, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
            loyal_payer: false,
        }
    }

    pub fn from_loyal_payer(id: i32, username: String, email: String, loyal_payer: bool) -> Self {
        Self{
            id,
            username,
            email,
            loyal_payer
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn is_loyal_payer(&self) -> bool {
        self.loyal_payer
    }

}