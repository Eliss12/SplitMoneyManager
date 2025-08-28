use rusqlite::{Connection};
use split_money_manager::*;

#[test]

fn registration() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            on_time_payments INTEGER DEFAULT 0,
            loyal_payer INTEGER DEFAULT 0
        );

    ").unwrap();

    conn.execute("INSERT INTO users (id, username, email, password_hash) VALUES (1, 'Ivan', 'ivan@example.com', '$argon2id$v=19$m=19456,t=2,p=1$X4JSodT9nxYkf0+4x2L9kw$wJ71QeNRhQleHDtWkL8BqrLzrPyARUQ9H11Ax3KCUdU')", []).unwrap();
    let fake_email = db::register_user(&conn, "username", "username", "12345678").unwrap_err();
    assert_eq!(fake_email, "Невалиден имейл.");

    let fake_password = db::register_user(&conn, "username", "username@example.com", "1234567").unwrap_err();
    assert_eq!(fake_password, "Паролата трябва да е поне 8 символа.");

    let user_email_exists = db::register_user(&conn, "username", "ivan@example.com", "12345678").unwrap_err();
    assert_eq!(user_email_exists, "Вече има регистриран потребител с този имейл.");

    let username_exists = db::register_user(&conn, "Ivan", "username@example.com", "12345678").unwrap_err();
    assert_eq!(username_exists, "Вече има регистриран потребител с това потребителско име.");

    let _ = db::register_user(&conn, "username", "username@example.com", "12345678");
    let username: String = conn.query_row(
        "SELECT username FROM users WHERE email = 'username@example.com' ",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(username, "username");

}