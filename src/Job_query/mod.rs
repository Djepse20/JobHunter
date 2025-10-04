pub mod job_queries;

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
