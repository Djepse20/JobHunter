use crate::Job_query::job_queries::{
    Job, JobFetcher, JobFetchers, database::DataBase, options::FetchOptions,
};

pub struct Jobs {
    page_size: usize,
    database: DataBase,
    job_siters: Vec<JobFetchers>,
}
