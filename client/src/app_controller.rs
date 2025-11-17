use std::{collections::HashMap, rc::Rc, sync::Arc};

use common::uuid::Uid;
use slint::{ComponentHandle, Model, VecModel};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    AppState, Chat, User, View,
    message::{NetworkMessage, UiMessage},
    network::handle_networking,
    ui::Ui,
};

pub struct AppController {
    ui: Ui,
    to_ui: UnboundedSender<NetworkMessage>,
    from_ui: UnboundedReceiver<UiMessage>,
    to_network: UnboundedSender<UiMessage>,
    from_network: UnboundedReceiver<NetworkMessage>,
    chats_model: Rc<VecModel<Chat>>,
    users_model: Rc<VecModel<User>>,
    users: HashMap<Uid, Arc<str>>,
}

impl AppController {
    pub fn new() -> Self {
        let (to_network, from_ui) = mpsc::unbounded_channel::<UiMessage>();
        let (to_ui, from_network) = mpsc::unbounded_channel::<NetworkMessage>();

        let chats_model = Rc::new(VecModel::from(vec![]));
        let users_model = Rc::new(VecModel::from(vec![]));

        let ui = Ui::new();
        let app_state = ui.handle().global::<AppState>();
        app_state.set_chats(chats_model.clone().into());
        app_state.set_online_users(users_model.clone().into());

        Self {
            ui,
            to_ui,
            from_ui,
            to_network,
            from_network,
            chats_model,
            users_model,
            users: HashMap::new(),
        }
    }

    pub async fn run(mut self) {
        {
            let tx = self.to_network.clone();
            self.ui.on_join(move |address, username| {
                let join = UiMessage::JoinRoom { address, username };
                let _ = tx.send(join);
            });
        }

        {
            let tx = self.to_network.clone();
            let chats_model = self.chats_model.clone();

            self.ui.on_send_message(move |message| {
                chats_model.push(Chat {
                    text: message.clone().into(),
                    username: "You".into(),
                    is_author: true,
                    is_system: false,
                });

                let chat = UiMessage::SendChat { text: message };
                let _ = tx.send(chat);
            });
        }

        tokio::spawn(handle_networking(self.to_ui, self.from_ui));

        let ui_weak = self.ui.as_weak();
        let mut users = self.users;
        tokio::spawn(async move {
            while let Some(message) = self.from_network.recv().await {
                match message {
                    NetworkMessage::InvalidAddress => todo!(),
                    NetworkMessage::ServerMessage(server_message) => match server_message {
                        common::protocol::ServerMessage::Chat(chat_message) => {
                            let username = users.get(&chat_message.from).unwrap();

                            let chat = Chat {
                                text: chat_message.text.to_string().into(),
                                username: username.to_string().into(),
                                is_author: false,
                                is_system: false,
                            };

                            ui_weak
                                .upgrade_in_event_loop(move |ui| {
                                    let chats_model = ui.global::<AppState>().get_chats();
                                    let chats_model = chats_model
                                        .as_any()
                                        .downcast_ref::<VecModel<Chat>>()
                                        .unwrap();
                                    chats_model.push(chat);
                                })
                                .unwrap();
                        }
                        common::protocol::ServerMessage::JoinAccepted {
                            history,
                            participants,
                        } => {
                            participants.iter().for_each(|(uuid, username)| {
                                users.insert(uuid.clone(), username.clone());
                            });

                            let chats: Vec<_> = history
                                .iter()
                                .map(|chat| {
                                    let username = users
                                        .get(&chat.from)
                                        .map_or("Unknown".to_string(), |username| {
                                            username.to_string()
                                        });
                                    Chat {
                                        text: chat.text.to_string().into(),
                                        username: username.into(),
                                        is_author: false,
                                        is_system: false,
                                    }
                                })
                                .collect();

                            let users: Vec<_> = participants
                                .iter()
                                .map(|(_, username)| User {
                                    username: username.to_string().into(),
                                })
                                .collect();

                            ui_weak
                                .upgrade_in_event_loop(move |ui| {
                                    let app_state = ui.global::<AppState>();

                                    let chats_model = app_state.get_chats();
                                    let chats_model = chats_model
                                        .as_any()
                                        .downcast_ref::<VecModel<Chat>>()
                                        .unwrap();

                                    let users_model = app_state.get_online_users();
                                    let users_model = users_model
                                        .as_any()
                                        .downcast_ref::<VecModel<User>>()
                                        .unwrap();

                                    chats_model.extend(chats);
                                    users_model.extend(users);
                                    ui.set_view(View::Chat);
                                })
                                .unwrap();
                        }
                        common::protocol::ServerMessage::UserJoined { uuid, username } => {
                            users.insert(uuid, username.clone());

                            let chat = Chat {
                                text: format!("{} joined the chat", username).into(),
                                username: "".into(),
                                is_author: false,
                                is_system: true,
                            };

                            ui_weak
                                .upgrade_in_event_loop(move |ui| {
                                    let chats_model = ui.global::<AppState>().get_chats();
                                    let chats_model = chats_model
                                        .as_any()
                                        .downcast_ref::<VecModel<Chat>>()
                                        .unwrap();
                                    chats_model.push(chat);
                                })
                                .unwrap();
                        }
                        common::protocol::ServerMessage::UserLeft { uuid } => {
                            if let Some(username) = users.remove(&uuid) {
                                let chat = Chat {
                                    text: format!("{} left the chat", username).into(),
                                    username: "".into(),
                                    is_author: false,
                                    is_system: true,
                                };

                                ui_weak
                                    .upgrade_in_event_loop(move |ui| {
                                        let chats_model = ui.global::<AppState>().get_chats();
                                        let chats_model = chats_model
                                            .as_any()
                                            .downcast_ref::<VecModel<Chat>>()
                                            .unwrap();
                                        chats_model.push(chat);
                                    })
                                    .unwrap();
                            }
                        }
                    },
                }
            }
        });

        self.ui.run();
    }
}
