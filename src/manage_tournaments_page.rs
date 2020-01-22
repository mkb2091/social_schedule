use seed::prelude::*;

use crate::{alert, database, prompt, style_control, Msg};

pub struct ManageTournaments {}

impl Default for ManageTournaments {
    fn default() -> Self {
        Self {}
    }
}

pub fn view_add_tournament(
    _model: &ManageTournaments,
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
            p![span!["Event name: "], input![],],
            button![style.button_style(), "Add"],
        ],
    ]
}
