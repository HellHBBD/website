use dioxus::prelude::*;

#[component]
pub fn RootPage() -> Element {
    rsx! {
        document::Title { "Home" }
        main { class: "home",
            div { class: "home-bg" }
            section { class: "home-hero",
                p { class: "home-kicker", "Schedule System" }
                h1 { class: "home-title", "Plan today. See the pattern." }
                p { class: "home-subtitle",
                    "A focused workspace for scheduling tasks and reading the rhythm of your time."
                }
                nav { class: "home-actions",
                    a { class: "home-card", href: "/histogram",
                        span { class: "home-card-title", "Histogram" }
                        span { class: "home-card-text", "Review your distribution and spot imbalances fast." }
                        span { class: "home-card-cta", "View histogram" }
                    }
                }
            }
        }
    }
}
