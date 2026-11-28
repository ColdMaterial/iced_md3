use common::section_header;
use iced::{
    Color, Element,
    widget::{center, column, row, text},
};
use iced_m3::{
    style::StateLayer,
    theme::{Accent, ColorScheme, Theme},
    widget::{
        Action,
        button::{self, Content as ButtonContent},
        card::{Interaction, MAX_CARD_BETWEEN_PADDING, Style as CardStyle},
    },
};

#[derive(Clone)]
pub enum Message {
    Pressed,
    SetNonInteractive,
    SetDisabled,
    SetPressable,
}

pub struct Page {
    press_count: u32,
    interaction: Interaction<'static, Message>,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            press_count: 0,
            interaction: Interaction::Press(Some(Action::Direct(Message::Pressed))),
        }
    }
}

const CARD_WIDTH: f32 = 250.0;
const CARD_HEIGHT: f32 = 150.0;

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Pressed => self.press_count += 1,
            Message::SetNonInteractive => self.interaction = Interaction::NonInteractive,
            Message::SetDisabled => self.interaction = Interaction::Press(None),
            Message::SetPressable => {
                self.interaction = Interaction::Press(Some(Action::Direct(Message::Pressed)))
            }
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let card = |style: CardStyle,
                    label: &'static str,
                    label_color: Color|
         -> Element<'static, Message> {
            let interaction = match &self.interaction {
                Interaction::NonInteractive => Interaction::NonInteractive,
                Interaction::Press(on_press) => {
                    Interaction::Press(on_press.as_ref().map(|m| Action::Direct(m.resolve())))
                }
            };
            let content: Element<'_, Message> =
                center(text(label).size(24.0).color(label_color)).into();
            iced_m3::widget::card(style, content)
                .interaction(interaction)
                .width(CARD_WIDTH)
                .height(CARD_HEIGHT)
                .into()
        };

        let interaction_buttons = row![
            iced_m3::widget::button(
                button::Style::elevated(theme, Accent::default()),
                ButtonContent::Label("Non-interactive".into())
            )
            .on_press(Message::SetNonInteractive),
            iced_m3::widget::button(
                button::Style::elevated(theme, Accent::default()),
                ButtonContent::Label("Disabled".into())
            )
            .on_press(Message::SetDisabled),
            iced_m3::widget::button(
                button::Style::elevated(theme, Accent::default()),
                ButtonContent::Label("Clickable".into())
            )
            .on_press(Message::SetPressable),
        ]
        .spacing(4.0);

        let cards = column![
            row![
                card(CardStyle::elevated(theme), "Elevated", theme.on_surface()),
                card(CardStyle::filled(theme), "Filled", theme.on_surface())
            ]
            .spacing(MAX_CARD_BETWEEN_PADDING),
            row![
                card(CardStyle::outlined(theme), "Outlined", theme.on_surface()),
                card(
                    CardStyle {
                        container_color: theme.tertiary_container(),
                        state_layer: StateLayer::new(theme.on_tertiary_container()),
                        ..CardStyle::filled(theme)
                    },
                    "Custom",
                    theme.on_tertiary_container()
                )
            ]
            .spacing(MAX_CARD_BETWEEN_PADDING)
        ]
        .spacing(MAX_CARD_BETWEEN_PADDING);

        column![
            section_header("Interaction", theme),
            interaction_buttons,
            section_header("Card styles", theme),
            cards,
            section_header("Press count", theme),
            text(format!("Pressed {} times", self.press_count)).color(theme.on_surface()),
        ]
        .spacing(24)
        .into()
    }
}

common::demo!(Page, Message, "Card Demo");
