use std::{borrow::Cow, fmt};

use iced::advanced::{svg, text::Fragment};
use iced_widget::core::keyboard::{self, key};
use iced_widget::text::IntoFragment;

use crate::widget::common_icons;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    SelectWord,
    SelectAll,
}

impl Action {
    pub const ALL: [Self; 7] = [
        Self::Copy,
        Self::Cut,
        Self::Paste,
        Self::SelectWord,
        Self::SelectAll,
        Self::Undo,
        Self::Redo,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Copy => "Copy",
            Self::Cut => "Cut",
            Self::Paste => "Paste",
            Self::SelectWord => "Select word",
            Self::SelectAll => "Select all",
        }
    }

    pub fn shortcut(self) -> Option<Shortcut> {
        let command = keyboard::Modifiers::COMMAND;

        Some(match self {
            Self::Undo => Shortcut::character(command, 'z'),
            #[cfg(target_os = "macos")]
            Self::Redo => Shortcut::character(command | keyboard::Modifiers::SHIFT, 'z'),
            #[cfg(not(target_os = "macos"))]
            Self::Redo => Shortcut::character(command, 'y'),
            Self::Copy => Shortcut::character(command, 'c'),
            Self::Cut => Shortcut::character(command, 'x'),
            Self::Paste => Shortcut::character(command, 'v'),
            Self::SelectWord => return None,
            Self::SelectAll => Shortcut::character(command, 'a'),
        })
    }

    pub fn shortcut_text(self) -> Option<String> {
        self.shortcut().map(|shortcut| shortcut.to_string())
    }

    pub(super) fn icon(self) -> svg::Handle {
        match self {
            Self::Undo => common_icons::UNDO.clone(),
            Self::Redo => common_icons::REDO.clone(),
            Self::Copy => common_icons::COPY.clone(),
            Self::Cut => common_icons::CUT.clone(),
            Self::Paste => common_icons::PASTE.clone(),
            Self::SelectWord => common_icons::SELECT_WORD.clone(),
            Self::SelectAll => common_icons::SELECT_ALL.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionLabels<'a> {
    pub undo: Fragment<'a>,
    pub redo: Fragment<'a>,
    pub copy: Fragment<'a>,
    pub cut: Fragment<'a>,
    pub paste: Fragment<'a>,
    pub select_word: Fragment<'a>,
    pub select_all: Fragment<'a>,
}

impl Default for ActionLabels<'_> {
    fn default() -> Self {
        Self {
            undo: Cow::Borrowed(Action::Undo.label()),
            redo: Cow::Borrowed(Action::Redo.label()),
            copy: Cow::Borrowed(Action::Copy.label()),
            cut: Cow::Borrowed(Action::Cut.label()),
            paste: Cow::Borrowed(Action::Paste.label()),
            select_word: Cow::Borrowed(Action::SelectWord.label()),
            select_all: Cow::Borrowed(Action::SelectAll.label()),
        }
    }
}

impl<'a> ActionLabels<'a> {
    pub fn get(&self, action: Action) -> &str {
        match action {
            Action::Undo => self.undo.as_ref(),
            Action::Redo => self.redo.as_ref(),
            Action::Copy => self.copy.as_ref(),
            Action::Cut => self.cut.as_ref(),
            Action::Paste => self.paste.as_ref(),
            Action::SelectWord => self.select_word.as_ref(),
            Action::SelectAll => self.select_all.as_ref(),
        }
    }

    pub fn set(&mut self, action: Action, label: impl IntoFragment<'a>) {
        *self.field_mut(action) = label.into_fragment();
    }

    fn field_mut(&mut self, action: Action) -> &mut Fragment<'a> {
        match action {
            Action::Undo => &mut self.undo,
            Action::Redo => &mut self.redo,
            Action::Copy => &mut self.copy,
            Action::Cut => &mut self.cut,
            Action::Paste => &mut self.paste,
            Action::SelectWord => &mut self.select_word,
            Action::SelectAll => &mut self.select_all,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shortcut {
    pub modifiers: keyboard::Modifiers,
    pub key: keyboard::Key,
}

impl Shortcut {
    pub fn new(modifiers: keyboard::Modifiers, key: keyboard::Key) -> Self {
        Self { modifiers, key }
    }

    pub fn character(modifiers: keyboard::Modifiers, character: char) -> Self {
        Self::new(
            modifiers,
            keyboard::Key::Character(character.to_string().into()),
        )
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let modifiers = self.modifiers;

        if modifiers.control() {
            f.write_str(if cfg!(target_os = "macos") {
                "⌃"
            } else {
                "Ctrl+"
            })?;
        }

        if modifiers.alt() {
            f.write_str(if cfg!(target_os = "macos") {
                "⌥"
            } else {
                "Alt+"
            })?;
        }

        if modifiers.shift() {
            f.write_str(if cfg!(target_os = "macos") {
                "⇧"
            } else {
                "Shift+"
            })?;
        }

        if modifiers.logo() {
            f.write_str(if cfg!(target_os = "macos") {
                "⌘"
            } else {
                "Super+"
            })?;
        }

        f.write_str(&key_label(&self.key))
    }
}

fn key_label(key: &keyboard::Key) -> String {
    match key {
        keyboard::Key::Character(character) => character.to_uppercase(),
        keyboard::Key::Named(named) => named_label(*named)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("{named:?}")),
        keyboard::Key::Unidentified => "?".to_owned(),
    }
}

fn named_label(named: key::Named) -> Option<&'static str> {
    Some(match named {
        key::Named::Enter => "Enter",
        key::Named::Space => "Space",
        key::Named::Tab => "Tab",
        key::Named::Escape => "Esc",
        key::Named::Backspace => "Backspace",
        key::Named::Delete => "Delete",
        key::Named::Insert => "Insert",
        key::Named::Home => "Home",
        key::Named::End => "End",
        key::Named::PageUp => "PageUp",
        key::Named::PageDown => "PageDown",
        key::Named::ArrowUp => "↑",
        key::Named::ArrowDown => "↓",
        key::Named::ArrowLeft => "←",
        key::Named::ArrowRight => "→",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_action_has_a_label() {
        for action in Action::ALL {
            assert!(!action.label().is_empty());
        }
    }

    #[test]
    fn action_labels_default_to_the_english_names() {
        let labels = ActionLabels::default();

        for action in Action::ALL {
            assert_eq!(labels.get(action), action.label());
        }
    }

    #[test]
    fn action_labels_override_a_single_action() {
        let mut labels = ActionLabels::default();
        labels.set(Action::Copy, "Копировать");

        assert_eq!(labels.get(Action::Copy), "Копировать");

        for action in Action::ALL
            .into_iter()
            .filter(|action| *action != Action::Copy)
        {
            assert_eq!(labels.get(action), action.label());
        }
    }

    #[test]
    fn select_word_has_no_shortcut() {
        assert!(Action::SelectWord.shortcut().is_none());
        assert!(Action::SelectWord.shortcut_text().is_none());
    }

    #[test]
    fn shortcuts_match_the_context_menu() {
        for action in Action::ALL
            .into_iter()
            .filter(|action| *action != Action::SelectWord)
        {
            assert!(
                action.shortcut().is_some(),
                "{action:?} should have a shortcut"
            );
        }
    }

    #[test]
    fn shortcuts_are_rendered_for_the_platform() {
        let copy = Action::Copy.shortcut().expect("a shortcut");

        assert_eq!(copy.modifiers, keyboard::Modifiers::COMMAND);

        #[cfg(target_os = "macos")]
        assert_eq!(copy.to_string(), "⌘C");

        #[cfg(not(target_os = "macos"))]
        assert_eq!(copy.to_string(), "Ctrl+C");
    }

    #[test]
    fn named_keys_are_rendered() {
        let shortcut = Shortcut::new(
            keyboard::Modifiers::COMMAND,
            keyboard::Key::Named(key::Named::Enter),
        );

        #[cfg(target_os = "macos")]
        assert_eq!(shortcut.to_string(), "⌘Enter");

        #[cfg(not(target_os = "macos"))]
        assert_eq!(shortcut.to_string(), "Ctrl+Enter");
    }
}
