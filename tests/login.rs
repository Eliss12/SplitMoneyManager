use rusqlite::{Connection};
use split_money_manager::*;

#[test]

fn login() {
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
    let user = db::login_user(&conn, "ivan@example.com", "12345678").unwrap();
    assert_eq!(user.username(), "Ivan");
    assert_eq!(user.email(), "ivan@example.com");
    assert_eq!(user.id(),1);

    let fake_user = db::login_user(&conn,"maria@example.com","12345678").unwrap_err();
    assert_eq!(fake_user, "Не е намерен потребител");
}