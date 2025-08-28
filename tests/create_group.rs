use rusqlite::{Connection, params};
use split_money_manager::*;

#[test]

fn create_group() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("
        CREATE TABLE users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
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

    conn.execute("INSERT INTO users (id, username, email) VALUES (1, 'Ivan', 'ivan@example.com')", []).unwrap();
    conn.execute("INSERT INTO users (id, username, email) VALUES (2, 'Maria', 'maria@example.com')", []).unwrap();
    conn.execute("INSERT INTO users (id, username, email) VALUES (3, 'Georgi', 'georgi@example.com')", []).unwrap();
    conn.execute("INSERT INTO users (id, username, email) VALUES (4, 'Petar', 'petar@example.com')", []).unwrap();

    let _ = db::create_group(&conn, "super group", 1, &[2, 3, 4]);

    let mut stmt = conn
        .prepare("SELECT id, name, owner_id FROM groups")
        .unwrap();

    let groups: Vec<(i32, String, i32)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .map(|e| e.unwrap())
        .collect();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].1, "super group");
    assert_eq!(groups[0].2, 1);

    let mut stmt = conn
        .prepare("SELECT user_id FROM group_members WHERE group_id = ?1")
        .unwrap();
    let saved_members: Vec<i32> = stmt
        .query_map(params![1], |row| row.get(0))
        .unwrap()
        .map(|e| e.unwrap())
        .collect();

    assert_eq!(saved_members, &[2, 3, 4]);


}