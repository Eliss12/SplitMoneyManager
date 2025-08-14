use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;


use crate::db::{init_db, register_user, login_user};
use crate::models::User;

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

}

#[derive(Debug)]
pub enum ServerResponse {
    Ok(String),
    Err(String),
    User(User),
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

            }
        }
    });

    (tx_cmd, rx_resp)
}
