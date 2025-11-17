use slint::{ComponentHandle, Weak};

use crate::{App, AppState, JoinLogic};

pub struct Ui {
    app: App,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            app: App::new().unwrap().into(),
        }
    }

    pub fn handle(&self) -> &App {
        &self.app
    }

    pub fn as_weak(&self) -> Weak<App> {
        self.app.as_weak()
    }

    pub fn on_join<F: Fn(String, String) + 'static>(&self, f: F) {
        self.app
            .global::<JoinLogic>()
            .on_join_room(move |address, username| f(address.to_string(), username.to_string()));
    }

    pub fn on_send_message<F: Fn(String) + 'static>(&self, f: F) {
        self.app
            .global::<AppState>()
            .on_send_message(move |message| f(message.to_string()));
    }

    pub fn run(self) {
        self.app.run().unwrap();
    }
}
