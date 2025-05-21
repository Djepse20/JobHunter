use std::collections::HashSet;
use std::time::Duration;

use async_trait::async_trait;
use reqwest::{StatusCode, Url, get, header::LOCATION, redirect::Policy};
use scraper::{Html, Selector};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use url::ParseError;

use crate::Job_query::{JobQuery, JobSiteUrl, JobUrl, PortalUrl};

impl JobNet {
    pub fn get_query(&self) -> String {
        let offset: usize = (self.page - 1) * 20;
        let count: usize = 20;

        return format!(
            "{{\"model\":{{\"Offset\":\"{}\",\"Count\":{},\"SearchString\":\"{}\",\"SortValue\":\"BestMatch\",\"Ids\":[],\"EarliestPublicationDate\":null,\"HotJob\":null,\"Abroad\":null,\"NearBy\":\"\",\"OnlyGeoPoints\":false,\"WorkPlaceNotStatic\":null,\"WorkHourMin\":null,\"WorkHourMax\":null,\"Facets\":{{\"Region\":[{{\"Id\":0,\"Value\":\"{}\",\"Count\":0}}],\"Country\":null,\"Municipality\":null,\"PostalCode\":null,\"OccupationAreas\":null,\"OccupationGroups\":null,\"Occupations\":null,\"EmploymentType\":null,\"WorkHours\":null,\"WorkHourPartTime\":null,\"JobAnnouncementType\":null,\"WorkPlaceNotStatic\":null}},\"LocatedIn\":null,\"LocationZip\":null,\"Location\":null,\"SearchInGeoDistance\":0,\"SimilarOccupations\":null,\"SearchWithSimilarOccupations\":false}},\"url\":\"/CV/FindWork?SearchString={}&Offset={}&Region={}&SortValue=BestMatch\"}}",
            offset, count, self.query, self.region, self.query, offset, self.region
        );
    }
}

#[async_trait]
impl JobQuery for JobNet {
    async fn job_query(&self) -> HashSet<JobSiteUrl> {
        let client =
            reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().unwrap();
        let job_data = client
            .post("https://job.jobnet.dk/CV/FindWork/Search")
            .body(self.get_query())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let job_data = serde_json::to_value(job_data).unwrap();

        JobNet::get_urls(job_data)
    }
}

impl JobNet {
    fn get_urls(job_data: Value) -> HashSet<JobSiteUrl> {
        if let Some(Value::Array(jobs)) =
            job_data.as_object().and_then(|object| object.get("JobPositionPostings"))
        {
            return jobs
                .iter()
                .filter_map(|job| {
                    job.as_object().and_then(|job| job.get("Url").and_then(|url| url.as_str()))
                })
                .filter_map(|job| Url::parse(job).ok())
                .map(|job_url| JobSiteUrl::JobUrl(JobUrl(job_url)))
                .collect::<HashSet<JobSiteUrl>>();
        }
        HashSet::new()
    }
}

pub struct JobNet {
    page: usize,
    query: Box<str>,
    region: Box<str>,
}
