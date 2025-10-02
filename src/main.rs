pub mod Job_query;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::marker::PhantomPinned;
use std::ops::{Add, Mul};
use std::sync::Arc;

use axum::{Router, routing::get};
use rkyv::Archive;

use crate::Job_query::job_queries::JobFetcher;
use crate::Job_query::job_queries::database::DataBase;
use crate::Job_query::job_queries::job_index::JobIndex;
use crate::Job_query::job_queries::jobs::{AddFetcher, Fetch, Jobs};
use crate::Job_query::job_queries::options::FetchOptions;

use axum::extract::State;
use axum::response::IntoResponse;

async fn fetch_jobs<T: Fetch>(
    fetch_options: FetchOptions,
    job_service: State<Arc<T>>,
) -> impl IntoResponse {
    job_service.fetch_jobs(fetch_options).await;
    // ...
}
struct X {}
macro_rules! split_call {
    ($path:path ) => {{ [stringify!($path)] }};
}
use crate::Job_query::equality::RecEqChecker;
use crate::Job_query::equality::TupleLength;
#[tokio::main]
async fn main() {
    let b = split_call!(X::new_with());
    println!("{:?}", b); // ["X", "::", "new_with(x)"]

    let jobs = Arc::new(Jobs::new().add_database(DataBase::new()));
    let app = Router::new()
        .route("/fetch_jobs", get(fetch_jobs))
        .with_state(jobs);

    // run our app with hyper, listening globally on port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
