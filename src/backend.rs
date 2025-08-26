use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use crate::db::{init_db, register_user, login_user, create_group, search_users, get_user_by_id, get_user_groups, add_expenses, get_user_debts_or_credits, payment_confirmation, get_user_notifications};
use crate::group::Group;
use crate::user::User;
use crate::expenses::Expenses;
use crate::notification::Notification;
use std::sync::mpsc::RecvError;

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
    AddExpenses { user_id: i32, group_id: i32, amount: f32, description: String, due_date: String },
    ShowDebtsOrCredits { user_id: i32 , is_debt: bool},
    PaymentConfirmation { user_id: i32, debt_id: i32 },
    ShowNotification { user_id: i32 },
}

#[derive(Debug)]
pub enum ServerResponse {
    Ok(String),
    Err(String),
    User(User),
    Users(Vec<User>),
    Groups(Vec<Group>),
    Expenses(Vec<Expenses>),
    Notifications(Vec<Notification>),
}

pub fn start_backend() -> (Sender<ServerCommand>, Receiver<ServerResponse>) {
    let (tx_cmd, rx_cmd) = mpsc::channel::<ServerCommand>();
    let (tx_resp, rx_resp) = mpsc::channel::<ServerResponse>();

    thread::spawn(move || {
        let conn = init_db().expect("Failed to initialize DB");

        loop {
            match rx_cmd.recv() {
                Ok(cmd) => {
                    let response = match cmd {
                        ServerCommand::Register { username, email, password } => {
                            register_user(&conn, &username, &email, &password)
                                .map(|_| ServerResponse::Ok("Успешна регистрация!".into()))
                                .unwrap_or_else(ServerResponse::Err)
                        }
                        ServerCommand::Login { email, password } => {
                            login_user(&conn, &email, &password)
                                .map(ServerResponse::User)
                                .unwrap_or_else(ServerResponse::Err)
                        }
                        ServerCommand::SearchUsers { query } => {
                            search_users(&conn, &query)
                                .map(ServerResponse::Users)
                                .unwrap_or_else(ServerResponse::Err)
                        }
                        ServerCommand::CreateGroup { name, owner_id, members } => {
                            create_group(&conn, &name, owner_id, &members)
                                .map(|_| ServerResponse::Ok("Групата е създадена успешно!".into()))
                                .unwrap_or_else(ServerResponse::Err)
                        }
                        ServerCommand::GetUser { owner_id } => {
                            get_user_by_id(&conn, owner_id)
                                .map(ServerResponse::User)
                                .unwrap_or_else(ServerResponse::Err)
                        }
                        ServerCommand::ShowGroups { user_id } => {
                            get_user_groups(&conn, user_id)
                                .map(ServerResponse::Groups)
                                .unwrap_or_else(ServerResponse::Err)
                        }
                        ServerCommand::AddExpenses { user_id, group_id, amount, description, due_date } => {
                            add_expenses(&conn, user_id, group_id, amount, &description, &due_date)
                                .map(|_| ServerResponse::Ok("Успешно добавихте разход!".into()))
                                .unwrap_or_else(ServerResponse::Err)
                        }
                        ServerCommand::ShowDebtsOrCredits { user_id, is_debt } => {
                            get_user_debts_or_credits(&conn, user_id, is_debt)
                                .map(ServerResponse::Expenses)
                                .unwrap_or_else(ServerResponse::Err)
                        }
                        ServerCommand::PaymentConfirmation { user_id, debt_id } => {
                            payment_confirmation(&conn, user_id, debt_id)
                                .map(ServerResponse::Ok)
                                .unwrap_or_else(ServerResponse::Err)
                        }
                        ServerCommand::ShowNotification { user_id } => {
                            get_user_notifications(&conn, user_id)
                                .map(ServerResponse::Notifications)
                                .unwrap_or_else(ServerResponse::Err)
                        }
                    };

                    if let Err(e) = tx_resp.send(response) {
                        eprintln!("Failed to send response: {}", e);
                    }
                }
                Err(RecvError) => {
                    eprintln!("Command channel disconnected. Shutting down server loop.");
                    break;
                }
            }
        }


    });

    (tx_cmd, rx_resp)
}
