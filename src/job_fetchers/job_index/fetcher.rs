use std::pin::pin;
use std::sync::Arc;

use bytes::Bytes;
use futures::{StreamExt, stream};
use serde_json::Value;

use sqlx::query;
use tokio::io::AsyncWriteExt;
use url::Url;

use crate::job_fetchers::de::preview::DateFormat;
use crate::job_fetchers::job_index::preview::JobPreview;
use crate::job_fetchers::jobs::get_all_unique_job;
use crate::job_fetchers::{Job, JobFetcher, streamer};
use crate::services::database_service::database::DataBase;
use crate::util::from_query::FromQuery;
use crate::util::options::FetchOptions;

use async_compression::tokio::write::GzipDecoder;
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
        //https://www.jobindex.dk/
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
        let database = database?;
        let (offset, queries) = self.create_query(options).await.ok()?;

        let stream = queries.map(async move |(jobs, url)| {
            let page = self.get_jobs(url.as_ref()).await?;

            Some((jobs, page))
        });
        let jobs = stream.buffer_unordered(8);

        let jobs = jobs.filter_map(async |data| match data {
            Some((val, page)) => Some((val, page)),
            None => None,
        });

        Some(
            get_all_unique_job::<JobPreview<'_, JobIndex>, _>(
                database,
                (offset, pin!(jobs)),
            )
            .await?,
        )
    }
}
impl JobIndex {
    pub const PAGE_SIZE: usize = 20;
}

impl JobIndex {
    pub async fn get_jobs(
        &self,
        query: &[(Arc<str>, Arc<str>)],
    ) -> Option<String> {
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
        streamer::Streamer::get_seq_in_stream(stream, start_seq, end_seq).await
    }
}
