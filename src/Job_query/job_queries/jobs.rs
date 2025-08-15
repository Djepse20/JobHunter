use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::Job_query::job_queries::{Job, JobFetcher, database::DataBase, options::FetchOptions};

pub struct Jobs {
    database: Option<DataBase>,
    job_fetchers: Vec<Box<dyn JobFetcher + Send + Sync>>,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            database: None,
            job_fetchers: vec![],
        }
    }
}

impl Jobs {
    pub async fn fetch_jobs(&self, options: FetchOptions) {
        for fetcher in &self.job_fetchers {
            fetcher.fetch_all_jobs_with_options_and_db(&options, self.database.as_ref()).await;
        }
    }
}

impl Jobs {
    pub fn add_fetcher<J: JobFetcher + Send + Sync + 'static>(mut self, fetcher: J) -> Jobs {
        self.job_fetchers.push(Box::new(fetcher));
        self
    }

    pub fn add_database(mut self, database: DataBase) -> Jobs {
        self.database = Some(database);
        self
    }
}
