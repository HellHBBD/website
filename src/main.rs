use dioxus::prelude::*;

use crate::routes::AppRouter;
use crate::task::Task;

mod components;
mod category;
mod pages;
mod routes;
mod server;
mod task;
#[cfg(test)]
mod test_support;

static CSS: Asset = asset!("assets/main.css");

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(Vec::<Task>::new()));
    use_context_provider(|| Signal::new(Option::<Task>::None));

    rsx! {
        document::Stylesheet { href: "https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css" }
        document::Stylesheet { href: CSS }
        AppRouter {}
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod http_smoke_tests {
    use super::App;
    use crate::test_support::{
        env_lock, restore_public_path, restore_storage_file, set_public_path, set_storage_file,
        temp_public_dir, temp_storage_path,
    };
    use dioxus::server::{axum, DioxusRouterExt, ServeConfig};
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn http_smoke_root_and_api() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let path = temp_storage_path();
        let previous = set_storage_file(&path);
        let public_path = temp_public_dir();
        let previous_public = set_public_path(&public_path);
        let router = axum::Router::new().serve_dioxus_application(ServeConfig::new(), App);

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/schedule")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("GET /schedule");
        let status = response.status();
        assert!(status.is_success(), "root endpoint returned {status}");
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body bytes");
        let body = String::from_utf8(bytes.to_vec()).expect("utf8 body");
        assert!(body.contains("<html"));
        assert!(body.contains("新增任務"));

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/upload")
                    .header("content-type", "application/json")
                    .body(Body::from("[]"))
                    .expect("request"),
            )
            .await
            .expect("POST /api/upload");
        let status = response.status();
        assert!(
            status != StatusCode::NOT_FOUND && status != StatusCode::METHOD_NOT_ALLOWED,
            "upload endpoint returned {status}"
        );

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/download")
                    .header("content-type", "application/json")
                    .body(Body::from("null"))
                    .expect("request"),
            )
            .await
            .expect("POST /api/download");
        assert!(response.status().is_success());

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/assets/app.js")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("GET /assets/app.js");
        assert!(response.status().is_success());
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.contains("javascript"),
            "unexpected js content-type: {content_type}"
        );

        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/assets/app.wasm")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("GET /assets/app.wasm");
        assert!(response.status().is_success());
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.contains("wasm"),
            "unexpected wasm content-type: {content_type}"
        );

        restore_public_path(previous_public);
        restore_storage_file(previous);
    }
}
