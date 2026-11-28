use common::section_header;
use iced::{
    Alignment, Element,
    widget::{column, row, text},
};
use iced_m3::theme::{ColorScheme, Theme};
use iced_m3::widget::{hybrid_icon::Icon, range_slider, slider};

#[derive(Clone)]
pub enum Message {
    ValueXs(f32),
    ValueS(f32),
    ValueM(f32),
    ValueL(f32),
    ValueXl(f32),
    Centered(f32),
    Stepped(f32),
    Ticks(f32),
    Volume(f32),
    Percent(f32),
    Decimals(f32),
    Range(f32, f32),
    RangeStepped(f32, f32),
    RangeSmooth(f32, f32),
    RangeSeparated(f32, f32),
    RangeOneHandle(f32, f32),
    Vertical(f32),
    VerticalRange(f32, f32),
    Disabled,
}

pub struct Page {
    value_xs: f32,
    value_s: f32,
    value_m: f32,
    value_l: f32,
    value_xl: f32,
    centered: f32,
    stepped: f32,
    ticks: f32,
    volume: f32,
    percent: f32,
    decimals: f32,
    range: (f32, f32),
    range_stepped: (f32, f32),
    range_smooth: (f32, f32),
    range_separated: (f32, f32),
    range_one_handle: (f32, f32),
    vertical: f32,
    vertical_range: (f32, f32),
}

impl Default for Page {
    fn default() -> Self {
        Self {
            value_xs: 20.0,
            value_s: 40.0,
            value_m: 60.0,
            value_l: 80.0,
            value_xl: 65.0,
            centered: 0.0,
            stepped: 50.0,
            ticks: 30.0,
            volume: 70.0,
            percent: 42.0,
            decimals: 42.25,
            range: (25.0, 75.0),
            range_stepped: (25.0, 75.0),
            range_smooth: (30.0, 70.0),
            range_separated: (20.0, 70.0),
            range_one_handle: (20.0, 60.0),
            vertical: 40.0,
            vertical_range: (30.0, 80.0),
        }
    }
}

impl Page {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ValueXs(v) => self.value_xs = v,
            Message::ValueS(v) => self.value_s = v,
            Message::ValueM(v) => self.value_m = v,
            Message::ValueL(v) => self.value_l = v,
            Message::ValueXl(v) => self.value_xl = v,
            Message::Centered(v) => self.centered = v,
            Message::Stepped(v) => self.stepped = v,
            Message::Ticks(v) => self.ticks = v,
            Message::Volume(v) => self.volume = v,
            Message::Percent(v) => self.percent = v,
            Message::Decimals(v) => self.decimals = v,
            Message::Range(start, end) => self.range = (start, end),
            Message::RangeStepped(start, end) => self.range_stepped = (start, end),
            Message::RangeSmooth(start, end) => self.range_smooth = (start, end),
            Message::RangeSeparated(start, end) => self.range_separated = (start, end),
            Message::RangeOneHandle(start, end) => self.range_one_handle = (start, end),
            Message::Vertical(v) => self.vertical = v,
            Message::VerticalRange(start, end) => self.vertical_range = (start, end),
            Message::Disabled => {}
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::none()
    }

    pub fn view(&self, theme: &Theme) -> Element<'_, Message> {
        let style = slider::Style::new(theme);

        let sizes = column![
            labeled_slider(
                theme,
                "XS",
                self.value_xs,
                Message::ValueXs,
                style,
                slider::Size::extra_small()
            ),
            labeled_slider(
                theme,
                "S",
                self.value_s,
                Message::ValueS,
                style,
                slider::Size::small()
            ),
            labeled_slider(
                theme,
                "M",
                self.value_m,
                Message::ValueM,
                style,
                slider::Size::medium()
            ),
            labeled_slider(
                theme,
                "L",
                self.value_l,
                Message::ValueL,
                style,
                slider::Size::large()
            ),
            labeled_slider(
                theme,
                "XL",
                self.value_xl,
                Message::ValueXl,
                style,
                slider::Size::extra_large()
            ),
        ]
        .spacing(20);

        let centered_slider = slider(-100.0..=100.0, self.centered, Message::Centered, style)
            .centered(true)
            .step(0.0_f32)
            .value_indicator(slider::ValueIndicator::Value);

        let steps = column![
            slider(0.0..=100.0, self.stepped, Message::Stepped, style)
                .step(25.0_f32)
                .continuous_tick_count(4)
                .value_indicator(slider::ValueIndicator::Value),
            slider(0.0..=100.0, self.ticks, Message::Ticks, style)
                .step(0.0_f32)
                .continuous_tick_count(8)
                .value_indicator(slider::ValueIndicator::Value),
            slider(0.0..=100.0, self.volume, Message::Volume, style)
                .stops(false)
                .inset_icon_start(slider::InsetIcon::new(Icon::Text {
                    text: "volume_down".into(),
                    font: Some(fonts::icons_filled()),
                }))
                .inset_icon_end(slider::InsetIcon::new(Icon::Text {
                    text: "volume_up".into(),
                    font: Some(fonts::icons_filled()),
                }))
                .size(slider::Size::medium())
                .value_indicator(slider::ValueIndicator::Value),
        ]
        .spacing(20);

        let percent_slider = slider(0.0..=100.0, self.percent, Message::Percent, style)
            .step(0.1_f32)
            .value_indicator(slider::ValueIndicator::Format(Box::new(|value: f32| {
                format!("{value:.1}%")
            })));

        let decimals_slider = slider(0.0..=100.0, self.decimals, Message::Decimals, style)
            .step(0.25_f32)
            .decimal_places(2)
            .value_indicator(slider::ValueIndicator::Value);

        let disabled = slider(0.0..=100.0, 70.0, |_| Message::Disabled, style)
            .on_change_maybe(None as Option<fn(f32) -> Message>)
            .value_indicator(slider::ValueIndicator::Value);

        let ranges = column![
            range_slider(
                0.0..=100.0,
                self.range.0,
                self.range.1,
                Message::Range,
                style
            )
            .value_indicator(slider::ValueIndicator::Value),
            range_slider(
                0.0..=100.0,
                self.range_stepped.0,
                self.range_stepped.1,
                Message::RangeStepped,
                style
            )
            .step(25.0_f32)
            .value_indicator(slider::ValueIndicator::Value),
            range_slider(
                0.0..=100.0,
                self.range_smooth.0,
                self.range_smooth.1,
                Message::RangeSmooth,
                style
            )
            .step(1.0_f32)
            .value_indicator(slider::ValueIndicator::Value),
            range_slider(
                0.0..=100.0,
                self.range_separated.0,
                self.range_separated.1,
                Message::RangeSeparated,
                style
            )
            .min_separation(20.0_f32)
            .value_indicator(slider::ValueIndicator::Value),
        ]
        .spacing(20);

        let range_one_handle = range_slider(
            0.0..=100.0,
            self.range_one_handle.0,
            self.range_one_handle.1,
            Message::RangeOneHandle,
            style,
        )
        .lower_enabled(false)
        .value_indicator(slider::ValueIndicator::Value);

        let vertical = row![
            slider(0.0..=100.0, self.vertical, Message::Vertical, style)
                .vertical()
                .height(300)
                .stops(false)
                .value_indicator(slider::ValueIndicator::Value),
            range_slider(
                0.0..=100.0,
                self.vertical_range.0,
                self.vertical_range.1,
                Message::VerticalRange,
                style
            )
            .vertical()
            .height(300)
            .stops(false)
            .value_indicator(slider::ValueIndicator::Value),
        ]
        .spacing(24)
        .align_y(Alignment::End);

        column![
            section_header("Sizes", theme),
            sizes,
            section_header("Centered (the active track grows from the middle)", theme),
            centered_slider,
            section_header("Steps, continuous ticks and inset icons", theme),
            steps,
            section_header("Formatted value indicator", theme),
            percent_slider,
            section_header("Value precision (step 0.25, two decimal places)", theme),
            decimals_slider,
            section_header("Disabled", theme),
            disabled,
            section_header("Range sliders (the handles can pass each other)", theme),
            ranges,
            section_header("Range slider with a disabled lower handle", theme),
            range_one_handle,
            section_header("Vertical sliders", theme),
            vertical,
        ]
        .spacing(24)
        .into()
    }
}

fn labeled_slider<'a>(
    theme: &Theme,
    label: &'a str,
    value: f32,
    on_change: impl 'a + Fn(f32) -> Message,
    style: slider::Style,
    size: slider::Size,
) -> iced::widget::Row<'a, Message> {
    row![
        text(label).width(40).color(theme.on_surface_variant()),
        slider(0.0..=100.0, value, on_change, style)
            .value_indicator(slider::ValueIndicator::Value)
            .stops(false)
            .size(size),
    ]
    .spacing(16)
    .align_y(Alignment::Center)
}

common::demo!(Page, Message, "Sliders Demo");
