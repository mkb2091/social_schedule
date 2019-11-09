use seed::prelude::*;

use crate::{button_style, database, Msg};

pub struct ManagePlayers {
    add_player_name_input: String,
}

impl Default for ManagePlayers {
    fn default() -> Self {
        Self {
            add_player_name_input: String::new(),
        }
    }
}

impl ManagePlayers {
    pub fn set_player_name_input(&mut self, player_name: String) {
        self.add_player_name_input = player_name;
    }
    pub fn add_player(&mut self, database: &mut database::Database) {
        let player_name = &self.add_player_name_input;
        if !player_name.is_empty() {
            database.add_player(player_name.to_string());
            self.add_player_name_input = String::new();
        }
    }
    pub fn remove_player(&self, database: &mut database::Database, id: u32) {
        database.remove_player(id);
    }
}

pub fn view_manage_players(_model: &ManagePlayers, database: &database::Database) -> Node<Msg> {
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
                let player_list = database.get_players();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(player_list.len());
                for (&id, player) in &player_list {
                    node_list.push(tr![
                        td![player.name],
                        td![button![
                            button_style(),
                            raw_ev(Ev::Click, move |_| Msg::MPRemovePlayer(id)),
                            "Remove"
                        ]]
                    ]);
                }
                node_list
            }],
        ],
        div![
            &box_style,
            p![
                span!["Player Name: "],
                input![input_ev(Ev::Input, Msg::MPAddPlayerNameInput)],
            ],
            p![span!["Email: "], input![],],
            button![
                button_style(),
                simple_ev(Ev::Click, Msg::MPAddPlayer),
                "Add"
            ],
        ],
    ]
}
