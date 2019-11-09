use seed::prelude::*;

use crate::{alert, button_style, database, player_select_box, Msg};

pub struct ManageGroups {
    pub add_group_name_input: String,
    pub add_player_to_group_input: std::collections::HashMap<u32, u32>,
}

impl Default for ManageGroups {
    fn default() -> Self {
        Self {
            add_group_name_input: String::new(),
            add_player_to_group_input: std::collections::HashMap::new(),
        }
    }
}

impl ManageGroups {
    pub fn set_add_group_name_input(&mut self, group_name: String) {
        self.add_group_name_input = group_name
    }

    pub fn add_group(&mut self, database: &mut database::Database) {
        let group_name = &self.add_group_name_input;
        if !group_name.is_empty() {
            database.add_group(group_name.to_string());
            self.add_group_name_input = String::new();
        }
    }

    pub fn set_add_player_to_group_input(&mut self, group_id: u32, player_id: String) {
        if let Ok(player_id) = player_id.parse::<u32>() {
            self.add_player_to_group_input.insert(group_id, player_id);
        } else {
            alert(&format!("Failed to parse {} as u32", player_id));
        }
    }

    pub fn add_player(&self, database: &mut database::Database, id: u32) {
        if let Some(player_id) = self.add_player_to_group_input.get(&id) {
            database.add_player_to_group(id, *player_id);
        }
    }
}

pub fn view_manage_groups(_model: &ManageGroups, database: &database::Database) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
        div![
            &box_style,
            h2!["Group List"],
            table![style![St::PaddingBottom => "5px";], {
                let group_list = database.get_groups();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(group_list.len());
                for (&id, group) in &group_list {
                    node_list.push(tr![
                        td![h3![group.name]],
                        td![button![button_style(), "Remove"]]
                    ]);

                    node_list.push(tr![td![
                        attrs! {At::ColSpan => 2},
                        select![
                            button_style(),
                            input_ev("input", move |player_id| Msg::MGAddPlayerInput(
                                id, player_id
                            )),
                            player_select_box(&database)
                        ],
                        button![
                            button_style(),
                            raw_ev(Ev::Click, move |_| Msg::MGAddPlayer(id)),
                            "Add Player"
                        ]
                    ]]);
                    let mut group_node: Vec<Node<Msg>> = Vec::new();
                    for player_id in group.get_players() {
                        if let Some(player) = database.get_player(*player_id) {
                            group_node.push(tr![
                                td![format!("{}: ({})", player.name, player_id)],
                                td![button![button_style(), "Remove"]]
                            ]);
                        }
                    }

                    node_list.push(tr![td![attrs! {At::ColSpan => 2}, table![group_node]]]);
                }
                node_list
            }],
        ],
        div![
            &box_style,
            p![
                span!["Group Name: "],
                input![input_ev(Ev::Input, Msg::MGAddGroupNameInput)],
            ],
            button![button_style(), simple_ev(Ev::Click, Msg::MGAddGroup), "Add"],
        ],
    ]
}
