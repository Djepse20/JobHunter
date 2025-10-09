pub mod job_index;

pub(super) mod de;

pub mod jobs;
pub mod streamer;
use futures::{
    Stream, StreamExt,
    stream::{self, FuturesUnordered},
};

use crate::{
    services::database_service::{database::DataBase, types::Job},
    util::options::FetchOptions,
};

pub const JOB_TAGS: &'static [(&'static str, &'static [&'static str])] = &[
    ("C#", &["c#", "c-sharp", "c sharp", "csharp"]),
    ("Python", &["python"]),
    ("Rust", &["rust"]),
    ("Go", &["go", "goLang", "go lang"]),
    (
        "Javscript/Typescript",
        &["javascript", "js", "ts", "typescript"],
    ),
    ("Pascal", &["pascal"]),
    ("Elixir", &["elixir"]),
    ("Gleam", &["gleam"]),
    ("html/css", &["html", "css"]),
    ("C", &["c"]),
    ("C++", &["c++", "cplusplus", "c plus plus", "c plusplus"]),
    ("Java", &["java"]),
    ("Flutter/dart", &["flutter", "dart"]),
    ("Haskell", &["haskell"]),
    (
        "Sql",
        &[
            "sql",
            "mssql",
            "microsoft-sql",
            "microsoft sql",
            "sql server",
            "postgresql",
            "postgre sql",
        ],
    ),
    ("Docker", &["kubernetes", "docker"]),
    ("Frontend", &["frontend", "front end"]),
    ("Backend", &["backend", "back end"]),
    (
        "AI",
        &["machine learning", "ai", "machineLearning", "ml", "llm"],
    ),
    ("Angular", &["angularJs", "angular"]),
    ("React", &["reactjs", "react js", "react"]),
    (".Net", &[".net", "dot net", "asp.net", "dot-net"]),
    ("Spring", &["Javaspring", "spring", "spring-framework"]),
];

impl<const N: usize, J: JobFetcher> JobFetcher for [J; N] {
    fn fetch_all_jobs_with_options_and_db<'a>(
        &'a self,
        options: &'a FetchOptions,
        database: Option<&'a DataBase>,
    ) -> impl Future<Output = Option<Vec<Job>>> {
        let jobs_stream =
            FuturesUnordered::from_iter(self.iter().map(|job_fetcher| {
                // returns Future<Output = Option<Vec<Job>>>
                job_fetcher
                    .fetch_all_jobs_with_options_and_db(&options, database)
            }))
            .filter_map(|opt_vec| async move { opt_vec }) // drop None, produce Vec<Job>
            .flat_map(|vec_jobs| stream::iter(vec_jobs));
        async { Some(jobs_stream.collect().await) }
    }
}

pub trait JobFetcher {
    async fn fetch_all_jobs_with_options_and_db<'a>(
        &'a self,
        options: &'a FetchOptions,
        database: Option<&'a DataBase>,
    ) -> Option<Vec<Job>>;

    async fn fetch_all_jobs_with_options<'a>(
        &'a self,
        options: &'a FetchOptions,
    ) -> Option<Vec<Job>> {
        self.fetch_all_jobs_with_options_and_db(options, None).await
    }
    async fn fetch_all_jobs<'a>(&'a self) -> Option<Vec<Job>> {
        self.fetch_all_jobs_with_options_and_db(&FetchOptions::full(), None)
            .await
    }
}

pub trait FromQuery<From>
where
    Self: Sized,
{
    type Error;
    type Item;
    type Output<S>;
    async fn create_query(
        &self,
        val: From,
    ) -> Result<Self::Output<impl StreamExt<Item = Self::Item>>, Self::Error>;
}
