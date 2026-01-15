use dioxus::prelude::*;

#[component]
pub fn NotFoundPage(segments: Vec<String>) -> Element {
    let _ = segments;
    rsx! {
        document::Title { "NotFound" }
        main { class: "container",
            h1 { "Not Found" }
            p { "The page you requested does not exist." }
            a { href: "/", "Back to Home" }
        }
    }
}
