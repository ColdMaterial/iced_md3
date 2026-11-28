use iced::{
    Alignment, Element, Length,
    widget::{center, column, row, text},
};
use iced_m3::{
    animation::motion::{self, fast_spatial},
    spacing::SPACE_150,
    theme::{ColorScheme, Theme},
    widget::{Action, card, shape},
};

#[derive(Clone, Debug)]
pub enum Message {
    Select(Shape),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    #[default]
    Circle,
    Square,
    Slanted,
    Arch,
    Fan,
    Arrow,
    SemiCircle,
    Oval,
    Pill,
    Triangle,
    Diamond,
    ClamShell,
    Pentagon,
    Gem,
    Sunny,
    VerySunny,
    Cookie4Sided,
    Cookie6Sided,
    Cookie7Sided,
    Cookie9Sided,
    Cookie12Sided,
    Ghostish,
    Clover4Leaf,
    Clover8Leaf,
    Burst,
    SoftBurst,
    Boom,
    SoftBoom,
    Flower,
    Puffy,
    PuffyDiamond,
    PixelCircle,
    PixelTriangle,
    Bun,
    Heart,
}

impl Shape {
    pub const ALL: [Self; 35] = [
        Self::Circle,
        Self::Square,
        Self::Slanted,
        Self::Arch,
        Self::Fan,
        Self::Arrow,
        Self::SemiCircle,
        Self::Oval,
        Self::Pill,
        Self::Triangle,
        Self::Diamond,
        Self::ClamShell,
        Self::Pentagon,
        Self::Gem,
        Self::Sunny,
        Self::VerySunny,
        Self::Cookie4Sided,
        Self::Cookie6Sided,
        Self::Cookie7Sided,
        Self::Cookie9Sided,
        Self::Cookie12Sided,
        Self::Ghostish,
        Self::Clover4Leaf,
        Self::Clover8Leaf,
        Self::Burst,
        Self::SoftBurst,
        Self::Boom,
        Self::SoftBoom,
        Self::Flower,
        Self::Puffy,
        Self::PuffyDiamond,
        Self::PixelCircle,
        Self::PixelTriangle,
        Self::Bun,
        Self::Heart,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Circle => "Circle",
            Self::Square => "Square",
            Self::Slanted => "Slanted",
            Self::Arch => "Arch",
            Self::Fan => "Fan",
            Self::Arrow => "Arrow",
            Self::SemiCircle => "SemiCircle",
            Self::Oval => "Oval",
            Self::Pill => "Pill",
            Self::Triangle => "Triangle",
            Self::Diamond => "Diamond",
            Self::ClamShell => "ClamShell",
            Self::Pentagon => "Pentagon",
            Self::Gem => "Gem",
            Self::Sunny => "Sunny",
            Self::VerySunny => "VerySunny",
            Self::Cookie4Sided => "Cookie4Sided",
            Self::Cookie6Sided => "Cookie6Sided",
            Self::Cookie7Sided => "Cookie7Sided",
            Self::Cookie9Sided => "Cookie9Sided",
            Self::Cookie12Sided => "Cookie12Sided",
            Self::Ghostish => "Ghostish",
            Self::Clover4Leaf => "Clover4Leaf",
            Self::Clover8Leaf => "Clover8Leaf",
            Self::Burst => "Burst",
            Self::SoftBurst => "SoftBurst",
            Self::Boom => "Boom",
            Self::SoftBoom => "SoftBoom",
            Self::Flower => "Flower",
            Self::Puffy => "Puffy",
            Self::PuffyDiamond => "PuffyDiamond",
            Self::PixelCircle => "PixelCircle",
            Self::PixelTriangle => "PixelTriangle",
            Self::Bun => "Bun",
            Self::Heart => "Heart",
        }
    }

    pub fn to_rounded_polygon(self) -> iced_m3::widget::shapes::polymorpher::RoundedPolygon {
        use iced_m3::widget::shapes::polymorpher::shapes;

        match self {
            Self::Circle => shapes::circle(None),
            Self::Square => shapes::square(),
            Self::Slanted => shapes::slanted(),
            Self::Arch => shapes::arch(),
            Self::Fan => shapes::fan(),
            Self::Arrow => shapes::arrow(),
            Self::SemiCircle => shapes::semi_circle(),
            Self::Oval => shapes::oval(),
            Self::Pill => shapes::pill(),
            Self::Triangle => shapes::triangle(),
            Self::Diamond => shapes::diamond(),
            Self::ClamShell => shapes::clam_shell(),
            Self::Pentagon => shapes::pentagon(),
            Self::Gem => shapes::gem(),
            Self::Sunny => shapes::sunny(),
            Self::VerySunny => shapes::very_sunny(),
            Self::Cookie4Sided => shapes::cookie4(),
            Self::Cookie6Sided => shapes::cookie6(),
            Self::Cookie7Sided => shapes::cookie7(),
            Self::Cookie9Sided => shapes::cookie9(),
            Self::Cookie12Sided => shapes::cookie12(),
            Self::Ghostish => shapes::ghostish(),
            Self::Clover4Leaf => shapes::clover4(),
            Self::Clover8Leaf => shapes::clover8(),
            Self::Burst => shapes::burst(),
            Self::SoftBurst => shapes::soft_burst(),
            Self::Boom => shapes::boom(),
            Self::SoftBoom => shapes::soft_boom(),
            Self::Flower => shapes::flower(),
            Self::Puffy => shapes::puffy(),
            Self::PuffyDiamond => shapes::puffy_diamond(),
            Self::PixelCircle => shapes::pixel_circle(),
            Self::PixelTriangle => shapes::pixel_triangle(),
            Self::Bun => shapes::bun(),
            Self::Heart => shapes::heart(),
        }
    }
}

impl From<Shape> for iced_m3::widget::shapes::polymorpher::RoundedPolygon {
    fn from(shape: Shape) -> Self {
        shape.to_rounded_polygon()
    }
}

#[derive(Default)]
pub struct Page {
    selected: Shape,
}

fn shape_widget(selected: Shape, theme: &Theme) -> impl Into<Element<'static, Message>> {
    shape(
        selected.to_rounded_polygon(),
        iced_m3::widget::shapes::Style::new(theme),
    )
    .size(48.0)
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Select(shape) => self.selected = shape,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let shape_cards = Shape::ALL.iter().map(|shape| {
            let is_selected = *shape == self.selected;
            let preview = center(shape_widget(*shape, theme))
                .width(Length::Fill)
                .height(72.0);
            let content = column![
                preview,
                text(shape.as_str()).size(14.0).color(theme.on_surface()),
            ]
            .spacing(8)
            .align_x(Alignment::Center);

            card(
                if is_selected {
                    card::Style::filled(theme)
                } else {
                    card::Style::outlined(theme)
                },
                content,
            )
            .width(124.0)
            .height(124.0)
            .padding(8.0)
            .interaction(card::Interaction::Press(Some(Action::Direct(
                Message::Select(*shape),
            ))))
            .into()
        });

        column![
            text("Material shapes")
                .font(fonts::text_bold())
                .size(24.0)
                .color(theme.on_surface()),
            shape(
                self.selected.to_rounded_polygon(),
                iced_m3::widget::shapes::Style::new(theme)
            )
            .overshoot_margin(iced_m3::widget::shapes::OvershootMargin::Auto)
            .transition_spring(fast_spatial(motion::Scheme::Expressive))
            .size(200.0),
            text(format!("Selected: {}", self.selected.as_str())).color(theme.on_surface()),
            row(shape_cards).spacing(SPACE_150).wrap(),
        ]
        .spacing(20)
        .into()
    }
}

common::demo!(Page, Message, "Shapes Demo", bare);
