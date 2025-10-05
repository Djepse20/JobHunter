pub mod job_fetchers;

pub mod services;
pub mod util;

use std::sync::Arc;

use axum::Router;

// async fn fetch_jobs<T>(
//     fetch_options: FetchOptions,
//     job_service: State<Arc<T>>,
// ) -> impl IntoResponse {
//     job_service.fetch_jobs(fetch_options).await;
//     ...
// }

#[tokio::main]
async fn main() {
    let jobs = Arc::new("");
    // let x = fetchers!(X : JobIndex = JobIndex::new());

    let app = Router::new()
        // .route("/fetch_jobs", get(fetch_jobs))
        .with_state(jobs);

    // run our app with hyper, listening globally on port

    let listener = tokio::net::TcpListener::bind("0.0.0.0:").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
