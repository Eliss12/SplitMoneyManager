use rusqlite::{params, Connection, Result};
use argon2::{Argon2, PasswordHasher};
use password_hash::{SaltString, PasswordHash, PasswordVerifier};
use rand_core::OsRng;
use regex::Regex;
use crate::user::{User};
use crate::group::Group;
use rusqlite::OptionalExtension;
use crate::expenses::Expenses;
use crate::notification::Notification;

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("Database.db")?;

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

        CREATE TABLE IF NOT EXISTS debts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            from_id INTEGER NOT NULL,
            to_id INTEGER NOT NULL,
            amount REAL NOT NULL,
            group_id INTEGER NOT NULL,
            due_date TEXT NOT NULL,
            description TEXT NOT NULL,
            confirmed_by_debtor BOOLEAN DEFAULT 0,
            confirmed_by_creditor BOOLEAN DEFAULT 0,
            settled BOOLEAN DEFAULT 0,
            FOREIGN KEY(from_id) REFERENCES users(id),
            FOREIGN KEY(to_id) REFERENCES users(id),
            FOREIGN KEY(group_id) REFERENCES groups(id)
        );

        CREATE TABLE IF NOT EXISTS notifications (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        message TEXT NOT NULL,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        shown BOOLEAN DEFAULT 0,
        FOREIGN KEY(user_id) REFERENCES users(id)
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


        let parsed_hash = PasswordHash::new(&stored_hash)
            .map_err(|_| "Invalid password hash format".to_string())?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| "Невалидна парола".to_string())?;

        Ok(User::new(id, username, email ))
    }
    else {
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

    if users.is_empty() {
        return Err("Няма такъв потребител!".to_string());
    }

    Ok(users)
}

pub fn get_user_by_id(conn: &Connection, user_id: i32) -> std::result::Result<User, String> {
    let mut stmt = conn
        .prepare("SELECT id, username, email FROM users WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let mut rows = stmt.query(params![user_id]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        Ok(User::new (
            row.get(0).unwrap(),
            row.get(1).unwrap(),
            row.get(2).unwrap(),

        ))
    }
    else {
        Err("Не е намерен потребител.".to_string())
    }

}


pub fn get_user_groups(conn: &Connection, user_id: i32) -> std::result::Result<Vec<Group>, String> {
    let mut stmt = conn
        .prepare("SELECT g.id, g.name, g.owner_id
             FROM groups g
             JOIN group_members gm ON g.id = gm.group_id
             WHERE gm.user_id = ?1",)
        .map_err(|e| e.to_string())?;

    let groups = stmt
        .query_map( [user_id], |row| {
            Ok(Group::new (
                row.get(0)?,
                row.get(1)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if groups.is_empty() {
        return Err("Нямате групи!".to_string());
    }

    Ok(groups)
}

pub fn add_or_update_debt(
    conn: &Connection,
    from_id: i32,
    to_id: i32,
    group_id: i32,
    amount: f32,
    due_date: &str,
    description: &str,
) -> Result<(), String> {

    let mut stmt = conn.prepare(
        "SELECT id, amount, confirmed_by_debtor, confirmed_by_creditor
         FROM debts
         WHERE from_id = ?1 AND to_id = ?2 AND group_id = ?3 AND settled = 0"
    ).map_err(|e| e.to_string())?;

    let existing: Option<(i32, f32, bool, bool)> = stmt.query_row(
        params![from_id, to_id, group_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).optional().map_err(|e| e.to_string())?;

    if let Some((debt_id, old_amount, confirmed_debtor, confirmed_creditor)) = existing {

        if !confirmed_debtor && !confirmed_creditor {

            let new_amount = old_amount + amount;
            conn.execute(
                "UPDATE debts SET amount = ?1, due_date = ?2, description = ?3 WHERE id = ?4",
                params![new_amount, due_date, description, debt_id],
            ).map_err(|e| e.to_string())?;
        }
        else {

            conn.execute(
                "INSERT INTO debts (from_id, to_id, group_id, amount, due_date, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![from_id, to_id, group_id, amount, due_date, description],
            ).map_err(|e| e.to_string())?;
        }
    }
    else {

        let mut stmt2 = conn.prepare(
            "SELECT id, amount, confirmed_by_debtor, confirmed_by_creditor
             FROM debts
             WHERE from_id = ?1 AND to_id = ?2 AND group_id = ?3 AND settled = 0"
        ).map_err(|e| e.to_string())?;

        let reverse: Option<(i32, f32, bool, bool)> = stmt2.query_row(
            params![to_id, from_id, group_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        ).optional().map_err(|e| e.to_string())?;

        if let Some((rev_id, rev_amount, confirmed_debtor, confirmed_creditor)) = reverse {
            if !confirmed_debtor && !confirmed_creditor {

                if amount > rev_amount {
                    let diff = amount - rev_amount;

                    conn.execute("DELETE FROM debts WHERE id = ?1", params![rev_id])
                        .map_err(|e| e.to_string())?;
                    conn.execute(
                        "INSERT INTO debts (from_id, to_id, group_id, amount, due_date, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![from_id, to_id, group_id, diff, due_date, description],
                    ).map_err(|e| e.to_string())?;
                }
                else if amount < rev_amount {
                    let diff = rev_amount - amount;

                    conn.execute(
                        "UPDATE debts SET amount = ?1, due_date = ?2, description = ?3 WHERE id = ?4",
                        params![diff, due_date, description, rev_id],
                    ).map_err(|e| e.to_string())?;
                }
                else {

                    conn.execute("DELETE FROM debts WHERE id = ?1", params![rev_id])
                        .map_err(|e| e.to_string())?;
                }
            }
            else {

                conn.execute(
                    "INSERT INTO debts (from_id, to_id, group_id, amount, due_date, description)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![from_id, to_id, group_id, amount, due_date, description],
                ).map_err(|e| e.to_string())?;
            }
        }
        else {

            conn.execute(
                "INSERT INTO debts (from_id, to_id, group_id, amount, due_date, description)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![from_id, to_id, group_id, amount, due_date, description],
            ).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}


pub fn add_expenses(conn: &Connection, payer_id: i32, group_id: i32, amount: f32, description: &str, due_date: &str) -> std::result::Result<(), String> {
    let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    if !re.is_match(due_date) && !due_date.is_empty() {
        return Err("Невалиден формат на дата. Използвайте YYYY-MM-DD.".to_string());
    }

    if amount <= 0.0 {
        return Err("Сумата трябва да е положително число.".to_string());
    }

    let mut stmt = conn
        .prepare("SELECT user_id FROM group_members WHERE group_id = ?1")
        .map_err(|e| e.to_string())?;

    let members: Vec<i32> = stmt
        .query_map(params![group_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<i32>, _>>()
        .map_err(|e| e.to_string())?;

    let share = amount / members.len() as f32;

    for member_id in members {
        if member_id == payer_id {
            continue;
        }

        add_or_update_debt(conn, member_id, payer_id, group_id, share, due_date, description)?;
    }

    Ok(())
}

pub fn get_user_debts_or_credits(conn: &Connection, user_id: i32, is_debt: bool) -> Result<Vec<Expenses>, String> {
    let condition2 = if is_debt { "d.from_id = ?" } else { "d.to_id = ?" };
    let condition1 = if is_debt { "d.to_id" } else { "d.from_id" };

    let query = format!(
        "SELECT d.id, u.username, d.amount, g.name, d.due_date, d.description
         FROM debts d
         JOIN users u ON {} = u.id
         JOIN groups g ON d.group_id = g.id
         WHERE {}
         AND d.settled = 0
         ORDER BY d.due_date ASC;",
        condition1, condition2
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let expenses = stmt
        .query_map([user_id], |row| {
            Ok(Expenses::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if expenses.is_empty() {
        return Err("Нямате дългове или вземания!".to_string());
    }

    Ok(expenses)
}

pub fn payment_confirmation(conn: &Connection, user_id: i32, debt_id: i32) -> std::result::Result<String, String> {
    let mut stmt = conn.prepare(
        "SELECT from_id, to_id, confirmed_by_debtor, confirmed_by_creditor
         FROM debts WHERE id = ?1 AND settled = 0"
    ).map_err(|e| e.to_string())?;

    let (from_id, to_id, mut debtor_conf, mut creditor_conf): (i32, i32, bool, bool) =
        stmt.query_row([debt_id], |row| {
            Ok(
                (row.get(0)?,
                 row.get(1)?,
                 row.get(2)?,
                 row.get(3)?)
            )
        }).map_err(|e| e.to_string())?;

    if user_id == from_id {
        debtor_conf = true;
    }
    else if user_id == to_id {
        creditor_conf = true;
    }
    else {
        return Err("Потребителят не е участник в този дълг.".to_string());
    }

    if debtor_conf && creditor_conf {
        conn.execute(
            "UPDATE debts
             SET confirmed_by_debtor = 1,
                 confirmed_by_creditor = 1,
                 settled = 1
             WHERE id = ?1",
            [debt_id],
        ).map_err(|e| e.to_string())?;

        let mut stmt_check = conn.prepare(
            "SELECT CASE
                        WHEN due_date IS NULL THEN -1
                        WHEN due_date >= date('now') THEN 1
                        ELSE 0
                    END
                FROM debts
                WHERE id = ?1;"
        ).map_err(|e| e.to_string())?;

        let on_time: i32 = stmt_check.query_row([debt_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        if on_time == 1 {
            conn.execute(
                "UPDATE users
                 SET on_time_payments = on_time_payments + 1
                 WHERE id = ?1",
                [from_id],
            ).map_err(|e| e.to_string())?;

            let mut stmt_user = conn.prepare(
                "SELECT on_time_payments FROM users WHERE id = ?1"
            ).map_err(|e| e.to_string())?;
            let payments: i32 = stmt_user.query_row([from_id], |row| row.get(0))
                .map_err(|e| e.to_string())?;

            if payments >= 20 {
                conn.execute(
                    "UPDATE users SET loyal_payer = 1 WHERE id = ?1",
                    [from_id],
                ).map_err(|e| e.to_string())?;
            }
        }
        else if on_time == 0 {

            conn.execute(
                "UPDATE users
                 SET loyal_payer = 0
                 WHERE id = ?1",
                [from_id],
            ).map_err(|e| e.to_string())?;
        }
        Ok("Дългът е напълно изплатен и приключен.".to_string())
    }
    else {
        conn.execute(
            "UPDATE debts
             SET confirmed_by_debtor = ?1,
                 confirmed_by_creditor = ?2
             WHERE id = ?3",
            params![debtor_conf, creditor_conf, debt_id],
        ).map_err(|e| e.to_string())?;
        Ok("Потвърдено. Очаква се другата страна да потвърди.".to_string())
    }
}

pub fn get_user_notifications(conn: &Connection, user_id: i32) -> Result<Vec<Notification>, String> {
    let mut stmt = conn.prepare(
        "SELECT amount, due_date
         FROM debts
         WHERE from_id = ?1
           AND settled = 0
           AND due_date IS NOT NULL
           AND due_date != ''
           AND due_date < date('now')"
    ).map_err(|e| e.to_string())?;

    let overdue: Vec<(f32, String)> = stmt.query_map([user_id], |row| {
        Ok(
            (row.get(0)?,
             row.get(1)?,)
        )
    }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for (amount, due_date) in overdue {
        let message = format!("Имате просрочен дълг от {:.2} лв. със срок {}", amount, due_date);
        conn.execute(
            "INSERT INTO notifications (user_id, message)
             VALUES (?1, ?2)",params![user_id, message]
        ).map_err(|e| e.to_string())?;
    }

    conn.execute(
        "DELETE FROM notifications WHERE user_id = ?1 AND shown = 1",
        params![user_id],
    ).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, message FROM notifications
         WHERE user_id = ?1 AND shown = 0
         ORDER BY created_at ASC"
    ).map_err(|e| e.to_string())?;

    let notes = stmt.query_map([user_id], |row| {
        Ok(Notification::new
            (row.get(0)?,
             row.get(1)?)
        )
    }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    for note in &notes {
        conn.execute("UPDATE notifications SET shown = 1 WHERE id = ?1", [note.id()]).map_err(|e| e.to_string())?;
    }

    if notes.is_empty() {
        return Err("Нямате известия!".to_string());
    }

    Ok(notes)
}

