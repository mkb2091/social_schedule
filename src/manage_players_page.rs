use seed::prelude::*;

use crate::{alert, database, prompt, style_control, Msg};

pub struct ManagePlayers {
    add_player_name_input: String,
    add_player_email_input: String,
}

impl Default for ManagePlayers {
    fn default() -> Self {
        Self {
            add_player_name_input: String::new(),
            add_player_email_input: String::new(),
        }
    }
}

impl ManagePlayers {
    pub fn set_player_name_input(&mut self, player_name: String) {
        self.add_player_name_input = player_name;
    }
    pub fn set_player_email_input(&mut self, player_email: String) {
        self.add_player_email_input = player_email;
    }
    pub fn add_player(&mut self, database: &mut database::Database) {
        let player_name = &self.add_player_name_input;
        let player_email = &self.add_player_email_input;
        if !player_name.is_empty() {
            if let Ok(email) = database::Email::parse_string(player_email) {
                database.add_player(player_name.to_string(), email);
                self.add_player_name_input = String::new();
                self.add_player_email_input = String::new();
            } else {
                alert(&format!("Failed to parse {:} as email", player_email));
            }
        }
    }
    pub fn remove_player(&self, database: &mut database::Database, id: u32) {
        database.remove_player(id);
    }

    pub fn change_name(&self, database: &mut database::Database, id: u32) {
        if let Some(new_name) = prompt("New name") {
            if new_name != "" {
                database.change_player_name(id, new_name);
            }
        }
    }
    pub fn change_email(&self, database: &mut database::Database, id: u32) {
        if let Some(new_email) = prompt("New email") {
            if let Ok(email) = database::Email::parse_string(&new_email) {
                database.change_player_email(id, email)
            } else {
                alert(&format!("Failed to parse {:} as email", new_email));
            }
        }
    }
}

pub fn view_manage_players(
    model: &ManagePlayers,
    database: &database::Database,
    style: &style_control::StyleControl,
) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
        div![
            &box_style,
            h2!["Player List"],
            table![style![St::PaddingBottom => "5px";], {
                let mut player_list = database.get_players();
                player_list.sort_by_key(|(_, player)| &player.name);
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(player_list.len() + 1);
                if !player_list.is_empty() {
                    node_list.push(tr![
                        td![style![St::PaddingRight => "25px";], "ID"],
                        td!["Name"],
                        td!["Email"]
                    ])
                }
                for (&id, player) in &player_list {
                    node_list.push(tr![
                        td![id.to_string()],
                        td![player.name],
                        td![if let Some(email) = &player.email {
                            email.to_string()
                        } else {
                            String::new()
                        }],
                        td![button![
                            style.button_style(),
                            raw_ev(Ev::Click, move |_| Msg::MPChangeName(id)),
                            "Edit Name"
                        ]],
                        td![button![
                            style.button_style(),
                            raw_ev(Ev::Click, move |_| Msg::MPChangeEmail(id)),
                            "Change Email"
                        ]],
                        td![button![
                            style.button_style(),
                            raw_ev(Ev::Click, move |_| Msg::MPRemovePlayer(id)),
                            "Delete from database"
                        ]],
                    ]);
                }
                node_list
            }],
        ],
        div![
            &box_style,
            h2!["Add new player"],
            p![
                span!["Player Name: "],
                input![
                    attrs! {At::Value => model.add_player_name_input},
                    input_ev(Ev::Input, Msg::MPAddPlayerNameInput)
                ],
            ],
            p![
                span!["Email: "],
                input![
                    attrs! {At::Value => model.add_player_email_input},
                    input_ev(Ev::Input, Msg::MPAddPlayerEmailInput)
                ]
            ],
            button![
                style.button_style(),
                simple_ev(Ev::Click, Msg::MPAddPlayer),
                "Add"
            ],
        ],
    ]
}
