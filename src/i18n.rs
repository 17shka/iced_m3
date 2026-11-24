use std::borrow::Cow;

#[cfg(feature = "rust-i18n")]
pub use rust_i18n::{locale, set_locale};

#[cfg(feature = "rust-i18n")]
use rust_i18n::Backend as _;

#[cfg(feature = "rust-i18n")]
#[must_use]
pub fn available_locales() -> Vec<(String, String)> {
    let Some(extension) = crate::_RUST_I18N_EXTENSION.get() else {
        return Vec::new();
    };
    let library = crate::_rust_i18n_backend();

    let mut locales: Vec<(String, String)> = crate::_rust_i18n_available_locales()
        .into_iter()
        .filter_map(|code| {
            extension
                .translate(&code, "language_name")
                .or_else(|| library.translate(&code, "language_name"))
                .map(|name| (code.into_owned(), name.into_owned()))
        })
        .collect();

    locales.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    locales
}

#[cfg(feature = "rust-i18n")]
fn translate(key: &'static str) -> Cow<'static, str> {
    rust_i18n::t!(key)
}

#[cfg(not(feature = "rust-i18n"))]
fn translate(key: &'static str) -> Cow<'static, str> {
    Cow::Borrowed(english::get(key))
}

#[cfg(not(feature = "rust-i18n"))]
mod english {
    use std::{collections::HashMap, sync::OnceLock};

    const YAML: &str = include_str!("../resources/locales/en.yml");

    static TABLE: OnceLock<HashMap<String, &'static str>> = OnceLock::new();

    pub(super) fn get(key: &'static str) -> &'static str {
        TABLE.get_or_init(parse).get(key).copied().unwrap_or(key)
    }

    fn parse() -> HashMap<String, &'static str> {
        let mut table = HashMap::new();
        let mut sections: Vec<(usize, &'static str)> = Vec::new();

        for line in YAML.lines() {
            let line = line.trim_end();
            let entry = line.trim_start();

            if entry.is_empty() || entry.starts_with('#') || entry.starts_with("---") {
                continue;
            }

            let indent = line.len() - entry.len();
            let Some((key, value)) = entry.split_once(':') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());

            while sections.last().is_some_and(|&(level, _)| level >= indent) {
                sections.pop();
            }

            if value.is_empty() {
                sections.push((indent, key));
                continue;
            }

            let mut path = String::new();
            for (_, section) in &sections {
                path.push_str(section);
                path.push('.');
            }
            path.push_str(key);

            let value = value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or(value);
            table.insert(path, value);
        }

        table
    }
}

macro_rules! widget_strings {
    ($($(#[$meta:meta])* $name:ident => $key:literal;)*) => {
        $(
            $(#[$meta])*
            pub(crate) fn $name() -> Cow<'static, str> {
                translate($key)
            }
        )*
    };
}

widget_strings! {
    dialog_title => "widget.dialog.title";
    dialog_close => "widget.dialog.close";
    select_search => "widget.select.search";
    select_clear => "widget.select.clear";
    select_clear_search => "widget.select.clear_search";
    navrail_expand_menu => "widget.navrail.expand_menu";
    navrail_collapse_menu => "widget.navrail.collapse_menu";
    fab_menu_open_menu => "widget.fab_menu.open_menu";
    fab_menu_close_menu => "widget.fab_menu.close_menu";
    theme_system => "widget.theme.system";
    theme_light => "widget.theme.light";
    theme_dark => "widget.theme.dark";
    text_input_undo => "widget.text_input.undo";
    text_input_redo => "widget.text_input.redo";
    text_input_copy => "widget.text_input.copy";
    text_input_cut => "widget.text_input.cut";
    text_input_paste => "widget.text_input.paste";
    text_input_select_word => "widget.text_input.select_word";
    text_input_select_all => "widget.text_input.select_all";
    text_input_clear => "widget.text_input.clear";
    text_input_show_password => "widget.text_input.show_password";
    text_input_hide_password => "widget.text_input.hide_password";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "rust-i18n"))]
    #[test]
    fn english_fallback_reads_the_bundled_locale() {
        assert_eq!(dialog_title(), "Title");
        assert_eq!(dialog_close(), "Close");
        assert_eq!(select_search(), "Search");
        assert_eq!(select_clear(), "Clear selection");
        assert_eq!(theme_system(), "System");
        assert_eq!(theme_light(), "Light");
        assert_eq!(theme_dark(), "Dark");
        assert_eq!(text_input_copy(), "Copy");
        assert_eq!(text_input_select_word(), "Select word");
        assert_eq!(text_input_clear(), "Clear");
        assert_eq!(translate("widget.unknown.key"), "widget.unknown.key");
    }

    #[cfg(feature = "rust-i18n")]
    #[test]
    fn available_locales_need_an_application_extension() {
        // Nothing is listed until the application calls `extend!`.
        assert!(available_locales().is_empty());
    }

    #[cfg(feature = "rust-i18n")]
    #[test]
    fn english_is_bundled_and_used_as_the_fallback() {
        let library = crate::_rust_i18n_backend();

        set_locale("en");
        assert_eq!(select_search(), "Search");
        assert_eq!(theme_dark(), "Dark");
        assert_eq!(
            library.translate("en", "language_name").as_deref(),
            Some("English")
        );

        // The library bundles no translation for unknown locales.
        set_locale("xx");
        assert_eq!(select_search(), "Search");

        set_locale("en");
    }
}
