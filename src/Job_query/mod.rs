pub mod job_queries;

pub mod equality;

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
