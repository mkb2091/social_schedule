use rand_core::SeedableRng;
use seed::prelude::*;

use crate::{alert, database, next_tick, performance_now, prompt, schedule, style_control, Msg};

pub struct GenerateSchedule {
    players: Vec<u32>,
    tables: usize,
    schedule: Option<schedule::Generator<rand_xorshift::XorShiftRng>>,
    rng: rand_xorshift::XorShiftRng,
    cpu_usage: f64,
    operations_per_second: u32,
    operation_history: [f64; 35],
    iteration: usize,
    running: bool,
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
            operations_per_second: 0,
            operation_history: [0.0; 35],
            iteration: 0,
            running: true,
        }
    }
}

impl GenerateSchedule {
    pub fn apply_parameters(&mut self, players: Vec<u32>, tables: usize) {
        if let Ok(rng) = rand_xorshift::XorShiftRng::from_rng(&mut self.rng) {
            self.players = players;
            self.tables = tables;
            self.schedule = Some(schedule::Generator::new(
                rng,
                self.players.len(),
                self.tables,
            ));
        }
    }

    pub fn stop(&mut self) {
        self.running = false
    }

    pub fn resume(&mut self) {
        self.running = true
    }

    pub fn set_cpu_usage(&mut self, cpu_usage: String) {
        if let Ok(cpu_usage) = cpu_usage.parse::<f64>() {
            self.cpu_usage = cpu_usage;
        } else {
            alert("Invalid CPU usage");
        }
    }

    pub fn generate(&mut self) {
        if let Some(schedule) = &mut self.schedule {
            if !self.running
                || schedule.get_player_count() == 0
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
            if let Some(schedule) = &model.schedule {
                let best = &schedule.best;
                p![
                    p![
                        style![St::FontWeight => "bold";],
                        if schedule.best.is_ideal() {
                            "Found ideal schedule"
                        } else {
                            "Generating schedules..."
                        }
                    ],
                    p![format!("Operations /s: {}", model.operations_per_second)],
                    p![format!(
                        "Average number of unique games played (maximium {}): {}",
                        (best.ideal_unique_games as f32 / schedule.get_player_count() as f32),
                        (best.unique_games_played() as f32 / schedule.get_player_count() as f32)
                    )],
                    p![format!(
                        "Average number of unique opponents/teammates played with (maximium {}): {}",
                        (best.ideal_unique_opponents as f32 / schedule.get_player_count() as f32),
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
                                            let id = model.players[player_number];
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
                                if percent as f64 == model.cpu_usage {
                                    attrs! {At::Selected => "selected"}
                                } else {
                                    attrs! {}
                                },
                                format!("{}%", percent)
                            ]);
                        }
                        cpu_options
                    }
                ]
            ],
            p![if model.running {
                button![
                    style.button_style(),
                    span![
                        style![St::Color => "red"],
                        simple_ev(Ev::Click, Msg::GSStop),
                        "STOP"
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
        ]
    ]
}
