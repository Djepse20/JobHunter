use crate::{
    job_fetchers::{FromQuery, job_index::fetcher::JobIndex},
    util::options::{FetchOptions, QueryOptions},
};

pub struct JobIndexQuery(String);

impl AsRef<str> for JobIndexQuery {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromQuery<&FetchOptions> for JobIndex {
    type Error = ();
    type Output = JobIndexQuery;
    async fn create_query(
        fetch_options: &FetchOptions,
    ) -> Result<Self::Output, Self::Error> {
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
                string.map(|str| JobIndexQuery(str)).ok_or(())
            }
            QueryOptions::All => Err(()),
        }
    }
}
impl JobIndex {
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
#[cfg(test)]
mod tests {
    #[tokio::test]

    async fn first_page() {}
}
