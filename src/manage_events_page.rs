use seed::prelude::*;

use crate::{alert, database, generate_schedule_page, player_select_box, style_control, Msg};

pub struct ManageEvents {}

impl Default for ManageEvents {
    fn default() -> Self {
        Self {}
    }
}

pub fn view_manage_events(
    _model: &ManageEvents,
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
            table![style![St::PaddingBottom => "5px";], {
                let events_list = database.get_events();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(events_list.len());
                for (&id, event) in &events_list {
                    node_list.push(tr![
                        td![id.to_string()],
                        td![event.name],
                        td![format!("{:} players", event.players.len())],
                        td![button![
                            style.button_style(),
                            raw_ev(Ev::Click, move |_| Msg::MPChangeName(id)),
                            "Change Name"
                        ]],
                    ]);
                }
                node_list
            }],
        ],
    ]
}

enum CreateEventStages {
    Details,
    AddPlayers,
    GenerateSchedule,
}

pub struct CreateEvent {
    event_name: String,
    event_date: String,
    players: Vec<u32>,
    add_player_select_box: String,
    add_group_select_box: String,
    tables: usize,
    stage: CreateEventStages,
}

impl Default for CreateEvent {
    fn default() -> Self {
        Self {
            event_name: String::new(),
            event_date: String::new(),
            players: Vec::new(),
            add_player_select_box: String::new(),
            add_group_select_box: String::new(),
            tables: 2,
            stage: CreateEventStages::Details,
        }
    }
}

impl CreateEvent {
    pub fn set_event_name(&mut self, name: String) {
        self.event_name = name
    }
    pub fn set_event_date(&mut self, date: String) {
        self.event_date = date
    }
    pub fn go_to_enter_players(&mut self) {
        if !self.event_name.is_empty() || !self.event_date.is_empty() {
            self.stage = CreateEventStages::AddPlayers;
        } else {
            alert("Empty name or date");
        }
    }
    pub fn back_to_details(&mut self) {
        self.stage = CreateEventStages::Details;
    }

    pub fn go_to_generate_schedule_page(
        &mut self,
        generate_schedule_model: &mut generate_schedule_page::GenerateSchedule,
    ) {
        if self.players.len() > 64 {
            alert(&format!(
                "Has {} players, which is above the maximum of 64",
                self.players.len()
            ));
            return;
        }
        self.stage = CreateEventStages::GenerateSchedule;
        generate_schedule_model.apply_parameters(self.players.clone(), self.tables)
    }

    pub fn set_add_player_select_box_input(&mut self, id: String) {
        self.add_player_select_box = id;
    }
    pub fn add_player(&mut self, database: &database::Database) {
        let player_id = &self.add_player_select_box;
        if !player_id.is_empty() {
            if let Ok(id) = player_id.parse::<u32>() {
                if database.contains_player(id) {
                    self.players.push(id);
                } else {
                    alert("Player with specified ID does not exist");
                }
            } else {
                alert("Invalid ID of player");
            }
        }
        if self.players.len() > 64 {
            alert(&format!(
                "Has {} players, which is above the maximum of 64",
                self.players.len()
            ));
        }
    }
    pub fn set_add_group_select_box_input(&mut self, id: String) {
        self.add_group_select_box = id;
    }

    pub fn add_group(&mut self, database: &database::Database) {
        let group_id = &self.add_group_select_box;
        if !group_id.is_empty() {
            if let Ok(id) = group_id.parse::<u32>() {
                if let Some(group) = database.get_group(id) {
                    for player in group.get_players() {
                        self.players.push(*player);
                    }
                } else {
                    alert("Player does not exist");
                }
            } else {
                alert("Failed to convert ID to integer");
            }
        }
        if self.players.len() > 64 {
            alert(&format!(
                "Has {} players, which is above the maximum of 64",
                self.players.len()
            ));
        }
    }

    pub fn remove_player(&mut self, id: u32) {
        if let Some((pos, _)) = self
            .players
            .iter()
            .enumerate()
            .find(|(_, &player_id)| id == player_id)
        {
            self.players.remove(pos);
        } else {
            alert("Player with specified ID not in list");
        }
    }

    pub fn remove_all_players(&mut self) {
        self.players = Vec::new();
    }
    pub fn set_tables(&mut self, tables: String) {
        if let Ok(tables) = tables.parse::<usize>() {
            self.tables = tables;
        } else {
            alert("Invalid player count");
        }
    }
}

fn view_create_event_details(
    model: &CreateEvent,
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
            table![
                tr![
                    td!["Event Name: "],
                    td![input![input_ev(Ev::Input, Msg::CESetEventName)]]
                ],
                tr![
                    td!["Event date: "],
                    td![input![input_ev(Ev::Input, Msg::CESetEventDate)]]
                ]
            ],
            button![
                style.button_style(),
                "Next",
                simple_ev(Ev::Click, Msg::CEGoToEnterPlayers)
            ],
        ],
    ]
}

fn view_create_event_players(
    model: &CreateEvent,
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
            table![style![St::PaddingBottom => "5px";], {
                let mut players_list: Vec<Node<Msg>> = Vec::with_capacity(model.players.len());
                for &player_id in &model.players {
                    players_list.push(tr![
                        td![if let Some(player) = database.get_player(player_id) {
                            &player.name
                        } else {
                            "Player does not exist"
                        }],
                        td![button![
                            style.button_style(),
                            raw_ev(Ev::Click, move |_| Msg::CERemovePlayer(player_id)),
                            "Remove"
                        ]]
                    ]);
                }
                players_list
            }],
        ],
        div![
            &box_style,
            p![
                span!["Group: "],
                select![
                    style.button_style(),
                    attrs! {At::Value => ""},
                    input_ev(Ev::Input, Msg::CEAddGroupSelectBoxInput),
                    {
                        let group_list = database.get_groups();
                        let mut node_list: Vec<Node<Msg>> =
                            Vec::with_capacity(group_list.len() + 1);
                        node_list.push(option![style.option_style(), attrs! {At::Value => ""}, ""]);
                        for (&id, group) in &group_list {
                            let player_count = group.get_players().len();
                            node_list.push(option![
                                style.option_style(),
                                attrs! {At::Value => id},
                                format!("{} ({})", group.name, player_count)
                            ]);
                        }
                        node_list
                    }
                ],
                button![
                    style.button_style(),
                    simple_ev(Ev::Click, Msg::CEAddGroup),
                    "Add"
                ],
            ],
            p![
                span!["Individual: "],
                select![
                    style.button_style(),
                    attrs! {At::Value => ""},
                    input_ev(Ev::Input, Msg::CEAddPlayerSelectBoxInput),
                    player_select_box(database, style),
                ],
                button![
                    style.button_style(),
                    simple_ev(Ev::Click, Msg::CEAddPlayer),
                    "Add"
                ],
            ],
            p![button![
                style.button_style(),
                simple_ev(Ev::Click, Msg::CERemoveAllPlayers),
                "Remove All"
            ]],
            p![
                span!["Tables: "],
                select![
                    style.button_style(),
                    input_ev(Ev::Input, Msg::CESetTables),
                    {
                        let mut table_size_list: Vec<Node<Msg>> = Vec::with_capacity(42);
                        for table_size in 2..43 {
                            table_size_list.push(option![
                                style.option_style(),
                                attrs! {At::Value => table_size},
                                format!("{}", table_size)
                            ]);
                        }
                        table_size_list
                    }
                ],
            ],
            button![
                style.button_style(),
                simple_ev(Ev::Click, Msg::CEBackToDetails),
                "Back"
            ],
            button![
                style.button_style(),
                simple_ev(Ev::Click, Msg::CEGoToGenerateSchedule),
                "Next"
            ]
        ]
    ]
}

pub fn view_create_event(
    model: &CreateEvent,
    generate_schedule_model: &generate_schedule_page::GenerateSchedule,
    database: &database::Database,
    style: &style_control::StyleControl,
) -> Node<Msg> {
    match model.stage {
        CreateEventStages::Details => view_create_event_details(model, database, style),
        CreateEventStages::AddPlayers => view_create_event_players(model, database, style),
        CreateEventStages::GenerateSchedule => {
            generate_schedule_page::view_generate_schedule(generate_schedule_model, database, style)
        }
    }
}
