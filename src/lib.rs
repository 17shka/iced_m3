#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "rust-i18n")]
rust_i18n::i18n!("resources/locales", fallback = "en");

pub mod animation;
pub mod i18n;
pub mod spacing;
pub mod style;
pub mod theme;
pub mod widget;
