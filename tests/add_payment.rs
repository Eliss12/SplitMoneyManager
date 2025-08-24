use rusqlite::Connection;
use split_money_manager::*;

#[test]

fn add_payment() {
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
            confirmed_by_debtor INTEGER DEFAULT 0,
            confirmed_by_creditor INTEGER DEFAULT 0,
            settled BOOLEAN DEFAULT 0
        );

    ").unwrap();

    conn.execute("INSERT INTO users (id, username) VALUES (1, 'Ivan')", []).unwrap();
    conn.execute("INSERT INTO users (id, username) VALUES (2, 'Maria')", []).unwrap();
    conn.execute("INSERT INTO users (id, username) VALUES (3, 'Georgi')", []).unwrap();
    conn.execute("INSERT INTO groups (id, name, owner_id) VALUES (1, 'gr1', 1)", []).unwrap();
    conn.execute("INSERT INTO group_members (group_id, user_id) VALUES (1, 1)", []).unwrap();
    conn.execute("INSERT INTO group_members (group_id, user_id) VALUES (1, 2)", []).unwrap();
    conn.execute("INSERT INTO group_members (group_id, user_id) VALUES (1, 3)", []).unwrap();

    let _ = db::add_expenses(&conn, 1, 1, 300.0, "Балони за рожден ден", "2026-01-01").unwrap();
    let _ = db::add_expenses(&conn, 2, 1,600.0, "Торта за рожден ден", "2026-03-03").unwrap();

    let amount_first_to_second: f32 = conn.query_row(
        "SELECT amount FROM debts WHERE from_id = 1 AND to_id = 2",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(amount_first_to_second, 100.0);

    let amount_third_to_first: f32= conn.query_row(
        "SELECT amount FROM debts WHERE from_id = 3 AND to_id = 1",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(amount_third_to_first, 100.0);

    let amount_third_to_second: f32 = conn.query_row(
        "SELECT amount FROM debts WHERE from_id = 3 AND to_id = 2",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(amount_third_to_second, 200.0);
}