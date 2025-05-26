use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos_fluent::tr;

#[component]
pub fn ConfirmationModal(
    children: Children,
    #[prop(into)] is_open: RwSignal<bool>,
    #[prop(into)] on_accept: Callback<((),)>,
) -> impl IntoView {
    view! {
        <Modal is_closable=false is_open=is_open>
            <div>{children()}</div>

            <div class="modal-action">
                <button
                    class="btn"
                    on:click=move |event| {
                        event.prevent_default();
                        is_open.set(false);
                    }
                >
                    {move || tr!("cancel")}
                </button>
                <button
                    class="btn btn-primary"
                    on:click=move |event| {
                        event.prevent_default();
                        is_open.set(false);
                        on_accept.run(((),));
                    }
                >
                    {move || tr!("accept")}
                </button>
            </div>
        </Modal>
    }
}

#[component]
pub fn Modal(
    #[prop(into)] is_open: RwSignal<bool>,
    children: Children,
    #[prop(into, optional)] class: &'static str,
    #[prop(into, optional)] on_close: Option<Callback<((),)>>,
    #[prop(default = true, into)] is_closable: bool,
) -> impl IntoView {
    let on_click_close = move |event: MouseEvent| {
        event.prevent_default();
        is_open.set(false);

        if let Some(oc) = on_close {
            oc.run(((),));
        }
    };

    view! {
        <dialog class=format!("modal {class}") class:modal-open=move || is_open.get()>
            <Show when=move || is_closable>
                <button class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2" on:click=on_click_close>
                    "✕"
                </button>
            </Show>

            <div class="modal-box">{children()}</div>

            <Show when=move || is_closable>
                <div class="modal-backdrop" on:click=on_click_close />
            </Show>
        </dialog>
    }
}
