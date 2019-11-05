use seed::prelude::*;

use crate::{alert, database, player_select_box, schedule, Msg};

pub struct GenerateSchedule {
    players: Vec<u32>,
    add_player_select_box: String,
    add_group_select_box: String,
    tables: usize,
    schedule: schedule::Schedule,
    display_schedule: bool,
}

impl Default for GenerateSchedule {
    fn default() -> Self {
        Self {
            players: Vec::new(),
            add_player_select_box: String::new(),
            add_group_select_box: String::new(),
            tables: 2,
            schedule: schedule::Schedule::new(1, 1),
            display_schedule: false,
        }
    }
}

impl GenerateSchedule {
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
                    alert("Played with specified ID does not exist");
                }
            } else {
                alert("Invalid ID of player");
            }
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
                    alert("Played does not exist");
                }
            } else {
                alert("Failed to convert ID to integer");
            }
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
            alert("Played with specified ID not in list");
        }
    }

    pub fn set_tables(&mut self, tables: String) {
        if let Ok(tables) = tables.parse::<usize>() {
            self.tables = tables;
        } else {
            alert("Invalid player count");
        }
    }

    pub fn generate(&mut self, rng: &mut rand_xorshift::XorShiftRng) {
        self.schedule = schedule::Schedule::new(self.players.len(), self.tables);
        self.schedule.generate_random(rng);
        self.display_schedule = true;
    }
}

pub fn view_generate_schedule(
    model: &GenerateSchedule,
    database: &database::Database,
) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
        div![
            &box_style,
            h2!["Tournament Players"],
            p![
                span!["Group: "],
                select![
                    attrs! {At::Value => ""},
                    input_ev(Ev::Input, Msg::GSAddGroupSelectBoxInput),
                    {
                        let group_list = database.get_groups();
                        let mut node_list: Vec<Node<Msg>> =
                            Vec::with_capacity(group_list.len() + 1);
                        node_list.push(option![attrs! {At::Value => ""}, ""]);
                        for (&id, group) in &group_list {
                            node_list.push(option![
                                attrs! {At::Value => id},
                                format!("{}: ({})", group.name, id)
                            ]);
                        }
                        node_list
                    }
                ],
                button![simple_ev(Ev::Click, Msg::GSAddGroup), "Add"],
            ],
            p![
                span!["Individual: "],
                select![
                    attrs! {At::Value => ""},
                    input_ev(Ev::Input, Msg::GSAddPlayerSelectBoxInput),
                    player_select_box(&database),
                ],
                button![simple_ev(Ev::Click, Msg::GSAddPlayer), "Add"],
            ],
            ul![style![St::PaddingBottom => "5px";], {
                let mut players_list: Vec<Node<Msg>> = Vec::with_capacity(model.players.len());
                for &player_id in &model.players {
                    players_list.push(li![
                        if let Some(player) = database.get_player(player_id) {
                            &player.name
                        } else {
                            "Player does not exist"
                        },
                        button![
                            raw_ev(Ev::Click, move |_| Msg::GSRemovePlayer(player_id)),
                            "Remove"
                        ]
                    ]);
                }
                players_list
            }],
        ],
        div![
            &box_style,
            p![
                span!["Tables: "],
                select![input_ev(Ev::Input, Msg::GSSetTables), {
                    let mut table_size_list: Vec<Node<Msg>> = Vec::with_capacity(42);
                    for table_size in 2..43 {
                        table_size_list.push(option![
                            attrs! {At::Value => table_size},
                            format!("{}", table_size)
                        ]);
                    }
                    table_size_list
                }],
            ],
            p![
                span!["Email Players: "],
                input![attrs! {At::Type => "checkbox"}],
            ],
            button![simple_ev(Ev::Click, Msg::GSGenerate), "Generate"],
            if model.display_schedule {
                p![
                    p![format!(
                        "Total unique games played(ideally players * games): {}",
                        model.schedule.unique_games_played()
                    )],
                    p![format!(
                        "Total unique opponents/teammates played(higher is better): {}",
                        model.schedule.unique_opponents()
                    )],
                    table![{
                        let tables = model.schedule.get_tables();

                        let mut table: Vec<Node<Msg>> = Vec::with_capacity(tables);
                        for round in 0..tables {
                            table.push(tr![{
                                let mut row: Vec<Node<Msg>> = Vec::with_capacity(tables);
                                for table in 0..tables {
                                    row.push(td![{
                                        format!("{:?}", model.schedule.get_game(round, table))
                                    }]);
                                }
                                row
                            }]);
                        }
                        table
                    }],
                ]
            } else {
                p![]
            }
        ],
        div![
            &box_style,
            p![
                span!["Runtime Limit: "],
                input![attrs! {At::Type => "checkbox"}],
                p![
                    style![St::PaddingLeft => "30px";],
                    span!["Max run time: "],
                    input![],
                    button!["Apply"]
                ]
            ],
            p![
                span!["Maximum CPU usage"],
                select![attrs! {At::Value => "100"}, {
                    let mut cpu_options: Vec<Node<Msg>> = Vec::with_capacity(100);
                    for percent in 0..100 {
                        let percent = 100 - percent;
                        cpu_options.push(option![
                            attrs! {At::Value => percent},
                            format!("{}%", percent)
                        ]);
                    }
                    cpu_options
                }]
            ],
        ]
    ]
}
