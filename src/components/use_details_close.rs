#[cfg(target_arch = "wasm32")]
use std::cell::Cell;

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::use_effect;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static DETAILS_LISTENER_ATTACHED: Cell<bool> = Cell::new(false);
}

#[cfg(target_arch = "wasm32")]
const DETAILS_SELECTOR: &str = "details.filter-menu[open]";

#[allow(dead_code)]
pub fn use_details_close_on_outside_click() {
    #[cfg(target_arch = "wasm32")]
    {
        use web_sys::wasm_bindgen::{prelude::Closure, JsCast};

        use_effect(move || {
            let already_attached = DETAILS_LISTENER_ATTACHED.with(|flag| {
                if flag.get() {
                    true
                } else {
                    flag.set(true);
                    false
                }
            });
            if already_attached {
                return;
            }

            let Some(window) = web_sys::window() else {
                return;
            };
            let Some(document) = window.document() else {
                return;
            };

            let listener =
                Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |event: web_sys::MouseEvent| {
                    let Some(target) = event.target() else {
                        return;
                    };
                    let Ok(node) = target.dyn_into::<web_sys::Node>() else {
                        return;
                    };
                    let Ok(list) = document.query_selector_all(DETAILS_SELECTOR) else {
                        return;
                    };
                    for index in 0..list.length() {
                        let Some(element) = list.item(index) else {
                            continue;
                        };
                        let Ok(details) =
                            element.dyn_into::<web_sys::HtmlDetailsElement>()
                        else {
                            continue;
                        };
                        if !details.contains(Some(&node)) {
                            let _ = details.remove_attribute("open");
                        }
                    }
                });

            if window
                .add_event_listener_with_callback("mousedown", listener.as_ref().unchecked_ref())
                .is_err()
            {
                return;
            }

            listener.forget();
        });
    }
}
