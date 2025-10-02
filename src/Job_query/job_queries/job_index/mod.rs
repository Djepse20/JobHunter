pub mod parser;

use std::cmp::Reverse;
use std::collections::{BTreeSet, HashSet};
use std::ffi::IntoStringError;
use std::hash::Hash;
use std::marker::PhantomData;
use std::pin::{Pin, pin};

use async_trait::async_trait;
use bytes::Bytes;
use chrono::format::Item;
use chrono::{DateTime, Utc, offset};
use futures::future::Map;
use futures::{StreamExt, stream};
use reqwest::{StatusCode, Url, redirect::Policy};
use scraper::selector::Parser;
use serde::de::Visitor;
use serde::{Deserialize, Serializer};
use serde_json::Value;
use serde_json::value::to_raw_value;
use sqlx::types::Json;
use sqlx::{database, query};
use tokio::io::AsyncWriteExt;
use url::form_urlencoded::parse;

use crate::Job_query::job_queries::database::DataBase;
use crate::Job_query::job_queries::options::{FetchOptions, QueryOptions};
use crate::Job_query::{
    JobSiteUrl, JobUrl, PortalUrl, job_queries::JobFetcher,
};
use scraper::{Html, Selector, html};

impl JobConstants for JobIndex {
    const DATE_FORMAT: &'static str = "%Y-%m-%d";
    const DATE_FORMAT_SIZE: usize = 10;
}
use async_compression::tokio::write::GzipDecoder;

use crate::Job_query::job_queries::job_constants::{
    JobConstants, JobIntermediate,
};
use crate::Job_query::job_queries::{JOB_TAGS, Job};
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

        JobIndex::get_all_unique_job(&self, database, (offset, pin!(jobs)))
            .await;

        todo!()
    }
}
use serde_json::value::RawValue;
impl JobIndex {
    pub const PAGE_SIZE: usize = 20;
}

impl JobIndex {
    async fn get_jobs(url: &str) -> Option<String> {
        let mut res = reqwest::Client::new().get(url).send().await.ok()?;

        let stream = res.bytes_stream().then(|bytes| async {
            let bytes = bytes?;
            let mut decoder = GzipDecoder::new(Vec::new());
            decoder.write_all(&bytes[..]).await?;
            decoder.shutdown().await?;
            Ok::<Bytes, Box<dyn std::error::Error>>(decoder.into_inner().into())
        });

        let start_seq = br#""results":["#;
        let end_seq = br#""skyscraper":{"#;
        parser::Parser::from_stream(stream, start_seq, end_seq).await
    }
}

async fn parse_job(url: &str) -> Option<Job> {
    let job_text: String = JobIndex::get_jobs(url).await?;

    let tags = JOB_TAGS
        .iter()
        .filter(|(_, sub_strs)| {
            sub_strs.iter().any(|sub_str| job_text.contains(sub_str))
        })
        .map(|(tag, _)| tag);

    Some(todo!())
}

async fn extract_job_text(job_url: &str) -> Option<String> {
    let html = reqwest::get(job_url).await.ok()?;

    todo!()
}

fn extract_jobs_tags(description: &str) -> Vec<String> {
    todo!()
}
fn extract_description(html: &Value) -> Option<String> {
    let start_seq = "</div>\\n\\n<p>";
    let end_seq = "</p>\\n";
    let string = html.as_str()?;

    let start_idx = string.find(start_seq)? + start_seq.len();

    let end_idx = string.find(end_seq)?;

    let string = string[start_idx..end_idx]
        .split("<p>")
        .flat_map(|x| x.split("</p>"))
        .filter(|x| !x.is_empty())
        .collect();
    Some(string)
}

impl JobIndex {
    pub async fn get_query_pages(
        fetch_options: &FetchOptions,
    ) -> Option<(usize, impl Iterator<Item = (usize, String)>)> {
        let query = JobIndex::get_query(fetch_options).await;

        let total_jobs = JobIndex::total_jobs(&query).await?;
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
    async fn total_jobs(query_str: &Option<String>) -> Option<usize> {
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

    pub async fn get_query(fetch_options: &FetchOptions) -> Option<String> {
        match &fetch_options.query_options {
            QueryOptions::Query {
                job_name,
                job_regions,
                ..
            } => {
                let mut string: Option<String> = None;

                if let regions @ [_, ..] = job_regions.as_slice() {
                    let region_query =
                        JobIndex::get_region_query(regions).await;
                    string
                        .get_or_insert_with(|| String::new())
                        .push_str(&region_query);
                }
                if let Some(job_name) = &job_name {
                    string
                        .get_or_insert_with(|| String::new())
                        .push_str(&("&q=".to_owned() + job_name));
                }
                string
            }
            QueryOptions::All => None,
        }
    }

    pub async fn get_region_query(regions: &[String]) -> String {
        let mut regions = regions.iter();
        let mut region_query = String::new();
        let mut started = false;
        while let Some(region) = regions.next() {
            let region = match JobIndex::take_first_region(&region).await {
                Some(region) => region,
                None => continue,
            };

            if started {
                region_query.push_str(&("&".to_string() + &region));
                continue;
            }
            started = true;
            region_query.push_str(&(region));
        }
        region_query
    }
    pub async fn take_first_region(job_region: &str) -> Option<String> {
        let query_string = format!(
            "https://www.jobindex.dk/api/jobsearch/v3/autocomplete?&types=geoareaid&q={}&limit=1",
            job_region
        );
        let res = reqwest::get(query_string).await.ok()?;
        let json = res.text().await.ok()?;
        let value: serde_json::Value = serde_json::from_str(&json).ok()?;
        let list = value.get("geoareaid")?.get("completions")?;
        let job = list.get(0)?;
        let uuid = job.get("id")?.as_str()?;
        Some("geoareaud=".to_owned() + uuid)
    }
}

pub struct JobIndex;
impl JobIndex {
    pub fn new() -> Self {
        JobIndex
    }
}

use crate::Job_query::job_queries::job_constants::DateTimeSerde;

impl<'a> JobIndex {
    async fn unique_jobs_for_json<'b, 'c>(
        (json, jobs_to_take, offset): (&'c str, usize, usize),

        should_ignore: &'b mut bool,
        newest_job: &'b Job,
    ) -> (
        Vec<JobIntermediate<'c, JobIndex>>,
        Vec<JobIntermediate<'c, JobIndex>>,
    ) {
        let mut new_unique_jobs = Vec::new();
        let mut old_unique_jobs = Vec::new();
        let mut jobs_iter = serde_json::Deserializer::from_str(json)
            .into_iter_seq::<JobIntermediate<JobIndex>>()
            .skip(offset)
            .take(jobs_to_take);

        while let Some(job) = jobs_iter.next() {
            let job = match job {
                Ok(job) => job,
                Err(_) => break,
            };

            if *should_ignore {
                old_unique_jobs.push(job);
                old_unique_jobs.extend(jobs_iter.flat_map(|val| val));
                break;
            }
            //IF the newest job recorded in the DB is newer than the job fetched, ignore it and the rest.
            // We can ignore the rest, cause we go trough the list in decending order (newest, first).
            if newest_job.created_at > job.date {
                *should_ignore = true;
                continue;
            }

            new_unique_jobs.push(job);
        }

        (new_unique_jobs, old_unique_jobs)
    }

    async fn get_all_unique_job(
        &self,
        database: &DataBase,
        (offset, mut job_pages): (
            usize,
            impl StreamExt<Item = (usize, String)> + Unpin,
        ),
    ) -> Option<()> {
        let newest_job = database.get_newest_job().await.ok()?;
        //TODO: mak all of this into a iterator.
        // Or a stream.
        let mut should_ignore: bool = false;
        let (jobs_to_take, html) = job_pages.next().await?;

        let mut newer_jobs = Vec::new();
        let mut older_jobs = Vec::new();

        let (mut new_jobs, mut old_jobs) = Self::unique_jobs_for_json(
            (&html, jobs_to_take, offset),
            &mut should_ignore,
            &newest_job,
        )
        .await;
        newer_jobs.append(&mut new_jobs);
        older_jobs.append(&mut old_jobs);
        while let Some((jobs_to_take, html)) = job_pages.next().await {
            let (mut new_jobs, mut old_jobs) = Self::unique_jobs_for_json(
                (todo!(), jobs_to_take, 0),
                &mut should_ignore,
                &newest_job,
            )
            .await;
            newer_jobs.append(&mut new_jobs);
            older_jobs.append(&mut old_jobs);
        }

        //database.insert_jobs(&newer_jobs).await;
        //database.delete_jobs(&older_jobs).await;

        None
    }
}
