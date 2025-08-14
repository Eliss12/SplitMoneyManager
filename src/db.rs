use rusqlite::{params, Connection, Result};
use argon2::{Argon2, PasswordHasher};
use password_hash::{SaltString, PasswordHasher as _, PasswordHash, PasswordVerifier};
use rand_core::OsRng;
use regex::Regex;
use crate::user::{User};

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("database.db")?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            on_time_payments INTEGER DEFAULT 0,
            loyal_payer BOOLEAN DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            owner_id INTEGER NOT NULL,
            FOREIGN KEY(owner_id) REFERENCES users(id)
        );

        CREATE TABLE IF NOT EXISTS group_members (
            group_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            PRIMARY KEY (group_id, user_id),
            FOREIGN KEY(group_id) REFERENCES groups(id),
            FOREIGN KEY(user_id) REFERENCES users(id)
        );

        CREATE TABLE IF NOT EXISTS expenses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER NOT NULL,
            payer_id INTEGER NOT NULL,
            description TEXT NOT NULL,
            amount REAL NOT NULL,
            due_date TEXT NOT NULL,
            paid BOOLEAN DEFAULT 0,
            FOREIGN KEY(group_id) REFERENCES groups(id),
            FOREIGN KEY(payer_id) REFERENCES users(id)
        );
        "
    )?;

    Ok(conn)
}

pub fn register_user(conn: &Connection, username: &str, email: &str, password: &str) -> std::result::Result<(), String> {
    let email_regex = Regex::new(r"^[\w.-]+@[\w.-]+\.\w+$").unwrap();
    if !email_regex.is_match(email) {
        return Err("Невалиден имейл.".to_string());
    }

    if password.len() < 8 {
        return Err("Паролата трябва да е поне 8 символа.".to_string());
    }

    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = ?1)",
            params![email],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        return Err("Вече има регистриран потребител с този имейл.".to_string());
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| "Проблем при хеширането на паролата.")?
        .to_string();

    conn.execute(
        "INSERT INTO users (username, email, password_hash) VALUES (?1, ?2, ?3)",
        params![username, email, password_hash],
    )
        .map_err(|e| format!("Грешка в базата данни: {}", e))?;

    Ok(())
}

pub fn login_user(conn: &Connection, email: &str, password: &str) -> std::result::Result<User, String> {
    let mut stmt = conn
        .prepare("SELECT id, username, email, password_hash, on_time_payments, loyal_payer FROM users WHERE email = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query([email]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let id: i32 = row.get(0).unwrap();
        let username: String = row.get(1).unwrap();
        let email: String = row.get(2).unwrap();
        let stored_hash: String = row.get(3).unwrap();
        let on_time_payments: i32 = row.get(4).unwrap();
        let loyal_payer: bool = row.get(5).unwrap();

        let parsed_hash = PasswordHash::new(&stored_hash)
            .map_err(|_| "Invalid password hash format".to_string())?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| "Невалидна парола".to_string())?;

        Ok(User::new(id, username, email, stored_hash, on_time_payments, loyal_payer ))
    } else {
        Err("Не е намерен потребител".to_string())
    }
}
