macro_rules! maybe_assign {
    ($(#[$meta:meta])* $name:ident, $field:ident, $ty:ty $(,)?) => {
        $(#[$meta])*
        #[must_use]
        pub fn $name(mut self, value: $ty) -> Self {
            self.$field = value;
            self
        }
    };
}

macro_rules! maybe_delegate {
    ($(#[$meta:meta])* $name:ident, $setter:ident, $ty:ty $(,)?) => {
        $(#[$meta])*
        #[must_use]
        pub fn $name(self, value: $ty) -> Self {
            match value {
                Some(value) => self.$setter(value),
                None => self,
            }
        }
    };
}

macro_rules! maybe_set_some {
    ($(#[$meta:meta])* $name:ident, $field:ident, $ty:ty $(,)?) => {
        $(#[$meta])*
        #[must_use]
        pub fn $name(mut self, value: $ty) -> Self {
            if let Some(value) = value {
                self.$field = Some(value);
            }
            self
        }
    };
}

macro_rules! impl_element_from {
    ([$($generics:tt)*] $widget:ty => $element:ty $(where $($bounds:tt)*)?) => {
        impl<$($generics)*> ::core::convert::From<$widget> for $element
        $(where $($bounds)*)?
        {
            fn from(value: $widget) -> Self {
                ::iced::Element::new(value)
            }
        }
    };
}

pub(crate) use impl_element_from;
pub(crate) use maybe_assign;
pub(crate) use maybe_delegate;
pub(crate) use maybe_set_some;
