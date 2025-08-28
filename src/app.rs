use std::sync::mpsc::{Sender, Receiver};
use crate::backend::{start_backend, ServerCommand, ServerResponse};
use crate::user::User;
use eframe::{egui, App, Frame};
use egui::{Frame as UiFrame, RichText, Color32, Margin};
use crate::group::Group;
use crate::expenses::Expenses;
use crate::notification::Notification;
use std::sync::mpsc::TryRecvError;

#[derive(Clone)]
pub enum Screen {
    Login,
    Register,
    MainApp(User),
    CreateGroup(i32),
    MyGroups(i32),
    AddExp(i32, i32),
    MyDebtsOrCredits(i32, bool),
    MyNotifications(i32),
}

pub enum Action {
    MainApp(User),
    Login,
    Register,
    CreateGroup(i32),
    MyGroups(i32),
    AddExp(i32, i32),
    MyDebtsOrCredits(i32, bool),
    MyNotifications(i32),
}

pub struct LoginState {
    login_email: String,
    login_password: String,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            login_email: String::new(),
            login_password: String::new(),
        }
    }
}

pub struct RegistrationState {
    reg_username: String,
    reg_email: String,
    reg_password: String,
}
impl Default for RegistrationState {
    fn default() -> Self {
        Self {
            reg_username: String::new(),
            reg_email: String::new(),
            reg_password: String::new(),
        }

    }
}

pub struct GroupState {
    group_name: String,
    search_query: String,
    search_results: Vec<User>,
    selected_users: Vec<i32>,
    group_loading: bool,
    my_groups: Vec<Group>,
}

impl Default for GroupState {
    fn default() -> Self {
        Self {
            group_name: String::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            selected_users: Vec::new(),
            group_loading: false,
            my_groups: Vec::new(),
        }
    }
}

pub struct ExpensesState {
    exp_amount: f32,
    exp_description: String,
    exp_due_date: String,
    my_debts_or_credits: Vec<Expenses>,
    debts_or_credits_loading: bool,
}

impl Default for ExpensesState {
    fn default() -> Self {
        Self {
            exp_amount: 0.0,
            exp_description: String::new(),
            exp_due_date: String::new(),
            my_debts_or_credits: Vec::new(),
            debts_or_credits_loading: false,
        }
    }
}

pub struct NotificationState {
    notifications: Vec<Notification>,
    notification_loading: bool,
}

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            notifications: Vec::new(),
            notification_loading: false,
        }
    }
}



pub struct MyApp {
    tx_cmd: Sender<ServerCommand>,
    rx_resp: Receiver<ServerResponse>,
    pub screen: Screen,
    login: LoginState,
    registration: RegistrationState,
    group_state: GroupState,
    expenses: ExpensesState,
    notifications_state: NotificationState,
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
            login: LoginState::default(),
            registration: RegistrationState::default(),
            group_state: GroupState::default(),
            expenses: ExpensesState::default(),
            notifications_state: NotificationState::default(),
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

        let action = {
            match &self.screen {
                Screen::MainApp(user) => Action::MainApp(user.clone()),
                Screen::Login => Action::Login,
                Screen::Register => Action::Register,
                Screen::CreateGroup(user_id) => Action::CreateGroup(*user_id),
                Screen::MyGroups(user_id) => Action::MyGroups(*user_id),
                Screen::AddExp(user_id, group_id) => Action::AddExp(*user_id, *group_id),
                Screen::MyDebtsOrCredits(user_id, is_debt) => {
                    Action::MyDebtsOrCredits(*user_id, *is_debt)
                }
                Screen::MyNotifications(user_id) => Action::MyNotifications(*user_id),
            }
        };

        match action {
            Action::MainApp(user) => self.show_main_app(ctx, &user),
            Action::Login => self.show_login(ctx),
            Action::Register => self.show_register(ctx),
            Action::CreateGroup(user_id) => self.show_create_group(ctx, user_id),
            Action::MyGroups(user_id) => self.show_my_groups(ctx, user_id),
            Action::AddExp(user_id, group_id) => self.show_add_expenses(ctx, user_id, group_id),
            Action::MyDebtsOrCredits(user_id, is_debt) => {
                self.show_my_debts_or_credits(ctx, user_id, is_debt)
            }
            Action::MyNotifications(user_id) => self.show_my_notifications(ctx, user_id),
        }
    }
}


impl MyApp {
    fn process_backend_responses(&mut self, ctx: &egui::Context) {
        loop {
            match self.rx_resp.try_recv() {
                Ok(response) => {
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
                            self.group_state.search_results = users;
                            self.loading = false;
                        }
                        ServerResponse::Groups(groups) => {
                            self.group_state.my_groups = groups;
                            self.loading = false;
                        }
                        ServerResponse::Expenses(expenses) => {
                            self.expenses.my_debts_or_credits = expenses;
                            self.loading = false;
                        }
                        ServerResponse::Notifications(notifications) => {
                            self.notifications_state.notifications = notifications;
                            self.loading = false;
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    self.error_message = Some("Server disconnected".to_string());
                }

            }
            ctx.request_repaint();
        }

    }

    fn update_messages(&mut self, ctx: &egui::Context) {

        if let Some(start) = self.success_time {
            if start.elapsed().as_secs() > 3 {
                self.success_message = None;
                self.success_time = None;
            }
        }

        if let Some(start) = self.error_time {
            if start.elapsed().as_secs() > 3 {
                self.error_message = None;
                self.error_time = None;
            }
        }

        ctx.request_repaint();
    }

    fn show_messages(&self, ui: &mut egui::Ui) {
        if let Some(msg) = &self.success_message {
            ui.colored_label(Color32::GREEN, msg);
        }
        if let Some(msg) = &self.error_message {
            ui.colored_label(Color32::RED, msg);
        }
    }

    fn show_login(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Вход");

            ui.label("Имейл:");
            ui.text_edit_singleline(&mut self.login.login_email);

            ui.label("Парола:");
            ui.add(
                egui::TextEdit::singleline(&mut self.login.login_password)
                    .password(true)
            );

            ui.add_space(10.0);
            ui.add_enabled_ui(!self.loading, |ui| {
                if ui.add(
                    egui::Button::new(
                        RichText::new("Вход").color(Color32::WHITE)
                    ).fill(Color32::from_rgb(30, 60, 150))
                ).clicked() {
                    if let Err(e) = self.tx_cmd.send(ServerCommand::Login {
                        email: std::mem::take(&mut self.login.login_email),
                        password: std::mem::take(&mut self.login.login_password),
                    }) {
                        self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                    }
                    self.loading = true;
                }

                ui.add_space(5.0);
                if ui.add(
                    egui::Button::new(
                        RichText::new("Нямаш акаунт? Регистрирай се").color(Color32::WHITE)
                    ).fill(Color32::from_rgb(0, 102, 0))
                ).clicked() {
                    self.screen = Screen::Register;
                }

                self.process_backend_responses(ctx);
            });
            if self.loading {
                ui.separator();
                ui.label("Моля изчакайте...");
            }

            self.update_messages(ctx);
            self.show_messages(ui);
        });
    }

    fn show_register(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Регистрация");

            ui.label("Потребителско име:");
            ui.text_edit_singleline(&mut self.registration.reg_username);

            ui.label("Имейл:");
            ui.text_edit_singleline(&mut self.registration.reg_email);

            ui.label("Парола:");
            ui.add(
                egui::TextEdit::singleline(&mut self.registration.reg_password)
                    .password(true)
            );

            ui.add_space(10.0);
            ui.add_enabled_ui(!self.loading, |ui| {
                if ui.add(
                    egui::Button::new(
                        RichText::new("Създай акаунт").color(Color32::WHITE)
                    ).fill(Color32::from_rgb(30, 60, 150))
                ).clicked() {
                    if self.registration.reg_email.trim().is_empty() || self.registration.reg_password.trim().is_empty() || self.registration.reg_username.trim().is_empty() {
                        self.error_message = Some(
                            "Моля попълнете всички полета.".to_string(),
                        );
                        self.error_time = Some(std::time::Instant::now());
                    } else {
                        if let Err(e) = self.tx_cmd.send(ServerCommand::Register {
                            username: std::mem::take(&mut self.registration.reg_username),
                            email: std::mem::take(&mut self.registration.reg_email),
                            password: std::mem::take(&mut self.registration.reg_password),
                        }) {
                            self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                        }
                        self.loading = true;
                    }
                }

                ui.add_space(5.0);
                if ui.add(
                    egui::Button::new(
                        RichText::new("Вече имаш акаунт? Влез").color(Color32::WHITE)
                    ).fill(Color32::from_rgb(0, 102, 0))
                ).clicked() {
                    self.screen = Screen::Login;
                }

                self.process_backend_responses(ctx);
            });

            if self.loading {
                ui.separator();
                ui.label("Моля изчакайте...");
            }

            self.update_messages(ctx);
            self.show_messages(ui);
        });
    }

    fn show_main_app(&mut self, ctx: &egui::Context, user: &User) {
        use egui::Color32;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(format!("Добре дошли, {}!", user.username()));
                        ui.add_space(20.0);

                        ui.group(|ui| {
                            ui.vertical_centered(|ui| {
                                let button_size = [200.0, 30.0];

                                if ui.add_sized(button_size, egui::Button::new("Моите групи").fill(Color32::from_rgb(30, 60, 150))).clicked() {
                                    self.screen = Screen::MyGroups(user.id());
                                }
                                ui.add_space(5.0);

                                if ui.add_sized(button_size, egui::Button::new("Създай група").fill(Color32::from_rgb(0, 102, 102))).clicked() {
                                    self.screen = Screen::CreateGroup(user.id());
                                }
                                ui.add_space(5.0);

                                if ui.add_sized(button_size, egui::Button::new("Моите дългове").fill(Color32::from_rgb(0, 102, 0))).clicked() {
                                    self.screen = Screen::MyDebtsOrCredits(user.id(), true);
                                }
                                ui.add_space(5.0);

                                if ui.add_sized(button_size, egui::Button::new("Моите вземания").fill(Color32::from_rgb(102, 102, 0))).clicked() {
                                    self.screen = Screen::MyDebtsOrCredits(user.id(), false);
                                }
                                ui.add_space(5.0);

                                if ui.add_sized(button_size, egui::Button::new("Известия").fill(Color32::from_rgb(153, 76, 0))).clicked() {
                                    self.screen = Screen::MyNotifications(user.id());
                                }

                                ui.add_space(5.0);
                                if ui.add_sized(button_size, egui::Button::new("Изход").fill(Color32::from_rgb(153, 0, 0))).clicked() {
                                    self.screen = Screen::Login;
                                }
                            });
                        });
                    });
           });
        });
    }


    fn show_my_notifications(&mut self, ctx: &egui::Context, user_id: i32) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.heading("Известия");
                    ui.add_space(20.0);
                    if !self.notifications_state.notification_loading {
                        self.notifications_state.notifications = Vec::new();
                        if let Err(e) = self.tx_cmd.send(ServerCommand::ShowNotification {
                            user_id,
                        }) {
                            self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                        }
                        self.notifications_state.notification_loading = true;
                        self.loading = true;
                        self.process_backend_responses(ctx);
                    }

                    for notification in &self.notifications_state.notifications {
                        UiFrame::group(ui.style())
                            .fill(Color32::from_rgb(0, 102, 204))
                            .rounding(6.0)
                            .inner_margin(Margin::same(6.0))
                            .show(ui, |ui| {
                                ui.label(RichText::new(notification.message())
                                    .color(Color32::WHITE));
                            });
                        ui.add_space(10.0);
                    }

                    ui.add_enabled_ui(!self.loading, |ui| {
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Назад").color(Color32::WHITE)
                            ).fill(Color32::from_rgb(30, 60, 150))
                        ).clicked() {
                            let owner_id = user_id;
                            if let Err(e) = self.tx_cmd.send(ServerCommand::GetUser { owner_id }){
                                self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                            }
                            self.process_backend_responses(ctx);
                            self.notifications_state.notification_loading = false;
                            self.loading = true;
                        }
                    });
                    if self.loading {
                        ui.separator();
                        ui.label("Моля изчакайте...");
                    }

                    self.update_messages(ctx);
                    self.show_messages(ui);
            });
        });
    }

    fn show_create_group(&mut self, ctx: &egui::Context, owner_id: i32) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.heading("Създаване на група");

                    ui.add_enabled_ui(!self.loading, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Име на групата:");
                            ui.text_edit_singleline(&mut self.group_state.group_name);
                        });

                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.group_state.search_query);
                            if ui.add(
                                egui::Button::new(
                                    RichText::new("Търси").color(Color32::WHITE)
                                ).fill(Color32::from_rgb(0, 102, 0))
                            ).clicked() {
                                if let Err(e) = self.tx_cmd.send(ServerCommand::SearchUsers {
                                    query: std::mem::take(&mut self.group_state.search_query),
                                }){
                                    self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                                }
                                self.loading = true;
                                self.process_backend_responses(ctx);
                            }
                        });

                        ui.separator();

                        for user in &self.group_state.search_results {
                            let mut checked = self.group_state.selected_users.contains(&user.id());

                            ui.horizontal(|ui| {

                                if ui.checkbox(&mut checked, "").changed() {
                                    if checked {
                                        if !self.group_state.selected_users.contains(&user.id()) {
                                            self.group_state.selected_users.push(user.id());
                                        }
                                    }
                                    else {
                                        self.group_state.selected_users.retain(|&id| id != user.id());
                                    }
                                }

                                ui.label(format!("{} ({})", user.username(), user.email()));

                                if user.is_loyal_payer() {
                                    ui.colored_label(Color32::GOLD, "⭐");
                                }
                            });
                        }

                        if !self.group_state.selected_users.contains(&owner_id) {
                            self.group_state.selected_users.push(owner_id);
                        }

                        ui.separator();
                        ui.add_space(10.0);
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Създай групата").color(Color32::WHITE)
                            ).fill(Color32::from_rgb(30, 60, 150))
                        ).clicked() {
                            if !self.group_state.group_name.trim().is_empty() && !self.group_state.selected_users.is_empty() {
                                if let Err(e) = self.tx_cmd.send(ServerCommand::CreateGroup {
                                    name: std::mem::take(&mut self.group_state.group_name),
                                    owner_id,
                                    members: std::mem::take(&mut self.group_state.selected_users),
                                }){
                                    self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                                }
                                self.loading = true;
                                self.process_backend_responses(ctx);
                            }
                            else {
                                self.error_message = Some(
                                    "Моля въведете име и изберете поне един член.".to_string(),
                                );
                                self.error_time = Some(std::time::Instant::now());
                            }
                        }

                        ui.add_space(5.0);
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Назад").color(Color32::WHITE)
                            ).fill(Color32::from_rgb(0, 102, 0))
                        ).clicked() {
                            if let Err(e) = self.tx_cmd.send(ServerCommand::GetUser { owner_id }){
                                self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                            }
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
        });
    }

    fn show_my_groups(&mut self, ctx: &egui::Context, user_id: i32) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.heading("Моите групи");
                    ui.add_space(10.0);

                    if !self.group_state.group_loading {
                        self.group_state.my_groups = Vec::new();
                        if let Err(e) = self.tx_cmd.send(ServerCommand::ShowGroups {
                            user_id,
                        }) {
                            self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                        }
                        self.group_state.group_loading = true;
                        self.loading = true;
                        self.process_backend_responses(ctx);
                    }

                    for group in &self.group_state.my_groups {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}", group.groupname()));
                            if ui.add(
                                egui::Button::new(
                                    RichText::new("Добави разход").color(Color32::WHITE)
                                ).fill(Color32::from_rgb(30, 60, 150))
                            ).clicked() {
                                self.screen = Screen::AddExp(user_id, group.id());
                            }
                        });

                        ui.separator();
                    }

                    ui.add_enabled_ui(!self.loading, |ui| {
                        ui.add_space(10.0);
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Назад").color(Color32::WHITE)
                            ).fill(Color32::from_rgb(0, 102, 0))
                        ).clicked() {
                            let owner_id = user_id;
                            if let Err(e) = self.tx_cmd.send(ServerCommand::GetUser { owner_id }){
                                self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                            }
                            self.group_state.group_loading = false;
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
        });
    }

    fn show_add_expenses(&mut self, ctx: &egui::Context, user_id: i32, group_id: i32) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Добавяне на разход");

            ui.add_enabled_ui(!self.loading, |ui| {
                let mut amount_str = self.expenses.exp_amount.to_string();
                ui.label("Сума:");
                ui.text_edit_singleline(&mut amount_str);
                if amount_str.is_empty() {
                    self.error_message = Some(
                        "Моля въведете сума.".to_string(),
                    );
                    self.error_time = Some(std::time::Instant::now());
                }
                if let Ok(parsed) = amount_str.parse::<f32>() {
                    self.expenses.exp_amount = parsed;
                }

                ui.label("Описание:");
                ui.text_edit_singleline(&mut self.expenses.exp_description);

                ui.label("Крайна дата за изплащане:");
                ui.text_edit_singleline(&mut self.expenses.exp_due_date);

                ui.add_space(10.0);


                if ui.add(
                    egui::Button::new(
                        RichText::new("Добави разход").color(Color32::WHITE)
                    ).fill(Color32::from_rgb(30, 60, 150))
                ).clicked() {
                    if let Err(e) = self.tx_cmd.send(ServerCommand::AddExpenses {
                        user_id,
                        group_id,
                        amount: std::mem::take(&mut self.expenses.exp_amount),
                        description: std::mem::take(&mut self.expenses.exp_description),
                        due_date: std::mem::take(&mut self.expenses.exp_due_date),
                    }) {
                        self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                    }
                    self.loading = true;
                }

                ui.add_space(5.0);
                if ui.add(
                    egui::Button::new(
                        RichText::new("Назад").color(Color32::WHITE)
                    ).fill(Color32::from_rgb(0, 102, 0))
                ).clicked() {
                    self.screen = Screen::MyGroups(user_id);
                }

                self.process_backend_responses(ctx);
            });

            if self.loading {
                ui.separator();
                ui.label("Моля изчакайте...");
            }

            self.update_messages(ctx);
            self.show_messages(ui);
        });
    }

    fn show_my_debts_or_credits(&mut self, ctx: &egui::Context, user_id: i32, is_debt: bool) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let heading;
                    let user;
                    if is_debt {
                        heading = "Моите дългове";
                        user = "Кредитор";
                    } else {
                        heading = "Моите вземания";
                        user = "Длъжник";
                    }
                    ui.heading(heading);

                    if !self.expenses.debts_or_credits_loading {
                        self.expenses.my_debts_or_credits = Vec::new();
                        if let Err(e) = self.tx_cmd.send(ServerCommand::ShowDebtsOrCredits {
                            user_id,
                            is_debt,
                        }){
                            self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                        }
                        self.expenses.debts_or_credits_loading = true;
                        self.loading = true;
                        self.process_backend_responses(ctx);
                    }

                    for debt_or_credit in &self.expenses.my_debts_or_credits {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "{}: {}\nСума: {:.2} лв.\nОписание: {}\nКрайна дата: {}\nГрупа: {}",
                                user,
                                debt_or_credit.username(),
                                debt_or_credit.amount(),
                                debt_or_credit.description(),
                                debt_or_credit.due_date(),
                                debt_or_credit.group_name()
                            ));
                            ui.separator();
                            if ui.add(
                                egui::Button::new(
                                    RichText::new("Потвърждаване на плащане").color(Color32::WHITE)
                                ).fill(Color32::from_rgb(30, 60, 150))
                            ).clicked() {
                                let debt_id = debt_or_credit.id();
                                if let Err(e) = self.tx_cmd.send(ServerCommand::PaymentConfirmation {
                                    user_id,
                                    debt_id,
                                }) {
                                    self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                                }
                                self.expenses.debts_or_credits_loading = false;
                            }
                        });

                        ui.separator();
                    }

                    ui.add_enabled_ui(!self.loading, |ui| {
                        ui.add_space(10.0);
                        if ui.add(
                            egui::Button::new(
                                RichText::new("Назад").color(Color32::WHITE)
                            ).fill(Color32::from_rgb(0, 102, 0))
                        ).clicked() {
                            let owner_id = user_id;
                            if let Err(e) = self.tx_cmd.send(ServerCommand::GetUser { owner_id }) {
                                self.error_message = Some(format!("Неуспешно изпращане: {}", e));
                            }
                            self.expenses.debts_or_credits_loading = false;
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
        });
    }
}




