use std::collections::HashMap;

use leptos::either::{Either, EitherOf3};
use leptos::ev;
use leptos::prelude::{
    ActionForm, AddAnyAttr, BindAttribute, Children, ChildrenFn, ClassAttribute, ElementChild, For, Get,
    GlobalAttributes, IntoAnyAttribute, IntoMaybeErased, IntoView, NodeRef, NodeRefAttribute, OnAttribute, RwSignal,
    ServerAction, ServerFnError, Signal, Update, ViewFn, component, provide_context, use_context, view,
};
use leptos::server_fn::{Http, ServerFn, client, codec, request};
use leptos_fluent::move_tr;
use leptos_use::use_event_listener;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::icons::{EyeMini, EyeSlashMini};

const KEY_CODE_ENTER: u32 = 13;

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct ActionResponse<T = ()> {
    pub success: Option<bool>,
    pub errors: Option<HashMap<String, String>>,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Clone)]
struct FormContext {
    action_response: Signal<ActionResponse>,
    is_pending: Signal<bool>,
}

pub struct EventFn(Box<dyn Fn(ev::Event) + Send + Sync + 'static>);

impl<T> From<T> for EventFn
where
    T: Fn(ev::Event) + Send + Sync + 'static,
{
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}

fn error_signal(name: &'static str) -> Signal<Option<String>> {
    let form_context = use_form_context();

    Signal::derive(move || {
        form_context
            .action_response
            .get()
            .errors
            .and_then(|errors| errors.get(name).cloned())
    })
}

fn use_form_context() -> FormContext {
    use_context().expect("Could not get form context")
}

#[component]
pub fn FormField(
    children: Children,
    #[prop(into)] error: Signal<Option<String>>,
    #[prop(into)] id: String,
    #[prop(into)] label: ViewFn,
) -> impl IntoView {
    view! {
        <fieldset class="fieldset w-full">
            <label class="fieldset-label empty:hidden" for=id>
                {label.run()}
            </label>

            {children()}

            <div class="fieldset-label text-error">{move || error.get()}</div>
        </fieldset>
    }
}

#[component]
pub fn FormProvider<ServFn, OutputProtocol>(action: ServerAction<ServFn>, children: Children) -> impl IntoView
where
    ServFn: DeserializeOwned
        + Clone
        + ServerFn<Protocol = Http<codec::PostUrl, OutputProtocol>, Output = ActionResponse, Error = ServerFnError>
        + Sync
        + 'static,
    <<ServFn::Client as client::Client<ServFn::Error>>::Request as request::ClientReq<ServFn::Error>>::FormData:
        From<web_sys::FormData>,
    <ServFn as ServerFn>::Client: client::Client<ServerFnError>,
{
    let action_value = action.value();
    let action_response = Signal::derive(move || action_value.get().and_then(|result| result.ok()).unwrap_or_default());

    provide_context(FormContext {
        action_response,
        is_pending: action.pending().into(),
    });

    view! {
        <ActionForm action=action attr:autocomplete="off" attr:novalidate="true" attr:class="form">
            {children()}
        </ActionForm>
    }
}

#[component]
pub fn PasswordField(
    #[prop(into)] id: &'static str,
    #[prop(into, optional)] label: ViewFn,
    #[prop(into)] name: &'static str,
) -> impl IntoView {
    let error = error_signal(name);
    let node_ref = NodeRef::new();
    let input_type = RwSignal::new("password".to_owned());

    let _ = use_event_listener(node_ref, ev::keydown, |event| {
        if event.key_code() == KEY_CODE_ENTER {
            event.prevent_default();
        }
    });

    let toggle_type = move |event: ev::MouseEvent| {
        event.prevent_default();

        input_type.update(|value| {
            *value = if value == "password" {
                "text".to_owned()
            } else {
                "password".to_owned()
            };
        });
    };

    view! {
        <FormField error=error id=id label=label>
            <div class="input flex items-center gap-2 pr-0 w-full" class:input-error=move || error.get().is_some()>
                <input node_ref=node_ref class="grow" id=id name=name type=input_type />
                <button class="btn btn-ghost btn-sm" type="button" on:click=toggle_type>
                    {move || {
                        if input_type.get() == "password" {
                            Either::Left(view! { <EyeSlashMini /> })
                        } else {
                            Either::Right(view! { <EyeMini /> })
                        }
                    }}
                </button>
            </div>
        </FormField>
    }
}

#[component]
pub fn SelectField(
    #[prop(into)] id: &'static str,
    #[prop(into, optional)] label: ViewFn,
    #[prop(into)] name: &'static str,
    #[prop(into, optional)] options: Signal<Vec<(String, String)>>,
    #[prop(into, optional)] value: Signal<String>,
) -> impl IntoView {
    let error = error_signal(name);

    view! {
        <FormField error=error id=id label=label>
            <select class="select w-full" class:select-error=move || error.get().is_some() id=id name=name>
                <For each=move || options.get() key=move |data| data.1.clone() let:data>
                    <option value=data.1.clone() selected=move || value.get() == data.1>
                        {data.0}
                    </option>
                </For>
            </select>
        </FormField>
    }
}

#[component]
pub fn SubmitButton(#[prop(optional)] children: Option<ChildrenFn>) -> impl IntoView {
    let form_context = use_form_context();

    let on_click = move |event: ev::MouseEvent| {
        if form_context.is_pending.get() {
            event.prevent_default();
        }
    };

    view! {
        <div class="py-2 w-full">
            <button class="btn btn-block btn-primary" on:click=on_click type="submit">
                {move || {
                    if form_context.is_pending.get() {
                        EitherOf3::A(view! { <span class="loading loading-spinner" /> })
                    } else if let Some(children) = &children {
                        EitherOf3::B(children())
                    } else {
                        EitherOf3::C(move_tr!("submit"))
                    }
                }}
            </button>
        </div>
    }
}

#[component]
pub fn TextField(
    #[prop(into)] id: &'static str,
    #[prop(default = "text", into)] input_type: &'static str,
    #[prop(into, optional)] label: ViewFn,
    #[prop(into)] name: &'static str,
    #[prop(into, optional)] on_input: Option<EventFn>,
    #[prop(into, optional)] value: RwSignal<String>,
) -> impl IntoView {
    let error = error_signal(name);
    let node_ref = NodeRef::new();

    let _ = use_event_listener(node_ref, ev::keydown, |event| {
        if event.key_code() == KEY_CODE_ENTER {
            event.prevent_default();
        }
    });

    view! {
        <FormField error=error id=id label=label>
            <input
                class="input w-full"
                class:input-error=move || error.get().is_some()
                id=id
                name=name
                node_ref=node_ref
                on:input=move |event| {
                    if let Some(on_input) = on_input.as_ref() {
                        on_input.0(event);
                    }
                }
                type=input_type
                bind:value=value
            />
        </FormField>
    }
}
