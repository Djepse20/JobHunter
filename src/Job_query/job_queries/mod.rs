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

pub trait JobFetcher {
    async fn fetch_all_jobs_with_options_and_db(
        &self,
        options: FetchOptions,
        database: Option<&DataBase>,
    ) -> Vec<Job>;

    async fn fetch_all_jobs_with_options(&self, options: FetchOptions) -> Vec<Job> {
        self.fetch_all_jobs_with_options_and_db(FetchOptions::full(), None).await
    }
    async fn fetch_all_jobs(&self) -> Vec<Job> {
        self.fetch_all_jobs_with_options_and_db(FetchOptions::full(), None).await
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

pub struct Job {
    job_info: JobInfo,
    job_urls: JobUrls,
    job_tags: JobTags,
    contact_info: ContactInfo,
}
pub struct JobTags(Vec<String>);

pub struct JobInfo {
    title: String,
    company: String,
    email_address: String,
    description: String,
    job_site_tag: String,
}

pub struct JobUrls {
    job_site_url: String,
    job_url: String,
}

pub struct ApplicationHandler {
    job_net_handler: JobNetHandler,
    docs_service_handler: (),
}
struct JobNetHandler;

pub struct ContactInfo {
    name: String,
    phone_number: PhoneNumber,
    email: String,
}

pub struct PhoneNumber(String);
