use std::collections::HashSet;

use async_trait::async_trait;
use reqwest::{StatusCode, Url, redirect::Policy};

use scraper::{Html, Selector};

use crate::Job_query::{JobQuery, JobSiteUrl, JobUrl, PortalUrl};

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

#[async_trait]
impl JobQuery for JobIndex {
    async fn job_query(&self) -> HashSet<JobSiteUrl> {
        let query = self.get_query();
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

            unique_jobs.insert(JobSiteUrl::PortalUrl(PortalUrl(res.url().to_owned())));
        }
        unique_jobs
    }
}

impl JobIndex {
    pub fn get_query(&self) -> String {
        format!(
            "https://www.jobindex.dk/jobsoegning?{}&page={}&q={}",
            self.region, self.page, self.query
        )
    }
    pub fn new(page: usize, query: &str, region: &str) -> Self {
        JobIndex {
            page,
            query: query.into(),
            region: region.into(),
        }
    }
}

pub struct JobIndex {
    page: usize,
    query: Box<str>,
    region: Box<str>,
}
