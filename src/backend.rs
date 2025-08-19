use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;


use crate::db::{init_db, register_user, login_user, create_group, search_users, get_user_by_id, get_user_groups};
use crate::group::Group;
use crate::user::User;

#[derive(Debug)]
pub enum ServerCommand {
    Register {
        username: String,
        email: String,
        password: String,
    },
    Login {
        email: String,
        password: String,
    },
    SearchUsers { query: String },
    CreateGroup { name: String, owner_id: i32, members: Vec<i32> },
    GetUser {owner_id: i32},
    ShowGroups {user_id: i32},
}

#[derive(Debug)]
pub enum ServerResponse {
    Ok(String),
    Err(String),
    User(User),
    Users(Vec<User>),
    Groups(Vec<Group>),
}

pub fn start_backend() -> (Sender<ServerCommand>, Receiver<ServerResponse>) {
    let (tx_cmd, rx_cmd) = mpsc::channel::<ServerCommand>();
    let (tx_resp, rx_resp) = mpsc::channel::<ServerResponse>();

    thread::spawn(move || {
        let conn = init_db().expect("Failed to initialize DB");

        while let Ok(cmd) = rx_cmd.recv() {
            match cmd {
                ServerCommand::Register { username, email, password } => {
                    match register_user(&conn, &username, &email, &password) {
                        Ok(_) => tx_resp.send(ServerResponse::Ok("Успешна регистрация!".into())).unwrap(),
                        Err(e) => tx_resp.send(ServerResponse::Err(e)).unwrap(),
                    }
                }
                ServerCommand::Login { email, password } => {
                    match login_user(&conn, &email, &password) {
                        Ok(user) => {
                            tx_resp.send(ServerResponse::User(user)).unwrap();
                        }
                        Err(e) => tx_resp.send(ServerResponse::Err(e)).unwrap(),
                    }
                }
                ServerCommand::SearchUsers { query } => {
                    match search_users(&conn, &query) {
                        Ok(users) => tx_resp.send(ServerResponse::Users(users)).unwrap(),
                        Err(e) => tx_resp.send(ServerResponse::Err(e)).unwrap(),
                    }
                }
                ServerCommand::CreateGroup { name, owner_id, members } => {
                    match create_group(&conn, &name, owner_id, &members) {
                        Ok(_) => tx_resp.send(ServerResponse::Ok("Групата е създадена успешно!".into())).unwrap(),
                        Err(e) => tx_resp.send(ServerResponse::Err(e)).unwrap(),
                    }
                }
                ServerCommand::GetUser { owner_id } => {
                    match get_user_by_id(&conn, owner_id) {
                        Ok(user) => tx_resp.send(ServerResponse::User(user)).unwrap(),
                        Err(e) => tx_resp.send(ServerResponse::Err(e)).unwrap(),
                    }
                }
                ServerCommand::ShowGroups { user_id } => {
                    match get_user_groups(&conn, user_id) {
                        Ok(groups) => tx_resp.send(ServerResponse::Groups(groups)).unwrap(),
                        Err(e) => tx_resp.send(ServerResponse::Err(e)).unwrap(),
                    }
                }

            }
        }
    });

    (tx_cmd, rx_resp)
}
