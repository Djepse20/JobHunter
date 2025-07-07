use std::collections::HashSet;
use std::ffi::IntoStringError;

use async_trait::async_trait;
use futures::{StreamExt, stream};
use reqwest::{StatusCode, Url, redirect::Policy};
use serde_json::Value;

use crate::Job_query::job_queries::database::DataBase;
use crate::Job_query::job_queries::options::{FetchOptions, QueryOptions};
use crate::Job_query::{JobQuery, JobSiteUrl, JobUrl, PortalUrl, job_queries::JobFetcher};
use scraper::{Html, Selector};

fn extract_data_click(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("[data-click]").unwrap(); // Select elements with data-click attribute
    let mut data_clicks = Vec::new();

    for element in document.select(&selector) {
        if let Some(data_click) = element.value().attr("data-click") {
            data_clicks.push(data_click.to_string());
        }
    }

    data_clicks
}

async fn get_html(request_str: &str) -> String {
    let html = reqwest::Client::new().get(request_str).send().await.unwrap().text().await.unwrap();
    html
}
use crate::Job_query::job_queries::Job;
impl JobFetcher for JobIndex {
    async fn fetch_all_jobs_with_options_and_db(
        &self,
        options: FetchOptions,
        database: Option<&DataBase>,
    ) -> Vec<Job> {
        todo!()
    }

    async fn fetch_all_jobs(&self) -> Vec<Job> {
        todo!()
    }
}
impl JobIndex {
    const PAGE_SIZE: usize = 20;
}

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq)]
struct JobIndexJobs(String);

impl JobIndex {
    pub async fn get_query_pages(fetch_options: &FetchOptions) -> Option<Vec<String>> {
        let query = JobIndex::get_query(fetch_options).await;

        let total_jobs = JobIndex::total_jobs(&query).await?;
        let jobs_to_query = fetch_options.size_options.job_num_to_query(total_jobs);
        let pages = match (
            jobs_to_query / JobIndex::PAGE_SIZE,
            jobs_to_query % JobIndex::PAGE_SIZE,
        ) {
            (pages, 1..) => pages + 1,

            (pages, _) => pages,
        };

        let base_job_search_url = "https://www.jobindex.dk/jobsoegning?".to_owned();

        Some(
            (1..=pages)
                .into_iter()
                .map(|page| match &query {
                    Some(query) => {
                        base_job_search_url.to_owned() + query + "&page=" + &page.to_string()
                    }
                    None => base_job_search_url.to_owned() + "&page=" + &page.to_string(),
                })
                .collect(),
        )
    }
    async fn total_jobs(query_str: &Option<String>) -> Option<usize> {
        let query_string = match query_str {
            Some(query) => {
                String::from("https://www.jobindex.dk/api/jobsearch/v3/jobcount") + query
            }
            None => String::from("https://www.jobindex.dk/api/jobsearch/v3/jobcount"),
        };

        let res = reqwest::get(&query_string).await.ok()?.text().await.ok()?;
        let json: Value = serde_json::from_str::<serde_json::Value>(&res).ok()?;
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

                if let Some(regions) = job_regions {
                    let region_query = JobIndex::get_region_query(regions).await;
                    string.get_or_insert_with(|| String::new()).push_str(&region_query);
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
        let regions_stream = stream::iter(regions.iter())
            .then(|region| async move { JobIndex::take_first_region(&region).await })
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
        let list = value.as_object()?.get("geoareaid")?.as_object()?.get("completions")?;
        let job = list.as_array()?.get(0)?;
        let uuid = job.as_object()?.get("id")?.as_str()?;
        Some("geoareaud=".to_owned() + uuid)
    }
}

pub struct JobIndex {}

impl JobIndex {
    async fn get_all_job_urls(&self, options: FetchOptions) -> Option<HashSet<JobIndexJobs>> {
        let queries = JobIndex::get_query_pages(&options).await?;
        let val: Vec<HashSet<JobIndexJobs>> = stream::iter(queries)
            .then(|query| async move { JobIndex::get_job_urls_from_query(&query).await })
            .collect()
            .await;

        Some(val.into_iter().flatten().collect())
    }

    async fn get_job_urls_from_query(query: &str) -> HashSet<JobIndexJobs> {
        let html = get_html(&query).await;
        let mut unique_jobs = HashSet::new();
        for link in extract_data_click(&html).iter().collect::<HashSet<_>>() {
            let curr_url =
                reqwest::Url::parse(&format!("https://www.jobindex.dk{}", link)).unwrap();
            if curr_url.query_pairs().count() != 3 {
                continue;
            }

            let res = reqwest::Client::builder()
                .redirect(Policy::custom(|attempt| attempt.stop()))
                .build()
                .unwrap()
                .get(curr_url)
                .send()
                .await
                .unwrap();

            unique_jobs.insert(JobIndexJobs(res.url().to_owned().to_string()));
        }
        unique_jobs
    }
}
