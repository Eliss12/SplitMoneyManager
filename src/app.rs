use std::sync::mpsc::{Sender, Receiver};
use crate::backend::{start_backend, ServerCommand, ServerResponse};
use crate::user::User;
use eframe::{egui, App, Frame};
use crate::group::Group;

#[derive(Clone)]
pub enum Screen {
    Login,
    Register,
    MainApp(User),
    CreateGroup(i32),
    MyGroups(i32),
    AddExp(i32, i32),
}

pub struct MyApp {
    tx_cmd: Sender<ServerCommand>,
    rx_resp: Receiver<ServerResponse>,
    pub screen: Screen,

    login_email: String,
    login_password: String,

    reg_username: String,
    reg_email: String,
    reg_password: String,

    group_name: String,
    search_query: String,
    search_results: Vec<User>,
    selected_users: Vec<i32>,

    my_groups: Vec<Group>,

    exp_amount: f32,
    exp_description: String,
    exp_due_date: String,

    loading: bool,
    success_message: Option<String>,
    success_time: Option<std::time::Instant>,
    error_message: Option<String>,
    error_time: Option<std::time::Instant>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx_cmd, rx_resp) = start_backend();

        Self {
            tx_cmd,
            rx_resp,
            screen: Screen::Login,
            login_email: String::new(),
            login_password: String::new(),
            reg_username: String::new(),
            reg_email: String::new(),
            reg_password: String::new(),
            group_name: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            selected_users: Vec::new(),
            my_groups: Vec::new(),
            exp_amount: 0.0,
            exp_description: String::new(),
            exp_due_date: String::new(),
            loading: false,
            success_message: None,
            success_time: None,
            error_message: None,
            error_time: None,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        self.process_backend_responses(ctx);
        let current_screen = self.screen.clone();

        match current_screen {
            Screen::MainApp(user) => self.show_main_app(ctx, &user),
            Screen::Login => self.show_login(ctx),
            Screen::Register => self.show_register(ctx),
            Screen::CreateGroup(user_id) => self.show_create_group(ctx, user_id),
            Screen::MyGroups(user_id) => self.show_my_groups(ctx, user_id),
            Screen::AddExp(user_id, group_id) => self.show_add_expenses(ctx, user_id, group_id),
        }
    }
}


impl MyApp {
    fn process_backend_responses(&mut self, ctx: &egui::Context) {
        while let Ok(response) = self.rx_resp.try_recv() {
            match response {
                ServerResponse::Ok(msg) => {
                    self.success_message = Some(msg);
                    self.success_time = Some(std::time::Instant::now());
                    self.loading = false;
                }
                ServerResponse::Err(msg) => {
                    self.error_message = Some(msg);
                    self.error_time = Some(std::time::Instant::now());
                    self.loading = false;
                }
                ServerResponse::User(user) => {
                    self.screen = Screen::MainApp(user);
                    self.loading = false;
                }
                ServerResponse::Users(users) => {
                    self.search_results = users;
                    self.loading = false;
                }
                ServerResponse::Groups(groups) => {
                    self.my_groups = groups;
                    self.loading = false;
                }

            }
            ctx.request_repaint();
        }

    }

    fn update_messages(&mut self, ctx: &egui::Context) {

        if let Some(start) = self.success_time {
            if start.elapsed().as_secs() > 5 {
                self.success_message = None;
                self.success_time = None;
            }
        }

        if let Some(start) = self.error_time {
            if start.elapsed().as_secs() > 5 {
                self.error_message = None;
                self.error_time = None;
            }
        }

        ctx.request_repaint();
    }

    fn show_messages(&self, ui: &mut egui::Ui) {
        if let Some(msg) = &self.success_message {
            ui.colored_label(egui::Color32::GREEN, msg);
        }
        if let Some(msg) = &self.error_message {
            ui.colored_label(egui::Color32::RED, msg);
        }
    }

    fn show_login(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Вход");

            ui.label("Имейл:");
            ui.text_edit_singleline(&mut self.login_email);

            ui.label("Парола:");
            ui.add(
                egui::TextEdit::singleline(&mut self.login_password)
                    .password(true)
            );

            if ui.button("Вход").clicked() {
                let _ = self.tx_cmd.send(ServerCommand::Login {
                    email: std::mem::take(&mut self.login_email),
                    password: std::mem::take(&mut self.login_password),
                });
            }

            if ui.button("Нямаш акаунт? Регистрирай се").clicked() {
                self.screen = Screen::Register;
            }

            self.process_backend_responses(ctx);
            self.update_messages(ctx);
            self.show_messages(ui);
        });
    }

    fn show_register(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Регистрация");

            ui.label("Потребителско име:");
            ui.text_edit_singleline(&mut self.reg_username);

            ui.label("Имейл:");
            ui.text_edit_singleline(&mut self.reg_email);

            ui.label("Парола:");
            ui.add(
                egui::TextEdit::singleline(&mut self.reg_password)
                    .password(true)
            );

            if ui.button("Създай акаунт").clicked() {

                let _ = self.tx_cmd.send(ServerCommand::Register {
                    username: std::mem::take(&mut self.reg_username),
                    email: std::mem::take(&mut self.reg_email),
                    password: std::mem::take(&mut self.reg_password),
                });
                self.loading = true;

            }

            if ui.button("Вече имаш акаунт? Влез").clicked() {
                self.screen = Screen::Login;
            }

            self.process_backend_responses(ctx);

            if self.loading {
                ui.separator();
                ui.label("Моля изчакайте...");
            }

            self.update_messages(ctx);
            self.show_messages(ui);
        });
    }

    fn show_main_app(&mut self, ctx: &egui::Context, user: &User) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("Добре дошли, {}!", user.username()));
                if ui.button("Изход").clicked() {
                    self.screen = Screen::Login;
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Моите групи ").clicked() {
                    self.screen = Screen::MyGroups(user.id());
                }
                if ui.button("Създай група").clicked() {
                    self.screen = Screen::CreateGroup(user.id());
                }
            });

        });
    }

    fn show_create_group(&mut self, ctx: &egui::Context, owner_id: i32) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Създаване на група");

            ui.add_enabled_ui(!self.loading, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Име на групата:");
                    ui.text_edit_singleline(&mut self.group_name);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.search_query);
                    if ui.button("Търси").clicked() {
                        let _ = self.tx_cmd.send(ServerCommand::SearchUsers {
                            query: std::mem::take(&mut self.search_query),
                        });
                        self.loading = true;
                        self.process_backend_responses(ctx);
                    }
                });

                ui.separator();

                for user in &self.search_results {
                    let mut checked = self.selected_users.contains(&user.id());

                    ui.horizontal(|ui| {

                        if ui.checkbox(&mut checked, "").changed() {
                            if checked {
                                if !self.selected_users.contains(&user.id()) {
                                    self.selected_users.push(user.id());
                                }
                            } else {
                                self.selected_users.retain(|&id| id != user.id());
                            }
                        }

                        ui.label(format!("{} ({})", user.username(), user.email()));

                        if user.is_loyal_payer() {
                            ui.colored_label(egui::Color32::GOLD, "⭐");
                        }
                    });
                }


                ui.separator();

                if ui.button("Създай групата").clicked() {
                    if !self.group_name.trim().is_empty() && !self.selected_users.is_empty() {
                        let _ = self.tx_cmd.send(ServerCommand::CreateGroup {
                            name: std::mem::take(&mut self.group_name),
                            owner_id,
                            members: std::mem::take(&mut self.selected_users),
                        });
                        self.loading = true;
                        self.process_backend_responses(ctx);
                    } else {
                        self.error_message = Some(
                            "Моля въведете име и изберете поне един член.".to_string(),
                        );
                        self.error_time = Some(std::time::Instant::now());
                    }
                }

                if ui.button("Назад").clicked() {
                    let _ = self.tx_cmd.send(ServerCommand::GetUser { owner_id });
                    self.loading = true;
                    self.process_backend_responses(ctx);
                }
            });

            if self.loading {
                ui.separator();
                ui.label("Моля изчакайте...");
            }

            self.update_messages(ctx);
            self.show_messages(ui);
        });
    }

    fn show_my_groups(&mut self, ctx: &egui::Context, user_id: i32) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Моите групи");

            ui.add_enabled_ui(!self.loading, |ui| {
                let _ = self.tx_cmd.send(ServerCommand::ShowGroups {
                            user_id,
                        });
                //self.loading = true;
                self.process_backend_responses(ctx);

                for group in &self.my_groups {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}", group.groupname()));
                        if ui.button("Добави разход").clicked() {
                            self.screen = Screen::AddExp(user_id, group.id());
                        }
                    });

                    ui.separator();
                }

                if ui.button("Назад").clicked() {
                    let owner_id = user_id;
                    let _ = self.tx_cmd.send(ServerCommand::GetUser { owner_id });
                    self.loading = true;
                    self.process_backend_responses(ctx);
                }
            });

            if self.loading {
                ui.separator();
                ui.label("Моля изчакайте...");
            }

            self.update_messages(ctx);
            self.show_messages(ui);

        });
    }

    fn show_add_expenses(&mut self, ctx: &egui::Context, user_id: i32, group_id: i32) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Добавяне на разход");

            let mut amount_str = self.exp_amount.to_string();
            ui.label("Сума:");
            ui.text_edit_singleline(&mut amount_str);
            if let Ok(parsed) = amount_str.parse::<f32>() {
                self.exp_amount = parsed;
            }

            ui.label("Описание:");
            ui.text_edit_singleline(&mut self.exp_description);

            ui.label("Крайна дата за изплащане:");
            ui.text_edit_singleline(&mut self.exp_due_date);


            if ui.button("Добави разход").clicked() {

                let _ = self.tx_cmd.send(ServerCommand::AddExpenses {
                    user_id,
                    group_id,
                    amount: std::mem::take(&mut self.exp_amount),
                    description: std::mem::take(&mut self.exp_description),
                    due_date: std::mem::take(&mut self.exp_due_date),
                });
                self.loading = true;

            }

            if ui.button("Назад").clicked() {
                self.screen = Screen::MyGroups(user_id);
            }

            self.process_backend_responses(ctx);

            if self.loading {
                ui.separator();
                ui.label("Моля изчакайте...");
            }

            self.update_messages(ctx);
            self.show_messages(ui);
        });
    }
}




