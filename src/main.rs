pub mod Job_query;

use std::cell::Cell;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::io::BufReader;
use std::marker::{PhantomData, PhantomPinned};
use std::pin::{self, Pin, pin};
use std::rc::Rc;
use std::slice::IterMut;
use std::sync::Arc;
use std::{default, fmt};

use axum::{Router, routing::get};
use bytes::Bytes;
use futures::sink::Buffer;
use futures::{FutureExt, Stream, StreamExt, channel};
use serde::Deserialize;
use serde::de::value::SeqDeserializer;
use serde::de::{self, SeqAccess, Visitor};
use serde_json::map::Iter;
use sqlx::Value;

use crate::Job_query::job_queries::Job;
use crate::Job_query::job_queries::database::DataBase;
use crate::Job_query::job_queries::job_index::JobIndex;
use crate::Job_query::job_queries::jobs::Jobs;
use crate::Job_query::job_queries::options::FetchOptions;
use axum_extra::extract::Query;
use std::net::SocketAddr;

use Job_query::job_queries::jobs;
use axum::extract::{FromRequestParts, State};
use axum::{extract::FromRequest, http::StatusCode, response::IntoResponse};
use axum_extra::extract::QueryRejection;

use serde_json::de::SeqAccess as JsonSeqAccess;

use serde_json::de::Read;

use serde_json::Deserializer;
#[axum::debug_handler]
async fn fetch_jobs(
    fetch_options: FetchOptions,
    job_service: State<Arc<Jobs>>,
) -> impl IntoResponse {
    job_service.fetch_jobs(fetch_options).await;

    // ...
}

// Define a non-Copy struct
// struct NonCopyData {
//     value: String,
// }

// // Implement FnOnceS for NonCopyData
// impl FnOnceS for NonCopyData {
//     fn call_once(self) {
//         println!("Called with: {}", self.value);
//     }
// }

use tokio::sync::mpsc::{
    self, Sender, UnboundedReceiver, UnboundedSender, unbounded_channel,
};

use serde_json::value::RawValue;

struct PinnedStr {
    str: String,
    pin: PhantomPinned,
}

#[tokio::main]
async fn main() {
    let str = String::from(
        r#"[{"abc":1},{"😤👿😳😀😡😀💩🥰😋😳🤣":"hah"},{"gg":"c"},{"skyscrape": {"abc":"haha"}}]"#,
    );

    let de = Deserializer::from_str(&str).into_iter_seq::<&[u8]>();

    for item in de {
        println!("{:?}", item);
    }
    // let x: IterMut<'_, i32> = Vec::new().iter_mut();
    // let my_iter = MyIterator::new::<&RawValue>(&mut de);
    // for ele in my_iter {
    //     println!("{}", ele);
    // }

    let jobs = Arc::new(
        Jobs::new()
            .add_fetcher(JobIndex::new())
            .add_database(DataBase::new()),
    );
    let app = Router::new()
        .route("/fetch_jobs", get(fetch_jobs))
        .with_state(jobs);

    // run our app with hyper, listening globally on port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
