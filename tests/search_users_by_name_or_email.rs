use rusqlite::Connection;
use split_money_manager::*;

#[test]

fn show_searched_users() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            on_time_payments INTEGER DEFAULT 0,
            loyal_payer INTEGER DEFAULT 0
        );

    ").unwrap();

    conn.execute("INSERT INTO users (id, username, email) VALUES (1, 'Ivan', 'ivan@example.com')", []).unwrap();
    conn.execute("INSERT INTO users (id, username, email) VALUES (2, 'Maria', 'maria@example.com')", []).unwrap();

    let result1 = db::search_users(&conn, "Ivan").unwrap();
    assert_eq!(result1[0].id(), 1);
    let result2 = db::search_users(&conn, "maria@example.com").unwrap();
    assert_eq!(result2[0].id(), 2);
    let result3 = db::search_users(&conn, "Georgi").unwrap_err();
    assert_eq!(result3, "Няма такъв потребител!");
}