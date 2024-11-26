use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use directories::BaseDirs;
use std::path::PathBuf;
use reqwest::{Client, cookie::Jar};
use std::sync::{Arc, RwLock};
use egui::PopupCloseBehavior;
use reqwest::cookie::CookieStore;
use reqwest::header::HeaderValue;
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, Clone)]
struct AuthData {
    username: String,
    password: String,
    cookies: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Redeem {
    name: String,
    api_name: String,
    requires_input: bool,
}

#[derive(Serialize, Deserialize)]
struct Config {
    redeems: Vec<Redeem>,
}

struct ClonkApp {
    config: Config,
    auth_data: AuthData,
    input_text: Arc<RwLock<String>>,
    runtime: Runtime,
    status_message: Arc<RwLock<String>>,
}

impl ClonkApp {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let base_dirs = BaseDirs::new().unwrap();
        let home = base_dirs.home_dir();

        let mut config_path = PathBuf::from(home);
        config_path.push(".clonk/redeems.toml");
        if !config_path.exists() {
            fs::write(&config_path, include_str!("../config/redeems.toml"))?;
        }
        let config_str = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_str)?;

        let mut auth_path = PathBuf::from(home);
        auth_path.push(".clonk/auth");
        let auth_str = fs::read_to_string(auth_path)?;
        let auth_data: AuthData = serde_json::from_str(&auth_str)?;

        Ok(ClonkApp {
            config,
            auth_data,
            input_text: Arc::new(RwLock::new(String::new())),
            runtime: Runtime::new()?,
            status_message: Arc::new(RwLock::new(String::new())),
        })
    }

    async fn redeem(auth_data: &AuthData, redeem: &Redeem, input: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(jar.clone())
            .build()?;

        // Set cookies
        jar.set_cookies(&mut std::iter::once(&HeaderValue::from_str(&auth_data.cookies).unwrap()), &"https://secure.colonq.computer".parse().unwrap());


        let form = reqwest::multipart::Form::new()
            .text("name", redeem.api_name.clone())
            .text("input", input.unwrap_or_else(|| "undefined".to_string()));

        let response = client
            .post("https://secure.colonq.computer/api/redeem")
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to redeem: {}", response.status()).into());
        }

        Ok(())
    }
}

impl eframe::App for ClonkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("the men:u");
                ui.label("exclusively for Our Esteemed Patrons");
                ui.add_space(20.0);
            });

            if !self.status_message.read().unwrap().is_empty() {
                ui.label(
                    self.status_message.
                        read()
                        .unwrap()
                        .as_str(),
                );
                ui.add_space(10.0);
            }


            let available_width = ui.available_width();
            let spacing = 10.0;
            let button_size = 200.0;
            let columns = ((available_width + spacing) / (button_size + spacing)).floor() as usize;
            let w_button_size = available_width / ((button_size + spacing) * columns as f32 - spacing) * button_size;
            let w_spacing = available_width / ((button_size + spacing) * columns as f32 - spacing) * spacing;
            let columns = columns.max(1);
            ui.vertical_centered(|ui| {
                egui::ScrollArea::vertical()
                    .show(ui, |ui| {
                        egui::Grid::new("redeems_grid")
                            .spacing(egui::vec2(w_spacing, spacing))
                            .show(ui, |ui| {
                                let mut column_count = 0;
                                for redeem in &self.config.redeems {
                                    let response = ui.add_sized(
                                        egui::vec2(w_button_size, button_size),
                                        egui::Button::new(
                                            egui::RichText::new(&redeem.name)
                                                .heading()
                                                .size(16.0)
                                        ),
                                    );

                                    if redeem.requires_input {
                                        let popup_id = ui.make_persistent_id(&redeem.name);

                                        if response.clicked() {
                                            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                        }

                                        egui::popup::popup_below_widget(ui, popup_id, &response, PopupCloseBehavior::CloseOnClickOutside, |ui| {
                                            ui.set_min_width(button_size);
                                            ui.add_space(5.0);
                                            let mut input_text = self.input_text.write().unwrap();
                                            ui.text_edit_multiline(&mut input_text as &mut String);
                                            drop(input_text);
                                            if ui.button("Submit").clicked() {
                                                let input = Some(self.input_text.read().unwrap().clone());
                                                let redeem = redeem.clone();
                                                let auth_data = self.auth_data.clone();
                                                let status_message = self.status_message.clone();
                                                self.runtime.spawn(async move {
                                                    match Self::redeem(&auth_data, &redeem, input).await {
                                                        Ok(_) => {
                                                            *status_message.write().unwrap() = format!("Successfully redeemed {}", redeem.name);
                                                        }
                                                        Err(e) => {
                                                            *status_message.write().unwrap() = format!("Error: {}", e);
                                                        }
                                                    }
                                                });
                                                ui.memory_mut(|mem| mem.close_popup());
                                            }
                                        });
                                    } else if response.clicked() {
                                        let redeem = redeem.clone();
                                        let status_message = self.status_message.clone();
                                        let redeem = redeem.clone();
                                        let auth_data = self.auth_data.clone();
                                        self.runtime.spawn(async move {
                                            match Self::redeem(&auth_data, &redeem, None).await {
                                                Ok(_) => {
                                                    *status_message.write().unwrap() = format!("Successfully redeemed {}", redeem.name);
                                                }
                                                Err(e) => {
                                                    *status_message.write().unwrap() = format!("Error: {}", e);
                                                }
                                            }
                                        });
                                    }
                                    column_count += 1;
                                    if column_count % columns == 0 {
                                        ui.end_row();
                                    }
                                }
                            })
                    })
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };

    eframe::run_native(
        "Clonk Redeemer",
        options,
        Box::new(|_cc| {
            Ok(Box::new(ClonkApp::new().expect("Failed to initialize application")))
        }),
    )?;
    Ok(())
}