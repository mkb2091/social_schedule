use rand_core::SeedableRng;
use seed::prelude::*;

use num_format::{Locale, WriteFormatted};

use crate::{alert, database, next_tick, performance_now, prompt, schedule, style_control, Msg};

pub struct GenerateSchedule {
    players: Vec<u32>,
    tables: usize,
    schedule: Option<schedule::Generator<rand_xorshift::XorShiftRng>>,
    rng: rand_xorshift::XorShiftRng,
    cpu_usage: f64,
    loops_per_milli: u32,
    operations_per_second: u32,
    operation_history: [f64; 35],
    total_operations: u64,
    iteration: usize,
    running: bool,
    event_name: String,
    event_date: String,
    last_paused: f64,
    pub found_ideal: bool,
    current_best: Node<Msg>,
}

impl Default for GenerateSchedule {
    fn default() -> Self {
        next_tick(0.0);
        Self {
            players: Vec::new(),
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
            loops_per_milli: 1,
            operations_per_second: 0,
            operation_history: [0.0; 35],
            total_operations: 0,
            iteration: 0,
            running: true,
            event_name: String::new(),
            event_date: String::new(),
            last_paused: 0.0,
            found_ideal: false,
            current_best: div![],
        }
    }
}

impl GenerateSchedule {
    pub fn apply_parameters(
        &mut self,
        players: Vec<u32>,
        tables: usize,
        event_name: &str,
        event_date: &str,
    ) {
        if let Ok(rng) = rand_xorshift::XorShiftRng::from_rng(&mut self.rng) {
            self.players = players;
            self.tables = tables;
            self.event_name = String::from(event_name);
            self.event_date = String::from(event_date);
            self.running = true;
            self.found_ideal = false;
            self.operations_per_second = 0;
            self.operation_history = [0.0; 35];
            self.total_operations = 1;
            self.schedule = Some(schedule::Generator::new(
                rng,
                self.players.len(),
                self.tables,
            ));
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.last_paused = performance_now();
    }

    pub fn resume(&mut self) {
        if performance_now() - self.last_paused > 250.0 {
            self.running = true;
        }
    }

    pub fn set_cpu_usage(&mut self, cpu_usage: String) {
        if let Ok(cpu_usage) = cpu_usage.parse::<f64>() {
            self.cpu_usage = cpu_usage;
        } else {
            alert("Invalid CPU usage");
        }
    }

    fn generate_table_display(&mut self, database: &database::Database) {
        self.current_best = if let Some(schedule) = &self.schedule {
            let best = &schedule.best;
            div![
                style![St::Border => "6px inset grey";
                    St::Padding => "10px";
                    St::Width => "max-content";],
                h3!["Current best schedule solution found"],
                p![format!(
                    "Average number of unique games played: {}",
                    best.unique_games_played() as f32 / schedule.get_player_count() as f32
                )],
                p![format!(
                    "Average number of unique opponents/teammates played with: {}",
                    (best.unique_opponents() as f32 / schedule.get_player_count() as f32)
                )],
                table![style![St::BorderSpacing => "5px 10px"; ], {
                    let tables = best.get_tables();

                    let mut table: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
                    let mut heading: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
                    heading.push(td![]);
                    for game in 1..=tables {
                        heading.push(th![format!("Table {:}", game)]);
                    }
                    table.push(tr![heading]);
                    for round in 0..tables {
                        table.push(tr![{
                            let mut row: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
                            row.push(td![format!("Round {:}", round + 1)]);
                            for table in 0..tables {
                                row.push(td![{
                                    let players = best.get_players_from_game(round, table);
                                    let mut data: Vec<Node<Msg>> = Vec::new();
                                    for player_number in players {
                                        let id = self.players[player_number];
                                        if let Some(player) = database.get_player(id) {
                                            data.push(span![player.name, br![]]);
                                        }
                                    }
                                    data
                                }]);
                            }
                            row
                        }]);
                    }
                    table
                }]
            ]
        } else {
            div![]
        }
    }

    pub fn generate(&mut self, database: &database::Database) {
        if let Some(schedule) = &mut self.schedule {
            if !self.running
                || self.found_ideal
                || schedule.get_player_count() == 0
                || schedule.get_tables() < 2
            {
                next_tick(500.0);
                self.iteration += 1;
                self.iteration %= 35;
                self.operation_history[self.iteration] = 0.0;
                self.operations_per_second = 0;
                return;
            }
            let now = performance_now();
            let ideal = if self.cpu_usage < 50.0 {
                now + self.cpu_usage * 2.0
            } else {
                now + self.cpu_usage
            };
            let mut operations: u32 = 0;
            let mut loops: u32 = 0;
            let old_score = schedule.best.get_score();
            while performance_now() < ideal {
                for _ in 0..(1.max(self.loops_per_milli / 2)) {
                    operations += schedule.process();
                    loops += 1;
                }
            }
            self.loops_per_milli = 1.max(loops / (self.cpu_usage as u32 + 1));
            self.total_operations += operations as u64;
            self.iteration += 1;
            self.iteration %= 35;
            self.operation_history[self.iteration] = (operations as f64) * 10.0;
            if self.iteration % 5 == 0 {
                self.operations_per_second =
                    (self.operation_history.iter().sum::<f64>() / 35.0) as u32;
            }
            if self.cpu_usage < 50.0 {
                next_tick(200.0 - self.cpu_usage * 2.0);
            } else {
                next_tick(100.0 - self.cpu_usage);
            }
            if schedule.best.is_ideal() {
                self.found_ideal = true
            }
            if schedule.best.get_score() > old_score {
                self.generate_table_display(&database);
            }
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
    style: &style_control::StyleControl,
) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap-Reverse";],
        div![
            &box_style,
            style![St::FlexGrow=> "0";
            St::Width => "min-content"],
            p![
                p![
                    style![St::FontWeight => "bold";],
                    if model.found_ideal {
                        "Found ideal schedule for "
                    } else {
                        "Generating schedules for "
                    },
                    model.event_name,
                ],
                ul![
                    li![format!("On {}", model.event_date)],
                    li![format!("{} players", model.players.len())],
                    li![format!("{} tables", model.tables)]
                ],
                p![
                "The algorithm will attempt to generate a schedule maximise the number of unique games each player plays, \
                while simultaneously attempting to maximise the number of unique opponents each player has",
                ],
                p!["Leaving the algorithm running longer can result in better schedules being generated. When happy with the current best generated schedule, then click 'Accept Schedule'"],
                model.current_best.clone(),
            ]
        ],
        div![
            &box_style,
            style![St::Width => "min-content"],
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
                                if percent == model.cpu_usage as usize {
                                    attrs! {At::Selected => "selected"}
                                } else {
                                    attrs! {}
                                },
                                format!("{}%", percent)
                            ]);
                        }
                        cpu_options
                    }
                ],
                br![],
                "This controls how much CPU the program will attempt to use. The high it is, the faster the schedule generation will be, but also the slower other programs running the this computer will be, and the higher temperature the CPU will reach."
            ],
            p![{
                let mut writer = String::from("Testing ");
                writer
                    .write_formatted(&model.operations_per_second, &Locale::en)
                    .unwrap();
                writer.push_str(" schedules per second");
                writer
            }],
            p![{
                let mut writer = String::from("Tested ");
                writer
                    .write_formatted(&model.total_operations, &Locale::en)
                    .unwrap();
                writer.push_str(" schedules");
                writer
            }],
            p![if model.found_ideal {span![]} else if model.running {
                button![
                    style.button_style(),
                    span![
                        style![St::Color => "red"],
                        simple_ev(Ev::Click, Msg::GSStop),
                        "PAUSE"
                    ]
                ]
            } else {
                button![
                    style.button_style(),
                    span![
                        style![St::Color => "green"],
                        simple_ev(Ev::Click, Msg::GSResume),
                        "RESUME"
                    ]
                ]
            }],
            p![
                button![
                    style.button_style(),
                    simple_ev(Ev::Click, Msg::GSBack),
                    "Back / Edit Details"
                ],
                button![style.button_style(), "Create New Event"],
                button![style.button_style(), style![St::FontWeight => "bold";], "Accept Schedule"],
            ]
        ]
    ]
}
