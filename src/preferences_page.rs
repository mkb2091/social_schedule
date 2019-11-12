use seed::prelude::*;

use crate::style_control::Theme;
use crate::{style_control, Msg};

pub struct Preferences {
    theme_input: Option<Theme>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self { theme_input: None }
    }
}

impl Preferences {
    pub fn set_theme_input(&mut self, theme: String) {
        let theme: &str = &theme;
        self.theme_input = match theme {
            "Light" => Some(Theme::Light),
            "Dark" => Some(Theme::Dark),
            _ => None,
        }
    }
    pub fn set_theme(&mut self, style_control: &mut style_control::StyleControl) {
        if let Some(theme) = self.theme_input {
            style_control.set_theme(theme);
        }
    }
}

pub fn view_preferences(_model: &Preferences, style: &style_control::StyleControl) -> Node<Msg> {
    div![
        span!["Theme: "],
        select![
            style.button_style(),
            input_ev(Ev::Input, Msg::PSetThemeInput),
            option![style.option_style(), ""],
            option![style.option_style(), "Light"],
            option![style.option_style(), "Dark"]
        ],
        button![
            style.button_style(),
            simple_ev(Ev::Click, Msg::PSetTheme),
            "Set"
        ]
    ]
}
