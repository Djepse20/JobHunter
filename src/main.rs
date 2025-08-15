pub mod Job_query;

use std::cell::Cell;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::io::BufReader;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::{default, fmt};

use axum::{Router, routing::get};
use futures::channel;
use futures::io::Read;
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::map::Iter;

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
use serde::Serialize;
use serde_json::value::RawValue;
use serde_json::{Value, json};

#[axum::debug_handler]
async fn fetch_jobs(
    fetch_options: FetchOptions,
    job_service: State<Arc<Jobs>>,
) -> impl IntoResponse {
    job_service
        .fetch_jobs(fetch_options)
        .await;

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

use tokio::sync::mpsc::{self, Sender, UnboundedReceiver, UnboundedSender};

#[derive(Debug)]
pub struct MyData;

impl MyData {
    pub async fn new<F: Fn(&RawValue) + 'static>(val: String, f: F) -> Self {
        unsafe impl<T: ?Sized> Send for TrustMeBro<T> {}
        unsafe impl<T: ?Sized> Sync for TrustMeBro<T> {}
        struct TrustMeBro<T: ?Sized>(*const T);

        impl<'de, T: 'de + ?Sized> Deserialize<'de> for TrustMeBro<T>
        where
            &'de T: Deserialize<'de>,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw_val = <&T>::deserialize(deserializer)?;
                Ok(TrustMeBro(raw_val as *const T))
            }
        }

        impl<'de, T: ?Sized> AsRef<T> for TrustMeBro<T> {
            fn as_ref(&self) -> &T {
                unsafe { &*self.0 }
            }
        }
        struct SenderVisitor<'de> {
            tx: UnboundedSender<TrustMeBro<RawValue>>,
            _life_time: PhantomData<&'de ()>,
        }
        let pinned: Pin<Box<str>> = val.into_boxed_str().into();

        impl<'de> Visitor<'de> for SenderVisitor<'de> {
            type Value = ();

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a JSON array")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                // Specify the element type so the compiler can infer it.
                while let Ok(Some(val)) = seq.next_element::<TrustMeBro<RawValue>>() {
                    // Ignore send errors (e.g., receiver dropped).
                    let _ = self.tx.send(val);
                }
                Ok(())
            }
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<TrustMeBro<RawValue>>();
        let ptr = TrustMeBro(pinned.as_ref().get_ref() as *const str);

        // ⚠️ Still unsafe: you're extending `val` across a 'static task via a raw pointer.
        let handle = unsafe {
            tokio::spawn(async move {
                let ptr = ptr;

                // Recreate &str from raw pointer.
                let s: &str = &*ptr.0;

                // Borrowing deserializer over `s`.
                let mut de = serde_json::Deserializer::from_str(s);

                let visitor = SenderVisitor {
                    tx,
                    _life_time: PhantomData,
                };

                // Drive the visitor. We ignore the result to keep it simple.
                let _ = de.deserialize_seq(visitor);
            })
        };
        while let Some(data) = rx.recv().await {
            unsafe { f(&*data.0) }
        }
        handle.await;
        MyData
    }
}

#[tokio::main]
async fn main() {
    // let parser: ParserHolder<SpecificParser> = serde_json::from_str(&data).unwrap();

    let val = String::from(r#"[{"b":2},{"b":4}]"#);
    let mut test: &str = &"abc";
    let mut my_data = MyData::new(val, |raw_value| {
        println!("{}", raw_value.get());
    })
    .await;

    // let service = Arc::new(J);
    let jobs = Arc::new(
        Jobs::new()
            .add_fetcher(JobIndex::new())
            .add_database(DataBase::new()),
    );
    let app = Router::new()
        .route("/fetch_jobs", get(fetch_jobs))
        .with_state(jobs);

    // run our app with hyper, listening globally on port
    let listener = tokio::net::TcpListener::bind("0.0.0.0:")
        .await
        .unwrap();
    axum::serve(listener, app)
        .await
        .unwrap();
}
