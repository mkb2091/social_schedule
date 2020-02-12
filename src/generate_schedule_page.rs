use rand_core::SeedableRng;
use seed::prelude::*;

use crate::{
    alert, database, next_tick, performance_now, player_select_box, prompt, schedule,
    style_control, Msg,
};

pub struct GenerateSchedule {
    players: Vec<u32>,
    add_player_select_box: String,
    add_group_select_box: String,
    tables: usize,
    schedule: Option<schedule::Generator<rand_xorshift::XorShiftRng>>,
    rng: rand_xorshift::XorShiftRng,
    cpu_usage: f64,
    operations_per_second: u32,
    operation_history: [f64; 35],
    iteration: usize,
}

impl Default for GenerateSchedule {
    fn default() -> Self {
        next_tick(0.0);
        Self {
            players: Vec::new(),
            add_player_select_box: String::new(),
            add_group_select_box: String::new(),
            tables: 2,
            schedule: None,
            rng: {
                let mut seed: [u8; 16] = [0; 16];
                if getrandom::getrandom(&mut seed).is_err() {
                    alert("Failed to seed RNG");
                };
                rand_xorshift::XorShiftRng::from_seed(seed)
            },
            cpu_usage: 99.0,
            operations_per_second: 0,
            operation_history: [0.0; 35],
            iteration: 0,
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
                    alert("Player with specified ID does not exist");
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
                    alert("Player does not exist");
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

    pub fn set_cpu_usage(&mut self, cpu_usage: String) {
        if let Ok(cpu_usage) = cpu_usage.parse::<f64>() {
            self.cpu_usage = cpu_usage;
        } else {
            alert("Invalid player count");
        }
    }

    pub fn apply(&mut self) {
        if let Ok(rng) = rand_xorshift::XorShiftRng::from_rng(&mut self.rng) {
            self.schedule = Some(schedule::Generator::new(
                rng,
                self.players.len(),
                self.tables,
            ));
        }
    }

    pub fn generate(&mut self) {
        if let Some(schedule) = &mut self.schedule {
            if schedule.get_player_count() == 0
                || schedule.get_tables() < 2
                || schedule.best.is_ideal()
            {
                next_tick(100.0);
                self.iteration += 1;
                self.iteration %= 35;
                self.operation_history[self.iteration] = 0.0;
                self.operations_per_second = 0;
                return;
            }
            let now = performance_now();
            let ideal = now + self.cpu_usage;
            let predicted_loops =
                ((self.operations_per_second as f64) / 1000.0 * self.cpu_usage) as u32;
            for _ in 0..(predicted_loops / 2) {
                // Reduce calls to performance_now() by predicting
                schedule.process();
            }
            let mut operations: u32 = predicted_loops / 2;
            while performance_now() < ideal {
                schedule.process();
                operations += 1;
            }
            self.iteration += 1;
            self.iteration %= 35;
            self.operation_history[self.iteration] = (operations as f64) * 10.0;
            self.operations_per_second = (self.operation_history.iter().sum::<f64>() / 35.0) as u32;
            next_tick(100.0 - self.cpu_usage);
        } else {
            next_tick(100.0);
            self.operations_per_second = 0;
        }
    }
    pub fn make_event(&self, database: &mut database::Database) {
        if let Some(schedule) = &self.schedule {
            if let Some(name) = prompt("Event name") {
                database.add_event(name, schedule.best.clone(), self.players.clone());
            }
        }
    }
}

pub fn view_generate_schedule(
    model: &GenerateSchedule,
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
            h2!["Event Players"],
            p![
                span!["Group: "],
                select![
                    style.button_style(),
                    attrs! {At::Value => ""},
                    input_ev(Ev::Input, Msg::GSAddGroupSelectBoxInput),
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
                    simple_ev(Ev::Click, Msg::GSAddGroup),
                    "Add"
                ],
            ],
            p![
                span!["Individual: "],
                select![
                    style.button_style(),
                    attrs! {At::Value => ""},
                    input_ev(Ev::Input, Msg::GSAddPlayerSelectBoxInput),
                    player_select_box(database, style),
                ],
                button![
                    style.button_style(),
                    simple_ev(Ev::Click, Msg::GSAddPlayer),
                    "Add"
                ],
            ],
            p![button![
                style.button_style(),
                simple_ev(Ev::Click, Msg::GSRemoveAllPlayers),
                "Remove All"
            ]],
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
                            raw_ev(Ev::Click, move |_| Msg::GSRemovePlayer(player_id)),
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
                span!["Tables: "],
                select![
                    style.button_style(),
                    input_ev(Ev::Input, Msg::GSSetTables),
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
            p![
                span!["Email Players: "],
                input![attrs! {At::Type => "checkbox"}],
            ],
            button![
                style.button_style(),
                simple_ev(Ev::Click, Msg::GSApply),
                "Apply parameters"
            ],
            if let Some(schedule) = &model.schedule {
                let best = &schedule.best;
                p![
                    p![format!("Operations /s: {}", model.operations_per_second)],
                    p![format!(
                        "Total unique games played(ideally {}): {}",
                        best.ideal_unique_games,
                        best.unique_games_played()
                    )],
                    p![format!(
                        "Total unique opponents/teammates played(ideally {}): {}",
                        best.ideal_unique_opponents,
                        best.unique_opponents()
                    )],
                    table![{
                        let tables = best.get_tables();

                        let mut table: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
                        let mut heading: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
                        heading.push(td![]);
                        for game in 1..=tables {
                            heading.push(td![format!("Table {:}", game)]);
                        }
                        table.push(tr![heading]);
                        for round in 0..tables {
                            table.push(tr![{
                                let mut row: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
                                row.push(td![format!("Round {:}", round + 1)]);
                                for table in 0..tables {
                                    row.push(td![{
                                        format!("{:?}", best.get_players_from_game(round, table))
                                    }]);
                                }
                                row
                            }]);
                        }
                        table
                    }],
                    button![
                        style.button_style(),
                        simple_ev(Ev::Click, Msg::GSMakeEvent),
                        "Make event with schedule"
                    ]
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
                    button![style.button_style(), "Apply"]
                ]
            ],
            p![
                span!["Maximum CPU usage: "],
                select![
                    style.button_style(),
                    input_ev(Ev::Input, Msg::GSSetCpuUsage),
                    attrs! {At::Value => format!("{:}", model.cpu_usage as u32)},
                    {
                        let mut cpu_options: Vec<Node<Msg>> = Vec::with_capacity(100);
                        for percent in 0..99 {
                            let percent = 99 - percent;
                            cpu_options.push(option![
                                style.option_style(),
                                attrs! {At::Value => percent},
                                format!("{}%", percent)
                            ]);
                        }
                        cpu_options
                    }
                ]
            ],
        ]
    ]
}
