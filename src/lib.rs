use std::sync::LazyLock;

use fluent_templates::{StaticLoader, static_loader};
use leptos::prelude::{
    AddAnyAttr, Children, ClassAttribute, Effect, ElementChild, Get, IntoAnyAttribute, IntoMaybeErased, IntoView,
    RwSignal, Set, component, view,
};
use leptos_fluent::leptos_fluent;
use leptos_meta::provide_meta_context;

pub mod components;
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
        Children, ChildrenFn, ClassAttribute, ElementChild, Get, IntoMaybeErased, IntoView, OnAttribute, ReadSignal,
        Resource, RwSignal, ServerAction, ServerFnError, Set, Signal, StoredValue, Suspend, Transition, ViewFn,
        component, server, signal, view,
    };
    pub use leptos_fluent::{move_tr, tr};
}

pub mod router {
    pub use leptos_router::StaticSegment;
    pub use leptos_router::components::{Redirect, Route, Router, Routes};
    pub use leptos_router::hooks::use_navigate;
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
pub fn AppProvider(
    #[prop(optional)] loading_icon_src: Option<&'static str>,
    translations: &'static LazyLock<StaticLoader>,
    children: Children,
) -> impl IntoView {
    provide_meta_context();

    let is_done = RwSignal::new(false);

    Effect::new(move || is_done.set(true));

    view! {
        <I18nProvider translations=translations>
            <div>{children()}</div>

            <div class="loading-overlay" class:is-done=is_done>
                <figure>
                    <div class="loading-pulse"></div>
                    {loading_icon_src.map(|icon_src| view! { <img src=icon_src /> })}
                </figure>
            </div>
        </I18nProvider>
    }
}

#[component]
fn I18nProvider(translations: &'static LazyLock<StaticLoader>, children: Children) -> impl IntoView {
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
