use seed::prelude::*;

use crate::{alert, database, player_select_box, prompt, style_control, Msg};

#[derive(Default)]
pub struct ManageGroups {
    pub add_group_name_input: String,
    pub add_player_to_group_input: std::collections::HashMap<u32, u32>,
    expanded: std::collections::HashSet<u32>,
}

impl ManageGroups {
    pub fn set_add_group_name_input(&mut self, group_name: String) {
        self.add_group_name_input = group_name
    }

    pub fn add_group(&mut self, database: &mut database::Database) {
        let group_name = &self.add_group_name_input;
        if !group_name.is_empty() {
            let id = database.add_group(group_name.to_string());
            self.add_group_name_input = String::new();
            self.expand(id);
        }
    }

    pub fn set_add_player_to_group_input(&mut self, group_id: u32, player_id: String) {
        if !player_id.is_empty() {
            if let Ok(player_id) = player_id.parse::<u32>() {
                self.add_player_to_group_input.insert(group_id, player_id);
            } else {
                alert(&format!("Failed to parse {} as u32", player_id));
            }
        }
    }

    pub fn add_player(&mut self, database: &mut database::Database, id: u32) {
        if let Some(player_id) = self.add_player_to_group_input.get(&id) {
            database.add_player_to_group(id, *player_id);
            self.add_player_to_group_input.remove(&id);
        }
    }
    pub fn remove_group(&self, database: &mut database::Database, id: u32) {
        database.remove_group(id);
    }
    pub fn change_name(&self, database: &mut database::Database, id: u32) {
        if let Some(new_name) = prompt("New name") {
            if new_name != "" {
                database.change_group_name(id, new_name);
            }
        }
    }
    pub fn expand(&mut self, id: u32) {
        self.expanded.insert(id);
    }
    pub fn hide(&mut self, id: u32) {
        self.expanded.remove(&id);
    }
}

pub fn view_manage_groups(
    model: &ManageGroups,
    database: &database::Database,
    style: &style_control::StyleControl,
) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap-Reverse"],
        div![
            &box_style,
            h2!["Group List"],
            table![style![St::PaddingBottom => "5px";], {
                let group_list = database.get_groups();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(group_list.len());
                for (&id, group) in &group_list {
                    let mut players: Vec<u32> = group.get_players().copied().collect();
                    players.sort_by_key(|&id| {
                        if let Some(player) = database.get_player(id) {
                            Some(&player.name)
                        } else {
                            None
                        }
                    });
                    node_list.push(tr![
                        td![h3![group.name],],
                        td![button![
                            style.button_style(),
                            ev(Ev::Click, move |_| Msg::MGChangeName(id)),
                            "Change Name"
                        ]],
                        td![button![
                            style.button_style(),
                            ev(Ev::Click, move |_| Msg::MGRemoveGroup(id)),
                            "Delete from database"
                        ]],
                        td![if model.expanded.contains(&id) {
                            button![
                                style.button_style(),
                                ev(Ev::Click, move |_| Msg::MGHide(id)),
                                "Hide players in group"
                            ]
                        } else {
                            button![
                                style.button_style(),
                                ev(Ev::Click, move |_| Msg::MGExpand(id)),
                                "Show / Edit players"
                            ]
                        }]
                    ]);
                    node_list.push(tr![td![format!("({} players)", players.len())]]);

                    if model.expanded.contains(&id) {
                        node_list.push(tr![td![
                            style![St::Border => "6px inset grey";
                        St::Padding => "10px";
                        St::Width => "min-content";],
                            attrs! {At::ColSpan => 3},
                            "Add a player to this group using the drop down menu.",
                            br![],
                            "If a player has not previously been added to the database, go to Manage Players to add a new player to the database.",
                            br![],
                            select![
                                style.button_style(),
                                input_ev("input", move |player_id| Msg::MGAddPlayerInput(
                                    id, player_id
                                )),
                                attrs!{At::Value => if let Some(&player_id) = model.add_player_to_group_input.get(&id) {
                                        player_id.to_string()
                                    } else {
                                        "".to_string()
                                    }},
                                player_select_box(
                                    database,
                                    style,
                                    &players.iter().copied().collect(),
                                )
                            ],
                            button![
                                style.button_style(),
                                ev(Ev::Click, move |_| Msg::MGAddPlayer(id)),
                                "Add Player"
                            ],
                            {
                                let mut group_node: Vec<Node<Msg>> = Vec::new();
                                for player_id in players.iter() {
                                    let player_id = *player_id;
                                    if let Some(player) = database.get_player(player_id) {
                                        group_node.push(tr![
                                            td![
                                                attrs! {
                                                    At::Title => player_id.to_string()
                                                },
                                                player.name,
                                            ],
                                            td![button![
                                                style.button_style(),
                                                ev(Ev::Click, move |_| {
                                                    Msg::MGRemovePlayerFromGroup(id, player_id)
                                                }),
                                                "Remove from group"
                                            ]]
                                        ]);
                                    }
                                }
                                table![group_node]
                            }
                        ]]);
                    }
                }
                node_list
            }],
        ],
        div![
            &box_style,
            h2!["Create New Group"],
            p![
                span!["Group Name: "],
                input![
                    attrs! {At::Value => model.add_group_name_input},
                    input_ev(Ev::Input, Msg::MGAddGroupNameInput)
                ],
            ],
            button![
                style.button_style(),
                simple_ev(Ev::Click, Msg::MGAddGroup),
                "Add"
            ],
        ],
    ]
}
