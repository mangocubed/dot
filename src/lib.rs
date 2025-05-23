use std::sync::LazyLock;

use fluent_templates::{StaticLoader, static_loader};
use leptos::prelude::{AddAnyAttr, Children, Get, IntoAnyAttribute, IntoMaybeErased, IntoView, component};
use leptos_fluent::leptos_fluent;
use leptos_meta::provide_meta_context;

pub mod forms;
pub mod icons;

#[cfg(feature = "server")]
pub mod server;

mod server_functions;

use crate::server_functions::{get_language, set_language};

pub mod meta {
    pub use leptos_meta::Title;
}

pub mod prelude {
    pub use leptos::either::{Either, EitherOf3};
    pub use leptos::prelude::{
        ClassAttribute, ElementChild, IntoMaybeErased, IntoView, ReadSignal, ServerAction, ServerFnError, Signal,
        ViewFn, component, server, signal, view,
    };
    pub use leptos_fluent::{move_tr, tr};
}

pub mod router {
    pub use leptos_router::StaticSegment;
    pub use leptos_router::components::{Route, Router, Routes};
}

#[cfg(feature = "hydrate")]
pub fn hydrate_body<IV>(app_fn: fn() -> IV)
where
    IV: leptos::prelude::IntoView + 'static,
{
    console_error_panic_hook::set_once();

    leptos::mount::hydrate_body(app_fn)
}

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
