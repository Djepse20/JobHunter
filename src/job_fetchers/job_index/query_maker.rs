use futures::{StreamExt, stream};
use serde_json::Value;

use crate::{
    job_fetchers::{FromQuery, job_index::fetcher::JobIndex, streamer},
    util::options::{FetchOptions, QueryOptions, SizeOptions},
};

pub struct JobIndexQuery(String);

impl AsRef<str> for JobIndexQuery {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl FromQuery<&QueryOptions> for JobIndex {
    type Error = ();
    type Item = (Arc<str>, Arc<str>);
    type Output<S> = S;

    async fn create_query(
        &self,
        query_options: &QueryOptions,
    ) -> Result<Self::Output<impl StreamExt<Item = Self::Item>>, Self::Error>
    {
        match query_options {
            QueryOptions::Query {
                job_name,
                job_regions,
                ..
            } => Ok(self.get_region_query(&job_regions).await.chain(
                stream::once(async {
                    job_name.as_ref().map(|job_name| {
                        ("q".into(), job_name.to_owned().into())
                    })
                })
                .filter_map(async |x| x),
            )),
            QueryOptions::All => Err(()),
        }
    }
}

impl JobIndex {
    pub async fn get_region_query(
        &self,
        regions: &[String],
    ) -> impl StreamExt<Item = (Arc<str>, Arc<str>)> {
        let regions = regions.iter();

        futures::stream::iter(regions)
            .filter_map(async |region| self.take_first_region(&region).await)
    }
    pub async fn take_first_region(
        &self,
        job_region: &str,
    ) -> Option<(Arc<str>, Arc<str>)> {
        let mut url = self.urls.job_regions.clone();
        url.query_pairs_mut()
            .append_pair("q", job_region)
            .append_pair("limit", "1")
            .finish();
        let res = reqwest::get(url.as_str()).await.ok()?;
        let json = res
            .bytes_stream()
            .then(async |bytes| bytes.map_err(Box::new));
        let uuid =
            streamer::Streamer::get_seq_in_stream(json, b"id\":", b",").await?;

        let uuid: u64 = uuid.parse().ok()?;
        Some(("geoareaid".into(), uuid.to_string().into()))
    }
}
use std::sync::Arc;

impl FromQuery<&FetchOptions> for JobIndex {
    type Error = ();
    type Item = (usize, Arc<[(Arc<str>, Arc<str>)]>);
    type Output<S> = (usize, S);

    async fn create_query(
        &self,
        fetch_options: &FetchOptions,
    ) -> Result<Self::Output<impl StreamExt<Item = Self::Item>>, Self::Error>
    {
        let region_query =
            self.create_query(&fetch_options.query_options).await?;

        let query: Arc<[(Arc<str>, Arc<str>)]> = region_query
            .collect::<Vec<(Arc<str>, Arc<str>)>>()
            .await
            .into();
        let (offset, pages_query) = {
            let query = query.clone();
            self.create_query((query, &fetch_options.size_options))
                .await?
        };

        let query_with_jobs = pages_query.map(move |(jobs, page)| {
            (
                jobs,
                ([
                    [("p".to_owned().into(), page.to_string().into())].into(),
                    query.clone(),
                ]
                .concat())
                .into(),
            )
        });

        Ok((offset, query_with_jobs))
    }
}

impl FromQuery<(Arc<[(Arc<str>, Arc<str>)]>, &SizeOptions)> for JobIndex {
    type Error = ();

    type Item = (usize, usize);

    type Output<S> = (usize, S);

    async fn create_query(
        &self,
        (query, size_options): (Arc<[(Arc<str>, Arc<str>)]>, &SizeOptions),
    ) -> Result<Self::Output<impl StreamExt<Item = Self::Item>>, Self::Error>
    {
        let total_jobs = self.total_jobs(&query).await.ok_or(())?;
        let (offset, _, pages) =
            size_options.job_num_to_query(total_jobs, JobIndex::PAGE_SIZE, 1);
        Ok((offset, stream::iter(pages)))
    }
}
impl JobIndex {
    async fn total_jobs(
        &self,
        query: &[(Arc<str>, Arc<str>)],
    ) -> Option<usize> {
        let res = self
            .client
            .get(self.urls.job_count.as_str())
            .query(&query)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;
        let json: Value =
            serde_json::from_str::<serde_json::Value>(&res).ok()?;
        Some(json.as_object()?.get("hitcount")?.as_u64()? as usize)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::sync::{LazyLock, Mutex};
    use tokio_stream::StreamExt;
    use url::Url;

    use crate::{
        job_fetchers::{FromQuery, job_index::fetcher::JobIndex},
        util::options::{FetchOptions, QueryOptions, SizeOptions},
    };

    static MOCK_URL_SERVER: LazyLock<Mutex<(Url, mockito::ServerGuard)>> =
        LazyLock::new(|| Mutex::new(create_mock_server()));
    fn create_mock_server() -> (Url, mockito::ServerGuard) {
        let mut server = mockito::Server::new();

        // Use one of these addresses to configure your client
        let url =
            Url::parse(&server.url()).expect("should be a valid mock url");
        // Create a mock

        (url, server)
    }

    #[test]

    fn first_page() {
        let (url, server) =
            &mut *MOCK_URL_SERVER.lock().expect("should unlock");

        let mock = server
            .mock(
                "GET",
                "/api/jobsearch/v3/autocomplete?&types=geoareaid&q=abc&limit=1",
            )
            .with_body(r#"{"geoareaid": {"completions": [{"id":3000}]}}"#)
            .create();
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let job = JobIndex::new_with_base(url.clone());
                let options = &FetchOptions {
                    query_options: QueryOptions::Query {
                        job_name: None,
                        job_regions: vec!["abc".to_string()],
                        job_tags: vec![],
                    },
                    size_options: SizeOptions::Page {
                        page_size: 20,
                        page: 3,
                    },
                };
                let mut query = job.create_query(options).await;

                // mock.remove();   // let query: Vec<(String, String)> =
                // //     query.unwrap().collect().await;

                // assert_eq!(query, [("abc".to_string(), "abc".to_string())]);
            })
    }
}
