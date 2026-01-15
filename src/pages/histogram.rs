use dioxus::prelude::*;

use crate::components::histogram_page::HistogramPage as HistogramView;

#[component]
pub fn HistogramPage() -> Element {
    rsx! {
        document::Title { "histogram" }
        HistogramView {}
    }
}
