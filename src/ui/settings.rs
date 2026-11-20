//! Settings panel shown from the application menu.

use iced::widget::{button, column, container, pick_list, row, text};
use iced::{Alignment, Element, Length, Theme};

use crate::config::ThemePreference;
use crate::ui::application::{Message, RuxApplication};

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl RuxApplication {
    pub(super) fn settings_panel(&self) -> Element<'_, Message> {
        let theme_picker = pick_list(
            ThemePreference::ALL.to_vec(),
            Some(self.theme_preference),
            Message::ThemePreferenceSelected,
        )
        .text_size(14)
        .padding([5, 10]);

        let sound_font_line = self.sound_font_display_path();
        let pick_sound_font = button("Choose file…").on_press(Message::PickSoundFontDialog);
        let clear_sound_font = button("Use built-in")
            .on_press(Message::ClearSoundFont)
            .style(button::secondary);

        let close = button("Close")
            .on_press(Message::CloseSettings)
            .style(button::secondary);

        let body = column![
            text("Settings").size(22),
            row![
                text("Theme").size(14).width(Length::Fixed(140.0)),
                theme_picker,
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            column![
                text("SoundFont").size(14),
                text(sound_font_line).size(13),
                row![pick_sound_font, clear_sound_font]
                    .spacing(8)
                    .align_y(Alignment::Center),
            ]
            .spacing(6),
            row![
                text("Version").size(14).width(Length::Fixed(140.0)),
                text(VERSION).size(14),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
            close,
        ]
        .spacing(16)
        .padding(20)
        .max_width(480);

        container(body)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.palette().background.into()),
                border: iced::Border::default()
                    .color(theme.palette().primary)
                    .width(1),
                ..Default::default()
            })
            .into()
    }

    fn sound_font_display_path(&self) -> String {
        self.sound_font_file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "Built-in (TimGM6mb)".to_string())
    }
}
