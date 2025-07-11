use crate::Job_query::job_queries::{JobFetchers, JobTags, database::DataBase};

pub struct FetchOptions {
    pub query_options: QueryOptions,
    pub size_options: SizeOptions,
}

pub enum DataBaseOptions {
    QuerySites(Vec<JobFetchers>),
    QueryEveryThing,
    QueryOnlyDb,
}

impl FetchOptions {
    pub fn full() -> FetchOptions {
        FetchOptions {
            query_options: QueryOptions::All,
            size_options: SizeOptions::All,
        }
    }
}

pub enum QueryOptions {
    Query {
        job_name: Option<String>,
        job_regions: Vec<String>,
        job_tags: Vec<String>,
        filter_applied: bool,
    },
    All,
}

pub enum SizeOptions {
    Page {
        page_size: usize,
        page_number: usize,
    },
    NotPaged {
        jobs: usize,
    },
    All,
}

impl SizeOptions {
    pub fn job_num_to_query(&self, total_jobs: usize) -> usize {
        match self {
            Self::Page {
                page_size,
                page_number,
            } => *page_size.min(&total_jobs),

            Self::NotPaged { jobs } => *jobs.min(&total_jobs),

            Self::All => total_jobs,
        }
    }
}
