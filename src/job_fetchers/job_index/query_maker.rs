use futures::{StreamExt, stream};
use serde::Deserialize;
use serde_json::Value;
use sqlx::types::JsonRawValue;

use crate::{
    job_fetchers::job_index::fetcher::JobIndex,
    util::{
        from_query::CreateQuery,
        options::{FetchOptions, QueryOptions, SizeOptions},
    },
};

pub struct JobIndexQuery(String);

impl AsRef<str> for JobIndexQuery {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl CreateQuery<&QueryOptions> for JobIndex {
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
            } => Ok(self.get_region_query(job_regions).await.chain(
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
            .filter_map(async |region| self.take_first_region(region).await)
    }
    pub async fn take_first_region(
        &self,
        job_region: &str,
    ) -> Option<(Arc<str>, Arc<str>)> {
        #[derive(Deserialize)]
        struct GeoLoc<'a> {
            #[serde(borrow)]
            geoareaid: Completions<'a>,
        }
        #[derive(Deserialize)]

        struct Completions<'a> {
            #[serde(borrow)]
            completions: &'a JsonRawValue,
        }
        #[derive(Deserialize)]

        struct Uuid {
            id: u64,
        }
        let mut url = self.urls.job_regions.clone();
        url.query_pairs_mut()
            .append_pair("q", job_region)
            .append_pair("limit", "1")
            .finish();
        let res = reqwest::get(url.as_str()).await.ok()?;
        let json = res.text().await.ok()?;
        let loc: GeoLoc = serde_json::from_str(&json).ok()?;
        let uuid =
            serde_json::Deserializer::from_str(loc.geoareaid.completions.get())
                .into_iter_seq::<Uuid>()
                .find_map(|uuid| uuid.ok())?;

        Some(("geoareaid".into(), uuid.id.to_string().into()))
    }
}
use std::sync::Arc;

impl CreateQuery<&FetchOptions> for JobIndex {
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

impl CreateQuery<(Arc<[(Arc<str>, Arc<str>)]>, &SizeOptions)> for JobIndex {
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
    use futures::FutureExt;
    use serde_json::json;
    use std::sync::{Arc, LazyLock, Mutex};
    use tokio_stream::StreamExt;
    use url::Url;

    use crate::{
        job_fetchers::job_index::fetcher::JobIndex,
        util::{
            from_query::CreateQuery,
            options::{FetchOptions, QueryOptions, SizeOptions},
        },
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
    // macro_rules! mock_with_body {
    //     ($($mock:ident = { method = $method:expr, path = $path:expr, body = body:expr }),*, test = async $test:block, server:expr) => {
    //         $(let $(mock) = server
    //         .mock(
    //             $method,
    //             $path,
    //         )
    //         .with_body(
    //             $body,
    //         )
    //         .create();),*
    //         tokio::runtime::Builder::new_current_thread()
    //         .enable_all()
    //         .build()
    //         .unwrap()
    //         .block_on(async $block);
    //     };

    // }
    #[test]

    fn test_query_options() {
        let (url, server) =
            &mut *MOCK_URL_SERVER.lock().expect("should unlock");

        let mock = server
            .mock(
                "GET",
                "/api/jobsearch/v3/autocomplete?&types=geoareaid&q=abc&limit=1",
            )
            .with_body(
                r#"{"geoareaid": {"completions": [{"id":3000,"abc":1000}]}}"#,
            )
            .create();
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let job = JobIndex::new_with_base(url.clone());
                let options = QueryOptions::Query {
                    job_name: None,
                    job_regions: vec!["abc".to_string()],
                    job_tags: vec![],
                };
                let mut query =
                    job.create_query(&options).await.expect("should unwrap");

                let query: Vec<(String, String)> = query
                    .map(|val| (val.0.to_string(), val.1.to_string()))
                    .collect()
                    .await;

                assert_eq!(
                    query,
                    [("geoareaid".to_string(), "3000".to_string())]
                );
            })
    }

    #[test]

    fn test_size_options() {
        let (url, server) =
            &mut *MOCK_URL_SERVER.lock().expect("should unlock");

        let mock = server
            .mock(
                "GET",
                "/api/jobsearch/v3/autocomplete?&types=geoareaid&q=abc&limit=1",
            )
            .with_body(
                r#"{"geoareaid": {"completions": [{"id":3000,"abc":1000}]}}"#,
            )
            .create();
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let job = JobIndex::new_with_base(url.clone());
                let options = QueryOptions::Query {
                    job_name: None,
                    job_regions: vec!["abc".to_string()],
                    job_tags: vec![],
                };
                let mut query =
                    job.create_query(&options).await.expect("should unwrap");

                let query: Vec<(String, String)> = query
                    .map(|val| (val.0.to_string(), val.1.to_string()))
                    .collect()
                    .await;

                assert_eq!(
                    query,
                    [("geoareaid".to_string(), "3000".to_string())]
                );
            })
    }
}
