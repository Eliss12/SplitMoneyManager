use std::sync::mpsc::{Sender, Receiver};
use crate::backend::{start_backend, ServerCommand, ServerResponse};
use crate::models::User;
use eframe::{egui, App, Frame};

#[derive(Clone)]
pub enum Screen {
    Login,
    Register,
    MainApp(User),
}

pub struct MyApp {
    tx_cmd: Sender<ServerCommand>,
    rx_resp: Receiver<ServerResponse>,
    pub screen: Screen,

    // Полета за вход
    login_email: String,
    login_password: String,

    // Полета за регистрация
    reg_username: String,
    reg_email: String,
    reg_password: String,
    success_message: Option<String>,
    error_message: Option<String>,
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

            success_message: None,
            error_message: None,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let current_screen = self.screen.clone();

        match current_screen {
            Screen::MainApp(user) => self.show_main_app(ctx, &user),
            Screen::Login => self.show_login(ctx),
            Screen::Register => self.show_register(ctx),
        }
    }
}


impl MyApp {
    fn process_backend_responses(&mut self) {
        while let Ok(response) = self.rx_resp.try_recv() {
            match response {
                ServerResponse::Ok(msg) => self.success_message = Some(msg),
                ServerResponse::Err(msg) => self.error_message = Some(msg),
                ServerResponse::User(user) => self.screen = Screen::MainApp(user),
            }
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
                    email: self.login_email.clone(),
                    password: self.login_password.clone(),
                });
            }

            if ui.button("Нямаш акаунт? Регистрирай се").clicked() {
                self.screen = Screen::Register;
            }

            self.process_backend_responses();

            if let Some(msg) = &self.success_message {
                ui.colored_label(egui::Color32::GREEN, msg);
            }
            if let Some(msg) = &self.error_message {
                ui.colored_label(egui::Color32::RED, msg);
            }
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
                    username: self.reg_username.clone(),
                    email: self.reg_email.clone(),
                    password: self.reg_password.clone(),
                });

            }

            if ui.button("Вече имаш акаунт? Влез").clicked() {
                self.screen = Screen::Login;
            }

            self.process_backend_responses();

            if let Some(msg) = &self.success_message {
                ui.colored_label(egui::Color32::GREEN, msg);
            }
            if let Some(msg) = &self.error_message {
                ui.colored_label(egui::Color32::RED, msg);
            }
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


        });
    }
}
