use iced::advanced::svg;
use std::sync::LazyLock;

macro_rules! svg_handles {
    ($($name:ident => $file:literal),* $(,)?) => {
        $(
            pub static $name: LazyLock<svg::Handle> = LazyLock::new(|| {
                svg::Handle::from_memory(include_bytes!(concat!("../../resources/svg/", $file)))
            });
        )*
    };
}

svg_handles! {
    CHECK => "check.svg",
    CLOSE => "close.svg",
    MENU => "menu.svg",
    MENU_OPEN => "menu_open.svg",
    ADD => "add.svg",
    ARROW_RIGHT => "arrow_right.svg",
    VISIBILITY => "visibility.svg",
    VISIBILITY_OFF => "visibility_off.svg",
    COPY => "content_copy.svg",
    CUT => "content_cut.svg",
    PASTE => "content_paste.svg",
    SELECT_WORD => "highlight_alt.svg",
    SELECT_ALL => "select_all.svg",
    SEARCH => "search.svg",
    UNDO => "undo.svg",
    REDO => "redo.svg",
}
