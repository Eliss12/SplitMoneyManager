use rusqlite::Connection;
use split_money_manager::*;

#[test]

fn get_user_groups() {
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
    ").unwrap();

    conn.execute("INSERT INTO users (id, username) VALUES (1, 'Ivan')", []).unwrap();
    conn.execute("INSERT INTO users (id, username) VALUES (2, 'Maria')", []).unwrap();
    conn.execute("INSERT INTO users (id, username) VALUES (3, 'Georgi')", []).unwrap();
    conn.execute("INSERT INTO groups (id, name, owner_id) VALUES (1, 'gr1', 1)", []).unwrap();
    conn.execute("INSERT INTO group_members (group_id, user_id) VALUES (1, 1)", []).unwrap();
    conn.execute("INSERT INTO group_members (group_id, user_id) VALUES (1, 2)", []).unwrap();

    let result1 = db::get_user_groups(&conn, 1).unwrap();
    let result2 = db::get_user_groups(&conn, 2).unwrap();
    let result3 = db::get_user_groups(&conn, 3).unwrap_err();

    assert_eq!(result1[0].groupname(), "gr1");
    assert_eq!(result2[0].groupname(), "gr1");
    assert_eq!(result3, "Нямате групи!");
}