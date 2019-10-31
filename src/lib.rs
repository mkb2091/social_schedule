pub mod database;
pub mod schedule;

extern crate rand_core;
extern crate rand_xorshift;
use rand_core::SeedableRng;
extern crate getrandom;
extern crate rand;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate seed;
use seed::prelude::*;

extern crate wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[derive(Clone)]
enum Page {
    GenerateSchedule,
    ManagePlayers,
    ManageGroups,
    Preferences,
}

struct GenerateSchedule {
    pub players: Vec<u32>,
    pub add_player_select_box: String,
    pub tables: usize,
    schedule: schedule::Schedule,
    display_schedule: bool,
}

struct ManagePlayers {
    pub add_player_name_input: String,
}

struct ManageGroups {
    add_group_name_input: String,
    add_player_to_group_input: std::collections::HashMap<u32, u32>,
}

struct Model {
    pub page: Page,
    pub generate_schedule: GenerateSchedule,
    pub manage_players: ManagePlayers,
    manage_groups: ManageGroups,
    database: database::Database,
    rng: rand_xorshift::XorShiftRng,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            page: Page::GenerateSchedule,
            generate_schedule: GenerateSchedule {
                players: Vec::new(),
                add_player_select_box: String::new(),
                tables: 2,
                schedule: schedule::Schedule::new(1, 1),
                display_schedule: false,
            },
            manage_players: ManagePlayers {
                add_player_name_input: String::new(),
            },
            manage_groups: ManageGroups {
                add_group_name_input: String::new(),
                add_player_to_group_input: std::collections::HashMap::new(),
            },
            database: database::Database::load(),
            rng: {
                let mut seed: [u8; 16] = [0; 16];
                if getrandom::getrandom(&mut seed).is_err() {
                    alert("Failed to seed RNG");
                };
                rand_xorshift::XorShiftRng::from_seed(seed)
            },
        }
    }
}

#[derive(Clone)]
enum Msg {
    ChangePage(Page),
    GSAddPlayer,
    GSAddPlayerSelectBoxInput(String),
    GSRemovePlayer(u32),
    GSSetTables(String),
    GSGenerate,
    MPAddPlayer,
    MPAddPlayerNameInput(String),
    MPRemovePlayer(u32),
    MGAddGroup,
    MGAddGroupNameInput(String),
    MGAddPlayerInput(u32, String),
    MGAddPlayer(u32),
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangePage(page) => {
            model.page = page;
            model.manage_players.add_player_name_input = String::new();
        }
        Msg::GSAddPlayerSelectBoxInput(id) => {
            model.generate_schedule.add_player_select_box = id;
        }
        Msg::GSAddPlayer => {
            let player_id = &model.generate_schedule.add_player_select_box;
            if !player_id.is_empty() {
                if let Ok(id) = player_id.parse::<u32>() {
                    if model.database.contains_player(id) {
                        model.generate_schedule.players.push(id);
                    } else {
                        alert("Played with specified ID does not exist");
                    }
                } else {
                    alert("Invalid ID of player");
                }
            }
        }
        Msg::GSRemovePlayer(id) => {
            if let Some((pos, _)) = model
                .generate_schedule
                .players
                .iter()
                .enumerate()
                .find(|(_, &player_id)| id == player_id)
            {
                model.generate_schedule.players.remove(pos);
            } else {
                alert("Played with specified ID not in list");
            }
        }
        Msg::GSSetTables(tables) => {
            if let Ok(tables) = tables.parse::<usize>() {
                model.generate_schedule.tables = tables;
            } else {
                alert("Invalid player count");
            }
        }
        Msg::GSGenerate => {
            model.generate_schedule.schedule = schedule::Schedule::new(
                model.generate_schedule.players.len(),
                model.generate_schedule.tables,
            );
            model
                .generate_schedule
                .schedule
                .generate_random(&mut model.rng);
            model.generate_schedule.display_schedule = true;
        }
        Msg::MPAddPlayerNameInput(player_name) => {
            model.manage_players.add_player_name_input = player_name;
        }
        Msg::MPAddPlayer => {
            let player_name = &model.manage_players.add_player_name_input;
            if !player_name.is_empty() {
                model.database.add_player(player_name.to_string());
                model.manage_players.add_player_name_input = String::new();
            }
        }
        Msg::MPRemovePlayer(id) => {
            model.database.remove_player(id);
        }
        Msg::MGAddGroup => {
            let group_name = &model.manage_groups.add_group_name_input;
            if !group_name.is_empty() {
                model.database.add_group(group_name.to_string());
                model.manage_groups.add_group_name_input = String::new();
            }
        }
        Msg::MGAddGroupNameInput(group_name) => {
            model.manage_groups.add_group_name_input = group_name;
        }
        Msg::MGAddPlayerInput(group_id, player_id) => {
            if let Ok(player_id) = player_id.parse::<u32>() {
                model
                    .manage_groups
                    .add_player_to_group_input
                    .insert(group_id, player_id);
            } else {
                alert(&format!("Failed to parse {} as u32", player_id));
            }
        }
        Msg::MGAddPlayer(id) => {
            if let Some(player_id) = model.manage_groups.add_player_to_group_input.get(&id) {
                model.database.add_player_to_group(id, *player_id);
            }
        }
    }
}

fn player_select_box(database: &database::Database) -> Vec<Node<Msg>> {
    let player_list = database.get_players();
    let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(player_list.len() + 1);
    node_list.push(option![attrs! {At::Value => ""}, ""]);
    for (id, player) in &player_list {
        node_list.push(option![
            attrs! {At::Value => id},
            format!("{}: ({})", player.name, id)
        ]);
    }
    node_list
}

fn view_generate_schedule(model: &Model) -> Node<Msg> {
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
                select![attrs! {At::Value => ""}, {
                    let group_list = model.database.get_groups();
                    let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(group_list.len() + 1);
                    node_list.push(option![attrs! {At::Value => ""}, ""]);
                    for (&id, group) in &group_list {
                        node_list.push(option![
                            attrs! {At::Value => id},
                            format!("{}: ({})", group.name, id)
                        ]);
                    }
                    node_list
                }],
                button!["Add"],
            ],
            p![
                span!["Individual: "],
                select![
                    attrs! {At::Value => ""},
                    input_ev(Ev::Input, Msg::GSAddPlayerSelectBoxInput),
                    player_select_box(&model.database),
                ],
                button![simple_ev(Ev::Click, Msg::GSAddPlayer), "Add"],
            ],
            ul![style![St::PaddingBottom => "5px";], {
                let mut players_list: Vec<Node<Msg>> =
                    Vec::with_capacity(model.generate_schedule.players.len());
                for &player_id in &model.generate_schedule.players {
                    players_list.push(li![
                        if let Some(player) = model.database.get_player(player_id) {
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
            if model.generate_schedule.display_schedule {
                p![
                    p![format!(
                        "Total unique games played(ideally players * games): {}",
                        model.generate_schedule.schedule.unique_games_played()
                    )],
                    p![format!(
                        "Total unique opponents/teammates played(higher is better): {}",
                        model.generate_schedule.schedule.unique_opponents()
                    )],
                    table![{
                        let tables = model.generate_schedule.schedule.get_tables();

                        let mut table: Vec<Node<Msg>> = Vec::with_capacity(tables);
                        for round in 0..tables {
                            table.push(tr![{
                                let mut row: Vec<Node<Msg>> = Vec::with_capacity(tables);
                                for table in 0..tables {
                                    row.push(td![{
                                        format!(
                                            "{:?}",
                                            model.generate_schedule.schedule.get_game(round, table)
                                        )
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

fn view_manage_players(model: &Model) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
        div![
            &box_style,
            h2!["Player List"],
            ul![style![St::PaddingBottom => "5px";], {
                let player_list = model.database.get_players();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(player_list.len());
                for (&id, player) in &player_list {
                    node_list.push(li![
                        player.name,
                        button![
                            raw_ev(Ev::Click, move |_| Msg::MPRemovePlayer(id)),
                            "Remove"
                        ]
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
            button![simple_ev(Ev::Click, Msg::MPAddPlayer), "Add"],
        ],
    ]
}

fn view_manage_groups(model: &Model) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
        div![
            &box_style,
            h2!["Group List"],
            ul![style![St::PaddingBottom => "5px";], {
                let group_list = model.database.get_groups();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(group_list.len());
                for (&id, group) in &group_list {
                    let mut group_node: Vec<Node<Msg>> = Vec::new();
                    group_node.push(li![
                        select![
                            input_ev("input", move |player_id| Msg::MGAddPlayerInput(
                                id, player_id
                            )),
                            player_select_box(&model.database)
                        ],
                        button![
                            raw_ev(Ev::Click, move |_| Msg::MGAddPlayer(id)),
                            "Add Player"
                        ]
                    ]);
                    for player_id in group.get_players() {
                        if let Some(player) = model.database.get_player(*player_id) {
                            group_node.push(li![format!("{}: ({})", player.name, player_id)]);
                        }
                    }
                    node_list.push(li![group.name, button!["Remove"], ul!(group_node)]);
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
            button![simple_ev(Ev::Click, Msg::MGAddGroup), "Add"],
        ],
    ]
}

fn view(model: &Model) -> impl View<Msg> {
    vec![
        title![match model.page {
            Page::GenerateSchedule => "Generate Schedule",
            Page::ManagePlayers => "Manage Players",
            Page::ManageGroups => "Manage Groups",
            Page::Preferences => "Preferences",
        }],
        h1![match model.page {
            Page::GenerateSchedule => "Generate Schedule",
            Page::ManagePlayers => "Manage Players",
            Page::ManageGroups => "Manage Groups",
            Page::Preferences => "Preferences",
        }],
        button![
            simple_ev(Ev::Click, Msg::ChangePage(Page::GenerateSchedule)),
            "Generate Schedule"
        ],
        button![
            simple_ev(Ev::Click, Msg::ChangePage(Page::ManagePlayers)),
            "Manage Players"
        ],
        button![
            simple_ev(Ev::Click, Msg::ChangePage(Page::ManageGroups)),
            "Manage Groups"
        ],
        button![
            simple_ev(Ev::Click, Msg::ChangePage(Page::Preferences)),
            "Preferences"
        ],
        match model.page {
            Page::GenerateSchedule => view_generate_schedule(model),
            Page::ManagePlayers => view_manage_players(model),
            Page::ManageGroups => view_manage_groups(model),
            _ => div![],
        },
    ]
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Model::default(), update, view)
        .finish()
        .run();
}
