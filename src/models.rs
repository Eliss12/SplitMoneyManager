#[derive(Debug, Clone)]
pub struct User {
    id: i32,
    username: String,
    email: String,
    password_hash: String,
    on_time_payments: i32,
    loyal_payer: bool,
}

impl User {

    pub fn new(id: i32, username: String, email: String, password_hash: String, on_time_payments: i32, loyal_payer: bool) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            on_time_payments,
            loyal_payer,
        }
    }

    // Getter-и
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn on_time_payments(&self) -> i32 {
        self.on_time_payments
    }

    pub fn is_loyal_payer(&self) -> bool {
        self.loyal_payer
    }


    pub fn add_on_time_payment(&mut self) {
        self.on_time_payments += 1;
        if self.on_time_payments >= 20 {
            self.loyal_payer = true;
        }
    }

    pub fn set_password_hash(&mut self, new_hash: String) {
        self.password_hash = new_hash;
    }
}