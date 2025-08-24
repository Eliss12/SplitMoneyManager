use rusqlite::Connection;
use split_money_manager::*;

#[test]

fn payment_confirmation_on_time_payments_20() {
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
            settled BOOLEAN DEFAULT 0
        );
    ").unwrap();

    conn.execute("INSERT INTO users (id, username, on_time_payments) VALUES (1, 'Ivan', 19)", []).unwrap();
    conn.execute("INSERT INTO users (id, username) VALUES (2, 'Maria')", []).unwrap();

    conn.execute(
        "INSERT INTO debts (id, from_id, to_id, amount, due_date)
         VALUES (1, 1, 2, 50.0, date('now', '+1 day'))",
        [],
    ).unwrap();

    let result1 = db::payment_confirmation(&conn, 1, 1).unwrap();
    assert_eq!(result1, "Потвърдено. Очаква се другата страна да потвърди.");

    let result2 = db::payment_confirmation(&conn, 2, 1).unwrap();
    assert_eq!(result2, "Дългът е напълно изплатен и приключен.");

    let settled: i32 = conn.query_row(
        "SELECT settled FROM debts WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap();

    assert_eq!(settled, 1);

    let on_time: i32 = conn.query_row(
        "SELECT on_time_payments FROM users WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap();

    assert_eq!(on_time, 20);

    let loyal_payer: i32 = conn.query_row(
        "SELECT loyal_payer FROM users WHERE id = 1",
        [],
        |row| row.get(0),
    ).unwrap();

    assert_eq!(loyal_payer, 1);
}
