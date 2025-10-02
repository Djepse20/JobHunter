pub mod database;
pub mod job_constants;
pub mod job_fetcher;
pub mod job_index;
pub mod jobs;
pub mod options;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use database::DataBase;

use crate::Job_query::job_queries::options::FetchOptions;

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

impl<T1> JobFetcher for (T1, ())
where
    T1: JobFetcher,
{
    async fn fetch_all_jobs_with_options_and_db<'a>(
        &'a self,
        options: &'a FetchOptions,
        database: Option<&'a DataBase>,
    ) -> Option<Vec<Job>> {
        self.0
            .fetch_all_jobs_with_options_and_db(options, database)
            .await
    }
}

impl<T1, T2> JobFetcher for (T1, T2)
where
    T1: JobFetcher,
    T2: JobFetcher,
{
    async fn fetch_all_jobs_with_options_and_db<'a>(
        &'a self,
        options: &'a FetchOptions,
        database: Option<&'a DataBase>,
    ) -> Option<Vec<Job>> {
        let mut jobs = Vec::new();
        match self
            .0
            .fetch_all_jobs_with_options_and_db(options, database)
            .await
        {
            Some(first_jobs) => jobs.extend(first_jobs.into_iter()),
            None => {}
        };

        match self
            .1
            .fetch_all_jobs_with_options_and_db(options, database)
            .await
        {
            Some(second_jobs) => jobs.extend(second_jobs.into_iter()),
            None => {}
        };
        Some(jobs)
    }
}

pub struct JobApplications {
    applications: Vec<JobApplication>,
}

pub struct JobApplication {
    job: Job,
}

pub struct JobId(u64);
pub struct Job {
    job_info: JobInfo,
    created_at: DateTime<Utc>,

    company_info: CompanyInfo,
    job_tags: Vec<JobTag>,
    contact_info: ContactInfo,
}

pub struct JobTag {
    tag_id: i64,
    name: String,
}

pub struct JobInfo {
    job_id: JobId,
    job_url: String,

    title: String,
    description: String,
}

pub struct CompanyInfo {
    company_id: i64,
    name: String,
    email_address: String,
    address: String,
}

pub struct ContactInfo {
    contact_id: i64,
    name: String,
    phone_number: PhoneNumber,
    email: String,
}

pub struct PhoneNumber(String);

pub struct ApplicationHandler {
    job_net_handler: JobNetHandler,
    docs_service_handler: (),
}
struct JobNetHandler;
