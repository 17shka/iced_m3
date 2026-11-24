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
Some widgets ship with text of their own (the `TextInput` context menu, `Dialog`, `Select` search,
`NavRail` toggles, `FabMenu` actions, the theme picker). English is bundled; other languages come
from your application through [`rust-i18n`](https://crates.io/crates/rust-i18n), enabled with the
`rust-i18n` feature:

```toml
iced_m3 = { version = "...", features = ["rust-i18n"] }
```

Register the library once at startup and override keys under the `iced_m3` namespace. Both crates
must use the same `rust-i18n` version.

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
    text_input: { copy: "Копировать", paste: "Вставить", select_all: "Выделить всё" }
```

Only keys you want to change are needed — the rest fall back to English. Switch languages with
`rust_i18n::set_locale` (re-exported as `iced_m3::i18n::set_locale`);
`iced_m3::i18n::available_locales()` returns the `(code, name)` pairs. Full example:
`demos/localization`.

`language_select` and `theme_select` wrap this into a ready-made `Select`:

```rust,ignore
widget::language_select(theme, Some(locale), Message::SetLocale);
widget::theme_select(theme, theme.mode, Message::SetTheme);
```

For `Mode::System`, store the platform preference in `Theme::system_mode`, the user's choice in
`Theme::mode`, and paint `Theme::resolved_mode()`.

Without the feature, English text is embedded and `rust-i18n` is not required.
