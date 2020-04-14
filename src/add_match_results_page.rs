use seed::prelude::*;

use crate::{alert, database, schedule, style_control, Msg};

use schedule::ScheduleStructure;

#[derive(Default)]
pub struct AddMatchResults {
    expanded_schedules: std::collections::HashSet<u32>,
    score_inputs:
        std::collections::HashMap<u32, std::collections::HashMap<(usize, usize, usize), usize>>,
}

impl AddMatchResults {
    pub fn expand_schedule(&mut self, id: u32) {
        self.expanded_schedules.insert(id);
    }
    pub fn hide_schedule(&mut self, id: u32) {
        self.expanded_schedules.remove(&id);
    }
    pub fn set_score(
        &mut self,
        id: u32,
        round: usize,
        table: usize,
        player_number: usize,
        score: String,
    ) {
        if score.is_empty() {
            self.score_inputs
                .entry(id)
                .or_default()
                .remove(&(round, table, player_number));
        } else if let Ok(score) = score.parse::<usize>() {
            self.score_inputs
                .entry(id)
                .or_default()
                .insert((round, table, player_number), score);
        } else {
            alert(&("Not a Number: ".to_owned() + &score))
        }
    }
    pub fn add_results(&mut self, id: u32, database: &mut database::Database) {
        if let Some(event) = database.get_event(id) {
            let event = event.clone();
            if let Some(event_matches) = self.score_inputs.get(&id) {
                let mut data: Vec<Vec<Vec<(u32, usize)>>> = Vec::with_capacity(event.tables);
                for round in 0..event.tables {
                    let mut round_vec: Vec<Vec<(u32, usize)>> = Vec::with_capacity(event.tables);
                    for table in 0..event.tables {
                        let mut table_vec: Vec<(u32, usize)> = Vec::new();
                        for player_number in event.schedule.get_players_from_game(round, table) {
                            let player_id = event.players[player_number];
                            if let Some(score) = event_matches.get(&(round, table, player_number)) {
                                table_vec.push((player_id, *score));
                            } else {
                                alert("Haven't entered all scores");
                                return;
                            }
                        }
                        round_vec.push(table_vec);
                    }
                    data.push(round_vec);
                }
                let mut match_ids: Vec<Vec<u32>> = Vec::with_capacity(event.tables);
                for round in data.iter().take(event.tables) {
                    let mut round_vec: Vec<u32> = Vec::with_capacity(event.tables);
                    for table in 0..event.tables {
                        let id = database
                            .add_match(round[table].iter().map(|(_, score)| *score).collect());
                        round_vec.push(id);
                    }
                    match_ids.push(round_vec);
                }
                database.set_matches(id, match_ids);
            } else {
                alert("Haven't entered any scores");
            }
        } else {
            alert("Schedule is not in the database")
        }
    }
}

fn view_schedule_with_result_boxes<T: schedule::ScheduleStructure>(
    schedule: &T,
    players: &[u32],
    database: &database::Database,
    id: u32,
    score_inputs: &std::collections::HashMap<
        u32,
        std::collections::HashMap<(usize, usize, usize), usize>,
    >,
) -> Node<Msg> {
    table![style![St::BorderSpacing => "5px 10px"; ], {
        let tables = schedule.get_tables();

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
                        let player_list = schedule.get_players_from_game(round, table);
                        let mut data: Vec<Node<Msg>> = Vec::new();
                        for player_number in player_list {
                            let player_id = players[player_number];
                            if let Some(player) = database.get_player(player_id) {
                                data.push(span![
                                    style![St::Display => "Flex";
		St::FlexWrap => "Wrap";],
                                    span![style![St::FlexGrow => "1"], player.name],
                                    input![
                                        {
                                            if let Some(event_matches) = score_inputs.get(&id) {
                                                if let Some(score) = event_matches.get(&(
                                                    round,
                                                    table,
                                                    player_number,
                                                )) {
                                                    attrs! {At::Value => score}
                                                } else {
                                                    attrs! {}
                                                }
                                            } else {
                                                attrs! {}
                                            }
                                        },
                                        input_ev(Ev::Input, move |score| Msg::ARSetScore(
                                            id,
                                            round,
                                            table,
                                            player_number,
                                            score
                                        )),
                                        attrs! {At::Type => "number"},
                                        style! {St::Width => "3em"; St::FlexGrow => "0"}
                                    ],
                                    br![]
                                ]);
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
}

pub fn view_add_match_results(
    model: &AddMatchResults,
    database: &database::Database,
    style: &style_control::StyleControl,
) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        h2!["Event List"],
        table![
            style![St::PaddingBottom => "5px";],
            tr![
                td![style![St::PaddingRight => "25px";], "ID"],
                td![style![St::PaddingRight => "25px";], "Name"],
                td![style![St::PaddingRight => "25px";], "Date"],
            ],
            {
                let events_list = database.get_events();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(events_list.len());
                for (&id, event) in &events_list {
                    if event.matches.is_empty() {
                        node_list.push(tr![
                            td![id.to_string()],
                            td![event.name],
                            td![event.date],
                            td![format!("{:} players", event.players.len())],
                            td![if model.expanded_schedules.contains(&id) {
                                vec![
                                    button![
                                        style.button_style(),
                                        ev(Ev::Click, move |_| Msg::ARHideSchedule(id)),
                                        "Hide Schedule"
                                    ],
                                    button![
                                        style.button_style(),
                                        ev(Ev::Click, move |_| Msg::ARAddMatchResults(id)),
                                        "Add Entered Match Results"
                                    ],
                                ]
                            } else {
                                vec![button![
                                    style.button_style(),
                                    ev(Ev::Click, move |_| Msg::ARExpandSchedule(id)),
                                    "Show Schedule"
                                ]]
                            }],
                        ]);
                        node_list.push(tr![td![
                            attrs! {At::ColSpan => 10},
                            div![
                                style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
                                div![
                                    &box_style,
                                    if model.expanded_schedules.contains(&id) {
                                        view_schedule_with_result_boxes(
                                            &event.schedule,
                                            &event.players,
                                            database,
                                            id,
                                            &model.score_inputs,
                                        )
                                    } else {
                                        div![]
                                    }
                                ]
                            ],
                        ],]);
                    }
                }
                node_list
            }
        ],
    ]
}
