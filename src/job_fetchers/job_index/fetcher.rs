use std::pin::pin;

use bytes::Bytes;
use futures::StreamExt;
use serde_json::Value;

use tokio::io::AsyncWriteExt;

use crate::job_fetchers::de::jobs::get_all_unique_job;
use crate::job_fetchers::job_index::preview::JobPreview;
use crate::job_fetchers::{
    JOB_TAGS, Job, JobFetcher, de::preview::DeserializableJob, streamer,
};
use crate::util::options::FetchOptions;
use crate::{
    job_fetchers::FromQuery, services::database_service::database::DataBase,
};

use async_compression::tokio::write::GzipDecoder;
pub struct JobIndex;
impl JobIndex {
    pub fn new() -> Self {
        JobIndex
    }
}

impl JobFetcher for JobIndex {
    async fn fetch_all_jobs_with_options_and_db<'a>(
        &'a self,
        options: &'a FetchOptions,
        database: Option<&'a DataBase>,
    ) -> Option<Vec<Job>> {
        let database = database?;
        let (offset, queries) = JobIndex::get_query_pages(options).await?;

        let stream = queries.map(async move |(jobs, url)| {
            let page = JobIndex::get_jobs(&url).await?;

            Some((jobs, page))
        });
        let jobs = futures::stream::FuturesUnordered::from_iter(stream);

        let jobs = jobs.filter_map(async |data| match data {
            Some((val, page)) => Some((val, page)),
            None => None,
        });

        get_all_unique_job::<_, JobPreview<'_, JobIndex>>(
            database,
            (offset, pin!(jobs)),
        )
        .await;

        todo!()
    }
}
impl JobIndex {
    pub const PAGE_SIZE: usize = 20;
}

impl JobIndex {
    pub async fn get_jobs(url: &str) -> Option<String> {
        let res = reqwest::Client::new().get(url).send().await.ok()?;

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

impl JobIndex {
    pub async fn get_query_pages(
        fetch_options: &FetchOptions,
    ) -> Option<(usize, impl Iterator<Item = (usize, String)>)> {
        let query = JobIndex::create_query(fetch_options)
            .await
            .ok()
            .map(|x| x.as_ref().to_owned());

        let total_jobs = JobIndex::total_jobs(query.as_ref()).await?;
        let (offset, _, pages) = fetch_options.size_options.job_num_to_query(
            total_jobs,
            JobIndex::PAGE_SIZE,
            1,
        );

        let base_job_search_url =
            "https://www.jobindex.dk/jobsoegning?".to_owned();

        let sorted_query = query.unwrap_or_default() + "&sort=date";

        Some((
            offset,
            pages.into_iter().map(move |(jobs, page)| {
                (
                    jobs,
                    base_job_search_url.to_owned()
                        + &sorted_query
                        + "&page="
                        + &page.to_string(),
                )
            }),
        ))
    }
    async fn total_jobs(query_str: Option<&String>) -> Option<usize> {
        let query_string = match query_str {
            Some(query) => {
                String::from(
                    "https://www.jobindex.dk/api/jobsearch/v3/jobcount",
                ) + query
            }
            None => String::from(
                "https://www.jobindex.dk/api/jobsearch/v3/jobcount",
            ),
        };

        let res = reqwest::get(&query_string).await.ok()?.text().await.ok()?;
        let json: Value =
            serde_json::from_str::<serde_json::Value>(&res).ok()?;
        Some(json.as_object()?.get("hitcount")?.as_u64()? as usize)
    }
}
