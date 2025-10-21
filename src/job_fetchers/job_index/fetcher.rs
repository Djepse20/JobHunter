use std::pin::pin;
use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;

use tokio::io::AsyncWriteExt;
use url::Url;

use crate::job_fetchers::jobs::get_all_unique_job;
use crate::job_fetchers::preview::DateFormat;
use crate::job_fetchers::preview::JobPreview;
use crate::job_fetchers::preview::UniqueJobs;
use crate::job_fetchers::preview::unique_job;
use crate::job_fetchers::{Job, JobFetcher};
use crate::services::database_service::database::DataBase;
use crate::util::from_query::CreateQuery;
use crate::util::options::FetchOptions;

use async_compression::tokio::write::GzipDecoder;

use crate::util::streamer::Streamer;
pub struct JobIndex {
    pub(super) client: reqwest::Client,
    pub(super) urls: JobIndexUrls,
}
impl DateFormat for JobIndex {
    const DATE_FORMAT: &'static str = "%Y-%M-%D";
}
pub(super) struct JobIndexUrls {
    job_search: Url,
    pub(super) job_count: Url,
    pub(super) job_regions: Url,
}
impl Default for JobIndex {
    fn default() -> Self {
        let base_url = Url::parse("https://www.jobindex.dk/").unwrap();
        JobIndex::new_with_base(base_url)
    }
}

impl JobIndex {
    pub fn new_with_base(base_url: Url) -> Self {
        JobIndex {
            client: reqwest::Client::new(),
            urls: JobIndexUrls {
                job_search: base_url.join("jobsoegning?").unwrap(),
                job_count: base_url.join("api/jobsearch/v3/jobcount").unwrap(),
                job_regions: base_url
                    .join("api/jobsearch/v3/autocomplete?&types=geoareaid")
                    .unwrap(),
            },
        }
    }
}

impl JobFetcher for JobIndex {
    async fn fetch_all_jobs_with_options_and_db<'a>(
        &'a self,
        options: &'a FetchOptions,
        database: Option<&'a DataBase>,
    ) -> Option<Vec<Job>> {
        let database = database;
        let (offset, queries) = self.create_query(options).await.ok()?;

        let stream = queries.map(async move |(jobs, url)| {
            let page = self.get_jobs(url.as_ref()).await?;

            Some((jobs, page))
        });
        let jobs = stream.buffer_unordered(8);

        let jobs = jobs.filter_map(async |data| data);

        get_all_unique_job::<JobPreview<'_, JobIndex>, _>(
            database,
            (offset, pin!(jobs)),
        )
        .await
    }
}
use serde_json::value::RawValue;

impl UniqueJobs for JobIndex {
    fn unique_jobs<'c>(
        jobs: &'c [u8],
        jobs_to_take: usize,
        offset: usize,
        newest_job: Option<&Job>,
    ) -> Option<impl StreamExt<Item = JobPreview<'c, Self>>> {
        let iter = serde_json::from_slice::<Vec<&RawValue>>(jobs)
            .map(|jobs| jobs.into_iter().map(|job| job.get().as_bytes()))
            .ok()?;
        unique_job(iter, jobs_to_take, offset, newest_job)
    }
}
impl JobIndex {
    pub const PAGE_SIZE: usize = 20;
}

impl JobIndex {
    pub async fn get_jobs(
        &self,
        query: &[(Arc<str>, Arc<str>)],
    ) -> Option<Vec<u8>> {
        let res = reqwest::Client::new()
            .get(self.urls.job_search.as_str())
            .query(query)
            .send()
            .await
            .ok()?;
        let stream = res.bytes_stream().then(|bytes| async {
            let bytes = bytes?;
            let mut decoder = GzipDecoder::new(Vec::new());
            decoder.write_all(&bytes[..]).await?;
            decoder.shutdown().await?;
            Ok::<Bytes, Box<dyn std::error::Error>>(decoder.into_inner().into())
        });

        let start_seq = br#""results":["#;
        let end_seq = br#""skyscraper":{"#;
        Streamer::get_seq_in_stream(stream, start_seq, end_seq)
            .await
            .map(|str| str.as_bytes().to_vec())
    }
}
