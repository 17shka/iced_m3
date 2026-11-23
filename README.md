<h1 align="center">iced_m3</h1>

<div align="center">
  
  ![](https://img.shields.io/github/last-commit/ColdMaterial/iced_md3?&style=for-the-badge&color=FFFFFF&logo=git&logoColor=C9C9C9&labelColor=252525)
  ![](https://img.shields.io/github/repo-size/ColdMaterial/iced_md3?&style=for-the-badge&color=FFFFFF&logo=git&logoColor=C9C9C9&labelColor=252525)
</div>

<div align="center">
  Implementation of the <a href="https://m3.material.io">Material Design 3</a> system for the <a href="https://iced.rs/">Iced GUI library</a> 
</div>

# Features
- [Motion Physics](https://m3.material.io/styles/motion/overview)
- [Dynamic Color](https://m3.material.io/styles/color/system)
- [Elevation](https://m3.material.io/styles/elevation/overview)
- Support for text or SVG icons
- Optional translations of built-in widget strings via [`rust-i18n`](https://crates.io/crates/rust-i18n)


Try the web  [showcase](https://coldmaterial.github.io/iced_m3/)!

You can go see the demos [here](https://github.com/ColdMaterial/iced_md3/blob/main/demos/)!

[Chilen](https://github.com/tpaau/chilen) uses `iced_m3` extensively, you can check how the library
is used there too!


# Usage
Run this in your project:
```bash
cargo add iced_m3 --git https://github.com/ColdMaterial/iced_md3.git
```

This library is largely for use in my personal projects (like
[Chilen](https://github.com/tpaau/chilen)), and constantly changing, so consider pinning a commit if
you wish to use it in your project. I also don't plan on releasing this on
[crates.io](https://crates.io) anytime soon, largely for the same reason.

`iced_m3` is compatible with Iced **`0.14.0`** and will most likely be kept up to date with stable
Iced releases.

# Advanced
Widgets used internally by `iced_m3` for other widgets that may also be useful outside of `iced_m3`.
Those are gated behind the `advanced` feature.

# Localization
Some widgets ship with their own text: the `TextInput` context menu, the default `Dialog` title,
the `Select` search placeholder. English is bundled (`resources/locales/en.yml`); every other
language comes from your application. Enable the `rust-i18n` feature (alias `i18n`) to translate
them:

```toml
iced_m3 = { version = "...", features = ["rust-i18n"] }
```

Register the library once at startup and provide its keys under the `iced_m3` namespace. The
application and `iced_m3` must depend on the same `rust-i18n` version.

```rust,ignore
rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    rust_i18n::extend!(iced_m3);
    // ...
}
```

```yaml
# locales/ru.yml
iced_m3:
  language_name: "Русский" # required for every locale except `en`
  widget:
    dialog: { title: "Заголовок" }
    select: { search: "Поиск" }
    theme: { system: "Системная", light: "Светлая", dark: "Тёмная" }
    text_input:
      undo: "Отменить"
      redo: "Повторить"
      copy: "Копировать"
      cut: "Вырезать"
      paste: "Вставить"
      select_word: "Выделить слово"
      select_all: "Выделить всё"
```

Missing keys fall back to the bundled English, so `locales/en.yml` can omit the `iced_m3` section
entirely. Switch locales with `rust_i18n::set_locale` (re-exported as
`iced_m3::i18n::set_locale`); see `demos/localization` for a complete example.

## Pickers
Both helpers return a regular `Select`, so its builders (`.width()`, `.placeholder()`, …) apply:

```rust,ignore
// Locales of the application plus English from the library.
iced_m3::widget::language_select(theme, Some(current), Message::SetLocale)
    .width(Length::Fixed(280.0));

// System preference (the default), light and dark.
iced_m3::widget::theme_select(theme, theme.mode, Message::SetTheme)
    .width(Length::Fixed(220.0));
```

`iced_m3::i18n::available_locales()` exposes the `(code, name)` list if you want a custom control.
`Mode::System` follows `iced::system::theme_changes()`: store it in `Theme::system_mode`, keep the
picker's value in `Theme::mode`, and read the painted mode from `Theme::resolved_mode()`.

With the feature disabled, the English fallback is embedded, so no extra dependency is required.
