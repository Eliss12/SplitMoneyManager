use rusqlite::Connection;
use split_money_manager::*;

#[test]

fn get_user_notifications() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            on_time_payments INTEGER DEFAULT 0,
            loyal_payer INTEGER DEFAULT 0
        );

        CREATE TABLE debts (
            id INTEGER PRIMARY KEY,
            from_id INTEGER NOT NULL,
            to_id INTEGER NOT NULL,
            amount REAL NOT NULL,
            due_date TEXT,
            confirmed_by_debtor INTEGER DEFAULT 0,
            confirmed_by_creditor INTEGER DEFAULT 0,
            settled INTEGER DEFAULT 0
        );

        CREATE TABLE notifications (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id INTEGER NOT NULL,
        message TEXT NOT NULL,
        created_at TEXT DEFAULT CURRENT_TIMESTAMP,
        shown BOOLEAN DEFAULT 0
        );

    ").unwrap();

    conn.execute("INSERT INTO users (id, username) VALUES (1, 'Ivan')", []).unwrap();
    conn.execute("INSERT INTO users (id, username) VALUES (2, 'Maria')", []).unwrap();

    conn.execute("INSERT INTO debts (id, from_id, to_id, amount, due_date) VALUES(1, 1, 2, 100, '2025-01-01')", [] ).unwrap();

    let result1 = db::get_user_notifications(&conn, 1).unwrap();
    let result2 = db::get_user_notifications(&conn, 2);

    assert_eq!(result1[0].message(), "Имате просрочен дълг от 100.00 лв. със срок 2025-01-01");
    assert_eq!(result2.unwrap_err(), "Нямате известия!");

    let shown: i32 = conn.query_row(
        "SELECT shown FROM notifications WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(shown, 1);

}



