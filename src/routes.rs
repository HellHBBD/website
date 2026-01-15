use dioxus::prelude::*;
use dioxus_router::{Routable, Router};

use crate::pages::histogram::HistogramPage;
use crate::pages::not_found::NotFoundPage;
use crate::pages::root::RootPage;
use crate::pages::schedule::SchedulePage;

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[route("/")]
    Root {},
    #[route("/schedule")]
    Schedule {},
    #[route("/histogram")]
    Histogram {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

#[component]
pub fn AppRouter() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Root() -> Element {
    rsx! {
        RootPage {}
    }
}

#[component]
fn Schedule() -> Element {
    rsx! {
        SchedulePage {}
    }
}

#[component]
fn Histogram() -> Element {
    rsx! {
        HistogramPage {}
    }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        NotFoundPage { segments }
    }
}
