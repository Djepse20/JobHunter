pub mod parser;

use std::cmp::Reverse;
use std::collections::{BTreeSet, HashSet};
use std::ffi::IntoStringError;
use std::hash::Hash;
use std::marker::PhantomData;

use async_trait::async_trait;
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
use url::form_urlencoded::parse;

use crate::Job_query::job_queries::database::DataBase;
use crate::Job_query::job_queries::options::{FetchOptions, QueryOptions};
use crate::Job_query::{
    JobQuery, JobSiteUrl, JobUrl, PortalUrl, job_queries::JobFetcher,
};
use scraper::{Html, Selector, html};

impl JobConstants for JobIndex {
    const DATE_FORMAT: &'static str = "%Y-%m-%d";
}

use crate::Job_query::job_queries::job_constants::{
    JobConstants, JobIntermediate, JobIntermediateWithString,
};
use crate::Job_query::job_queries::{JOB_TAGS, Job};
#[async_trait]
impl JobFetcher for JobIndex {
    async fn fetch_all_jobs_with_options_and_db(
        &self,
        options: &FetchOptions,
        database: Option<&DataBase>,
    ) -> Vec<Job> {
        todo!()
    }
}

impl JobIndex {
    pub const PAGE_SIZE: usize = 20;
}

impl JobIndex {
    async fn get_html(url: &str) -> Option<String> {
        let mut res = reqwest::Client::new().get(url).send().await.ok()?;

        let len = res.content_length();

        let stream = res.bytes_stream();

        let start_seq = br#""results":["#;
        let end_seq = br#""skyscraper":{"#;
        parser::Parser::from_stream(stream, start_seq, end_seq, len).await
    }
}

async fn parse_job(url: &str) -> Option<Job> {
    let job_text: String = JobIndex::get_html(url).await?;

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

pub fn extract_json_for_page(page: &str) -> Option<impl Iterator<Item = &str>> {
    let start_seq = "var Stash = ";
    let end_seq = "}]}";

    let start_idx = page.find(start_seq)? + start_seq.len();
    let end_idx = page.find(end_seq)? + end_seq.len() - 1;
    let json_str = &page[start_idx..=end_idx];

    // Find the "results" key in the JSON string (naive)
    let results_key = "\"results\":";
    let results_pos = json_str.find(results_key)? + results_key.len();

    // Now, from results_pos, find the full array string [...] matching brackets
    find_balanced_json_array(&json_str[results_pos..]).map(|(start, end)| {
        find_outermost_array_slices(
            &json_str[results_pos + start..results_pos + end],
        )
    })
}

pub fn find_outermost_array_slices(json: &str) -> impl Iterator<Item = &str> {
    let bytes = json.as_bytes();
    let mut pos = 0;
    let mut depth = 0;
    let mut start_idx = None;

    std::iter::from_fn(move || {
        while pos < bytes.len() {
            let b = bytes[pos];
            pos += 1;

            match b {
                b'[' => {
                    depth += 1;
                    if depth == 1 {
                        start_idx = Some(pos - 1);
                    }
                }
                b']' => {
                    depth -= 1;

                    if depth == 1 {
                        let s = start_idx.take()?;
                        return Some(&json[s..pos]);
                    }
                }
                _ => {}
            }
        }
        None
    })
}

fn find_balanced_json_array(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut depth = 0;
    let mut start = None;

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'[' {
            if depth == 0 {
                start = Some(i);
            }
            depth += 1;
        } else if b == b']' {
            depth -= 1;
            if depth == 0 {
                return Some((start?, i + 1)); // end index is exclusive
            }
        }
    }
    None
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
        let regions_stream =
            stream::iter(regions.iter())
                .then(|region| async move {
                    JobIndex::take_first_region(&region).await
                })
                .filter_map(|val| async move { val })
                .peekable();
        tokio::pin!(regions_stream);
        let mut region_query = String::new();

        while let Some(region) = regions_stream.as_mut().next().await {
            match regions_stream.as_mut().peek().await {
                Some(_) => {
                    region_query.push_str(&(region + "&"));
                }
                None => {
                    region_query.push_str(&region);
                }
            }
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

impl<'de> Deserialize<'de> for JobIntermediate<'de, JobIndex> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct JobIntermediateJobIndex<'a> {
            #[serde(rename = "url")]
            job_url: &'a str,
            #[serde(with = "DateTimeSerde::<JobIndex>", rename = "firstdate")]
            pub date: DateTime<chrono::Utc>,
        }

        let intermediate = JobIntermediateJobIndex::deserialize(deserializer)?;
        Ok(JobIntermediate::<JobIndex> {
            job_url: intermediate.job_url,
            date: intermediate.date,
            _phantom: PhantomData,
        })
    }
}

impl<'a> JobIndex {
    async fn unique_jobs_for_query<'b>(
        (html, jobs_to_take, offset): (&'a str, usize, usize),

        should_ignore: &'b mut bool,
        newest_job: &'b Job,
    ) -> (
        BTreeSet<JobIntermediate<'a, JobIndex>>,
        BTreeSet<JobIntermediate<'a, JobIndex>>,
    ) {
        let mut new_unique_jobs = BTreeSet::new();
        let mut old_unique_jobs = BTreeSet::new();

        match JobIndex::get_job_urls_from_html(html).ok() {
            Some(jobs) => {
                let mut jobs_iter = jobs
                    .into_iter()
                    .map(|job| job.0.job_info)
                    .skip(offset)
                    .take(jobs_to_take);

                while let Some(job) = jobs_iter.next() {
                    if *should_ignore {
                        old_unique_jobs.insert(job);
                        old_unique_jobs.extend(jobs_iter);
                        break;
                    }
                    //IF the newest job recorded in the DB is newer than the job fetched, ignore it and the rest.
                    // We can ignore the rest, cause we go trough the list in decending order (newest, first).
                    if newest_job.created_at > job.date {
                        *should_ignore = true;
                        continue;
                    }

                    new_unique_jobs.insert(job);
                }
            }
            None => {
                //no jobs for query, move along.

                return (new_unique_jobs, old_unique_jobs);
            }
        }

        (new_unique_jobs, old_unique_jobs)
    }
    async fn get_all_unique_job<F>(
        &self,

        options: &FetchOptions,
        database: &DataBase,
    ) -> Option<()> {
        let (offset, queries) = JobIndex::get_query_pages(&options).await?;
        let mut new_unique_jobs = BTreeSet::new();
        let mut old_unique_jobs = BTreeSet::new();

        let newest_job = database.get_newest_job().await.ok()?;

        let mut should_ignore: bool = false;

        let mut queries_iter = queries.into_iter();

        let (jobs_to_take, query) = queries_iter.next()?;
        let html = Self::get_html(&query).await?;

        let (new_jobs, old_jobs) = Self::unique_jobs_for_query(
            (&html, jobs_to_take, offset),
            &mut should_ignore,
            &newest_job,
        )
        .await;

        new_unique_jobs.extend(new_jobs);
        old_unique_jobs.extend(old_jobs);

        for (jobs_to_take, query) in queries_iter {
            let (new_jobs, old_jobs) = Self::unique_jobs_for_query(
                (&html, jobs_to_take, offset),
                &mut should_ignore,
                &newest_job,
            )
            .await;
            new_unique_jobs.extend(new_jobs);

            old_unique_jobs.extend(old_jobs);
        }

        Some(())
    }

    fn get_job_urls_from_html(
        html: &'a str,
    ) -> Result<
        impl IntoIterator<Item = Reverse<JobIntermediateWithString<'a, JobIndex>>>,
        (),
    > {
        let mut jobs = extract_json_for_page(&html).ok_or(())?;

        Ok(jobs
            .into_iter()
            .filter_map(|job: &str| {
                JobIntermediateWithString::try_from(job).ok()
            })
            .map(|val| Reverse(val)))
    }
}
