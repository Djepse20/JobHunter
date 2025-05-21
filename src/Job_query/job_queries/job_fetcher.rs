use std::{
    collections::HashSet,
    sync::{
        Arc, Mutex, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use crate::Job_query::{JobQuery, JobSiteUrl, JobUrl, PortalUrl};
use async_trait::async_trait;
use serde_json::map::Iter;
use sqlx::{PgConnection, database, postgres::PgPool};
use tokio::task::JoinSet;
use tokio_stream::{self as stream, StreamExt};
use url::Url;

#[derive(Debug)]
struct JobResult {
    job_info: JobResultInfo,
    job_name: String,
    job_description: String,
    job_tags: Vec<JobTag>,
}
#[derive(Debug)]
struct JobResultInfo {
    job_id: usize,
    portal_url: Option<PortalUrl>,
    job_url: JobUrl,
}

impl JobResult {
    async fn create_from_job(url: &JobSiteUrl) -> JobResult {
        match url {
            JobSiteUrl::JobUrl(url) => {
                todo!()
            }
            JobSiteUrl::PortalUrl(url) => {
                todo!()
            }
        }
    }
}

impl PartialEq<JobResultInfo> for JobResultInfo {
    fn eq(&self, other: &JobResultInfo) -> bool {
        self.job_id == other.job_id
    }
}

impl PartialEq<JobSiteUrl> for JobResultInfo {
    fn eq(&self, job_site_url: &JobSiteUrl) -> bool {
        match job_site_url {
            JobSiteUrl::PortalUrl(url) => match &self.portal_url {
                None => false,
                Some(portal_url) => portal_url == url,
            },

            JobSiteUrl::JobUrl(url) => &self.job_url == url,
        }
    }
}

impl PartialEq<JobResult> for JobResult {
    fn eq(&self, other: &JobResult) -> bool {
        self.job_info.job_id == other.job_info.job_id
    }
}

impl PartialEq<JobSiteUrl> for JobResult {
    fn eq(&self, job_site_url: &JobSiteUrl) -> bool {
        self.job_info == *job_site_url
    }
}

impl Eq for JobResult {}
#[derive(Debug)]

struct JobTag {
    tag_id: usize,
    tag: String,
}

struct JobFetcher {
    database_conn: PgConnection,
    job_sites: Vec<Arc<dyn JobQuery + Sync + Send>>,
    paginator: PaginatedJobResults,
}

impl JobFetcher {
    pub fn new(
        database_conn: PgConnection,
        job_sites: Vec<Arc<dyn JobQuery + Sync + Send>>,
    ) -> Self {
        JobFetcher {
            database_conn: database_conn,
            job_sites: job_sites,

            paginator: PaginatedJobResults::new(),
        }
    }
}

struct PaginatedJobResults {
    jobs: Vec<JobResult>,
    total_jobs: AtomicUsize,
    curr_page_number: AtomicUsize,
    page_size: AtomicUsize,
}

impl PaginatedJobResults {
    fn new() -> Self {
        PaginatedJobResults {
            jobs: vec![],
            total_jobs: AtomicUsize::new(0),
            curr_page_number: AtomicUsize::new(0),
            page_size: AtomicUsize::new(20),
        }
    }

    fn set_page_size(&self, size: usize) {
        self.page_size.store(size, Ordering::Relaxed);
    }

    fn increment_curr_page(&self) {
        self.page_size.fetch_add(1, Ordering::Relaxed);
    }
    fn decrement_curr_page(&self) {
        self.page_size.fetch_sub(1, Ordering::Relaxed);
    }
    fn get_curr_page(&self) -> usize {
        self.curr_page_number.load(Ordering::Relaxed)
    }
    fn get_page_size(&self) -> usize {
        self.page_size.load(Ordering::Relaxed)
    }
}

impl JobFetcher {
    async fn query_all_sites_for_job(
        &self,
    ) -> Result<(), PoisonError<RwLockReadGuard<'_, Vec<JobResult>>>> {
        let job_info: Vec<JobResultInfo> = self.query_database_job_info().await;
        let job_urls_from_jobsites = self.query_jobsites_url().await;
        let new_jobs = job_urls_from_jobsites
            .iter()
            .filter(|job_url| !job_info.iter().any(|job| job == *job_url));

        Ok(())
    }

    async fn query_jobsites_url(&self) -> HashSet<JobSiteUrl> {
        let mut job_tasks: JoinSet<HashSet<JobSiteUrl>> = tokio::task::JoinSet::new();
        for job_site in &self.job_sites {
            let job_site = job_site.clone();
            job_tasks.spawn(async move {
                let jobs = job_site.job_query().await;
                jobs
            });
        }
        job_tasks.join_all().await.into_iter().flatten().collect()
    }
    async fn query_database_job_info(&self) -> Vec<JobResultInfo> {
        todo!()
    }

    async fn get_page<'a>(&'a self) -> Option<Page> {
        let page = self.paginator.get_curr_page();
        let page_size = self.paginator.get_page_size();
        let start = page * page_size;
        let end = std::cmp::min(
            self.paginator.total_jobs.load(Ordering::Relaxed),
            start + page_size,
        );
        let jobs = self.get_page_from_db(start, end).await;
        Some(Page {
            jobs: jobs,
            start: start,
            curr: start,
            end: end,
        })
    }

    async fn get_page_from_db(&self, start: usize, end: usize) -> Vec<JobResult> {
        todo!()
    }
}

impl JobFetcher {
    fn add_jobs_to_database(&self, jobs: &[JobResult]) {
        todo!()
    }
}

struct Page {
    jobs: Vec<JobResult>,
    start: usize,
    curr: usize,
    end: usize,
}

impl std::ops::Index<usize> for Page {
    type Output = JobResult;
    fn index(&self, idx: usize) -> &JobResult {
        &self.jobs[idx]
    }
}

impl Page {
    pub fn iter(&self) -> impl Iterator<Item = &JobResult> {
        self.jobs[self.start..self.end].iter()
    }
}
impl JobFetcher {
    fn add_jobs(
        &self,
        jobs: Vec<JobResult>,
    ) -> Result<usize, PoisonError<RwLockWriteGuard<'_, Vec<JobResult>>>> {
        let jobs_len = jobs.len();
        let prev_len = self.paginator.total_jobs.fetch_add(jobs_len, Ordering::Relaxed);
        self.add_jobs_to_database(&jobs);
        Ok(prev_len + jobs_len)
    }
}

impl JobFetcher {}
