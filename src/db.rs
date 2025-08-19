use rusqlite::{params, Connection, Result};
use argon2::{Argon2, PasswordHasher};
use password_hash::{SaltString, PasswordHasher as _, PasswordHash, PasswordVerifier};
use rand_core::OsRng;
use regex::Regex;
use crate::user::{User};
use crate::group::Group;

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

    let username_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?1)",
            params![username],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if username_exists {
        return Err("Вече има регистриран потребител с това потребителско име.".to_string());
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

pub fn create_group(conn: &Connection, name: &str, owner_id: i32, members: &[i32]) -> std::result::Result<(), String> {
    conn.execute(
        "INSERT INTO groups (name, owner_id) VALUES (?1, ?2)",
        params![name, owner_id],
    )
        .map_err(|e| e.to_string())?;

    let group_id = conn.last_insert_rowid();

    for &user_id in members {
        conn.execute(
            "INSERT INTO group_members (group_id, user_id) VALUES (?1, ?2)",
            (group_id, user_id),
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn search_users(conn: &Connection, query: &str) -> std::result::Result<Vec<User>, String> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn
        .prepare("SELECT id, username, email, loyal_payer FROM users WHERE username LIKE ?1 OR email LIKE ?1")
        .map_err(|e| e.to_string())?;

    let users = stmt
        .query_map([pattern], |row| {
            Ok(User::from_loyal_payer (
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(users)
}

pub fn get_user_by_id(conn: &Connection, user_id: i32) -> std::result::Result<User, String> {
    let mut stmt = conn
        .prepare("SELECT id, username, email FROM users WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query(params![user_id]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(User::from_id (
            row.get(0).unwrap(),
            row.get(1).unwrap(),
            row.get(2).unwrap(),

        ))
    } else {
        Err("Не е намерен потребител.".to_string())
    }

}


pub fn get_user_groups(conn: &Connection, user_id: i32) -> std::result::Result<Vec<Group>, String> {
    let mut stmt = conn
        .prepare("SELECT g.id, g.name, g.owner_id
             FROM groups g
             JOIN group_members gm ON g.id = gm.group_id
             WHERE gm.user_id = ?1
             UNION
             SELECT g1.id, g1.name, g1.owner_id
             FROM groups g1
             WHERE g1.owner_id = ?1",)
        .map_err(|e| e.to_string())?;

    let groups = stmt
        .query_map( [user_id], |row| {
            Ok(Group::new (
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(groups)
}

pub fn add_expenses(conn: &Connection, group_id: i32, payer_id: i32, amount: f32, description: &str, due_date: &str) -> std::result::Result<(), String> {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    if !re.is_match(due_date) {
        return Err("Невалиден формат на дата. Използвайте YYYY-MM-DD.".to_string());
    }

    if amount < 0.0 {
        return Err("Сумата трябва да е положително число.".to_string());
    }

    conn.execute(
        "INSERT INTO expenses (group_id, payer_id, description, amount, due_date)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![group_id, payer_id, description, amount, due_date],
    )
        .map_err(|e| format!("Грешка в базата данни: {}", e))?;

    Ok(())
}