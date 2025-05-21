pub mod job_queries;

use std::collections::HashSet;

use async_trait::async_trait;

use reqwest::Url;

#[derive(Debug, PartialEq, Eq, Hash)]

pub struct PortalUrl(Url);

#[derive(Debug, PartialEq, Eq, Hash)]

pub struct JobUrl(Url);
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum JobSiteUrl {
    PortalUrl(PortalUrl),
    JobUrl(JobUrl),
}

#[async_trait]
pub trait JobQuery {
    async fn job_query(&self) -> HashSet<JobSiteUrl>;
}
