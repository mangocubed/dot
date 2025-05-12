use std::sync::LazyLock;

use fluent_templates::{StaticLoader, static_loader};
use leptos_fluent::leptos_fluent;
use leptos_meta::provide_meta_context;

use crate::prelude::*;
use crate::server_functions::{get_language, set_language};

static_loader! {
    static DOT_TRANSLATIONS = {
        locales: "./locales",
        fallback_language: "en",
    };
}

#[component]
pub fn AppProvider(translations: &'static LazyLock<StaticLoader>, children: Children) -> impl IntoView {
    provide_meta_context();

    leptos_fluent! {
        #[cfg(debug_assertions)]
        check_translations: "./src/**/*.rs",

        languages: "./locales/languages.yaml",
        locales: "./locales",
        sync_html_tag_dir: true,
        sync_html_tag_lang: true,
        translations: [DOT_TRANSLATIONS, translations],

        initial_language_from_server_function: get_language,
        set_language_to_server_function: set_language,

        children: children(),
    }
}
