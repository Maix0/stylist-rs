use std::borrow::Cow;

use crate::{Result, Style};

/// A trait to create Style
///
/// Any struct that implements this trait can call [`self.style()`](YieldStyle::style) to get a style class.
///
/// [`prefix()`](YieldStyle::prefix) and [`style_str()`](YieldStyle::style_str) will be called everytime
/// [`self.style()`](YieldStyle::style) is called.
///
/// You can use this to achieve dynamic theming.
///
/// # Example:
///
/// ```rust
/// use yew::prelude::*;
///
/// use std::borrow::Cow;
/// use stylist::YieldStyle;
///
/// struct MyStyledComponent {}
///
/// impl Component for MyStyledComponent {
///     type Message = ();
///     type Properties = ();
///
///     fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
///         Self {}
///     }
///
///     fn change(&mut self, _: Self::Properties) -> ShouldRender {
///         false
///     }
///
///     fn update(&mut self, _: Self::Message) -> ShouldRender {
///         false
///     }
///
///     fn view(&self) -> Html {
///         html! {<div class=self.style()>{"Hello World!"}</div>}
///     }
/// }
///
/// impl YieldStyle for MyStyledComponent {
///     fn style_str(&self) -> Cow<'static, str> {
///         "color: red;".into()
///     }
/// }
/// ```
pub trait YieldStyle {
    /// Returns the prefix to use in the style.
    ///
    /// Override this if you want to use a custom style prefix.
    ///
    /// By default, the prefix is `stylist`.
    fn prefix(&self) -> Cow<'static, str> {
        "stylist".into()
    }

    /// Returns the raw style string.
    fn style_str(&self) -> Cow<'static, str>;

    /// Returns the generated style.
    ///
    /// Returns [`Err(Error)`](crate::Error) when failed to create a style.
    fn try_style(&self) -> Result<Style> {
        Style::new(self.style_str())
    }

    /// Returns the generated style.
    ///
    /// # Panics
    ///
    /// Panics if [`try_style`](YieldStyle::try_style) returns [`Err(Error)`](crate::Error).
    fn style(&self) -> Style {
        self.try_style().expect("Failed to create style.")
    }

    /// Returns the class name of the generated style.
    ///
    /// Returns [`Err(Error)`](crate::Error) when failed to create a style.
    fn try_style_class(&self) -> Result<String> {
        Ok(self.try_style()?.get_class_name().to_string())
    }

    /// Returns the class name of the generated style.
    ///
    /// # Panics
    ///
    /// Panics if [`try_style_class`](YieldStyle::try_style) returns [`Err(Error)`](crate::Error).
    fn style_class(&self) -> String {
        self.try_style_class().expect("Failed to create style.")
    }
}