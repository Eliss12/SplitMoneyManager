use rusqlite::Connection;
use split_money_manager::*;

#[test]

fn get_user_debt_or_credit() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            on_time_payments INTEGER DEFAULT 0,
            loyal_payer INTEGER DEFAULT 0
        );

        CREATE TABLE groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            owner_id INTEGER NOT NULL
        );

        CREATE TABLE group_members (
            group_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            PRIMARY KEY (group_id, user_id)
        );

        CREATE TABLE debts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            from_id INTEGER NOT NULL,
            to_id INTEGER NOT NULL,
            amount REAL NOT NULL,
            group_id INTEGER NOT NULL,
            due_date TEXT NOT NULL,
            description TEXT NOT NULL,
            settled BOOLEAN DEFAULT 0
        );

    ").unwrap();

    conn.execute("INSERT INTO users (id, username) VALUES (1, 'Ivan')", []).unwrap();
    conn.execute("INSERT INTO users (id, username) VALUES (2, 'Maria')", []).unwrap();
    conn.execute("INSERT INTO groups (id, name, owner_id) VALUES (1, 'gr1', 1)", []).unwrap();
    conn.execute("INSERT INTO group_members (group_id, user_id) VALUES (1, 1)", []).unwrap();
    conn.execute("INSERT INTO group_members (group_id, user_id) VALUES (1, 2)", []).unwrap();
    conn.execute("INSERT INTO debts (id, from_id, to_id, amount, group_id, due_date, description) VALUES(1, 1, 2, 100, 1, '2026-01-01', 'Балони за рожден ден')", [] ).unwrap();

    let result = db::get_user_debts_or_credits(&conn, 1, true).unwrap();
    assert_eq!(result[0].username(), "Maria");
    assert_eq!(result[0].group_name(), "gr1");
    assert_eq!(result[0].due_date(), "2026-01-01");
    assert_eq!(result[0].amount(), 100.0);
    assert_eq!(result[0].description(), "Балони за рожден ден");
}