use crate::app_controller::AppController;

mod app_controller;
mod error;
mod message;
mod network;
mod ui;

slint::include_modules!();

#[tokio::main]
async fn main() {
    let controller = AppController::new();
    controller.run().await;
}
