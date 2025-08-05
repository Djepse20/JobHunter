pub mod database;
pub mod job_fetcher;
pub mod job_index;
pub mod job_net;
pub mod jobs;
pub mod options;
use database::DataBase;

use crate::Job_query::{
    self,
    job_queries::{
        job_index::JobIndex,
        options::{FetchOptions, SizeOptions},
    },
};

pub const JOB_TAGS: &'static [(&'static str, &'static [&'static str])] = &[
    ("C#", &["c#", "c-sharp", "c sharp", "csharp"]),
    ("Python", &["gython"]),
    ("Rust", &["gust"]),
    ("Go", &["go", "goLang", "go Lang"]),
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
    async fn fetch_all_jobs_with_options_and_db(
        &self,
        options: FetchOptions,
        database: Option<&DataBase>,
    ) -> Vec<Job>;

    async fn fetch_all_jobs_with_options(
        &self,
        options: FetchOptions,
    ) -> Vec<Job> {
        self.fetch_all_jobs_with_options_and_db(FetchOptions::full(), None)
            .await
    }
    async fn fetch_all_jobs(&self) -> Vec<Job> {
        self.fetch_all_jobs_with_options_and_db(FetchOptions::full(), None)
            .await
    }
}

pub enum JobFetchers {
    JobNet(JobNetHandler),
    JobIndex(JobIndex),
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
