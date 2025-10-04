use std::ops::Range;

use axum::response::IntoResponse;
use axum_extra::extract::{Query, QueryRejection};
use axum_macros::FromRequestParts;
use chrono::format::Item;
use chrono::offset;
use reqwest::StatusCode;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, de};
use serde_json::json;

#[derive(FromRequestParts, Debug, Serialize)]
#[from_request(via(Query), rejection(ApiError))]

pub struct FetchOptions {
    pub query_options: QueryOptions,
    pub size_options: SizeOptions,
}

// We implement `IntoResponse` for our extractor so it can be used as a response
impl IntoResponse for FetchOptions {
    fn into_response(self) -> axum::response::Response {
        let fetch_options = self;
        axum::Json(fetch_options).into_response()
    }
}

// We create our own rejection type
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

// We implement `From<JsonRejection> for ApiError`
impl From<QueryRejection> for ApiError {
    fn from(rejection: QueryRejection) -> Self {
        Self {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

// We implement `IntoResponse` so `ApiError` can be used as a response
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let payload = json!({
            "message": self.message,
            "origin": "derive_from_request"
        });

        (self.status, axum::Json(payload)).into_response()
    }
}

impl<'de> Deserialize<'de> for FetchOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Params {
            job_name: Option<String>,

            #[serde(default)]
            job_regions: Vec<String>,
            #[serde(default)]
            job_tags: Vec<String>,
            page_size: Option<usize>,
            page: Option<usize>,
            jobs: Option<usize>,
        }

        let params: Params = Params::deserialize(deserializer)?;
        let size_options = match (params.page_size, params.page, params.jobs) {
            (Some(page_size), Some(page), None) => {
                SizeOptions::Page { page_size, page }
            }

            (None, None, Some(jobs)) => SizeOptions::NotPaged { jobs },
            (None, None, None) => SizeOptions::All,
            _ => {
                return Err(D::Error::custom(
                    "please provied either page_size and page or jobs",
                ));
            }
        };
        let query_options = if params.job_name.is_some()
            || params.job_regions.is_empty()
            || !params.job_tags.is_empty()
        {
            QueryOptions::Query {
                job_name: params.job_name,
                job_regions: params.job_regions,
                job_tags: params.job_tags,
            }
        } else {
            QueryOptions::All
        };

        Ok(FetchOptions {
            size_options,
            query_options,
        })
    }
}

impl FetchOptions {
    pub fn full() -> FetchOptions {
        FetchOptions {
            query_options: QueryOptions::All,
            size_options: SizeOptions::All,
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub enum QueryOptions {
    #[default]
    All,
    #[serde(untagged)]
    Query {
        job_name: Option<String>,
        job_regions: Vec<String>,
        job_tags: Vec<String>,
    },
}
#[derive(Debug, Default, Deserialize, Serialize)]

pub enum SizeOptions {
    #[default]
    All,
    #[serde(untagged)]
    Page { page_size: usize, page: usize },
    #[serde(untagged)]
    NotPaged { jobs: usize },
}

impl SizeOptions {
    // TODO, change page_size to an assoicated constant on a trait implemented on JobFetcher
    pub fn job_num_to_query(
        &self,
        total_jobs: usize,
        max_page_size: usize,
        start_offset: usize,
    ) -> (usize, usize, impl IntoIterator<Item = (usize, usize)>) {
        let get_num_pages = |jobs: usize| -> usize {
            (match (jobs / max_page_size, jobs % max_page_size) {
                (pages, 1..) => pages + 1,

                (pages, _) => pages,
            }) + start_offset
        };
        fn get_pages_with_jobs(
            jobs: usize,
            max_page_size: usize,
            pages: Range<usize>,
            mut first_page: Option<usize>,
        ) -> impl Iterator<Item = (usize, usize)> {
            pages.into_iter().scan(jobs, move |remaining_jobs, page| {
                if *remaining_jobs == 0 {
                    return None;
                }

                let jobs_taken =
                    if let Some(first_page @ 1..) = first_page.take() {
                        (*remaining_jobs).min(first_page)
                    } else {
                        (*remaining_jobs).min(max_page_size)
                    };

                *remaining_jobs -= jobs_taken;

                Some((jobs_taken, page))
            })
        }
        match *self {
            Self::Page { page_size, page } => {
                let start = (page_size * page).min(total_jobs);
                let end = (page_size * (page + 1)).min(total_jobs);
                //number of jobs remaining.
                let jobs = end - start;
                // if end and start are bigger than total jobs, then no jobs are found.
                //otherwise, they will allways find a job

                if jobs == 0 {
                    let pages_with_jobs =
                        get_pages_with_jobs(0, max_page_size, 0..0, None);
                    return (0, jobs, pages_with_jobs);
                }

                let start_page: usize = start / max_page_size;

                let offset = start - (max_page_size * start_page);
                //this number is the number of jobs left, if we assume there are infite jobs left.

                let jobs_on_first_page = max_page_size - offset;
                //we need to add offset here, casuse if we need to go between pages.
                let end_page = start_page + get_num_pages(jobs + offset);

                let pages = start_page + start_offset..end_page;

                let pages_with_jobs = get_pages_with_jobs(
                    jobs,
                    max_page_size,
                    pages,
                    Some(jobs_on_first_page),
                );
                (offset, jobs, pages_with_jobs)
            }

            Self::NotPaged { jobs } => {
                let jobs = jobs.min(total_jobs);
                let pages = start_offset..get_num_pages(jobs);
                let pages_with_jobs =
                    get_pages_with_jobs(jobs, max_page_size, pages, None);
                (0, jobs, pages_with_jobs)
            }

            Self::All => {
                let jobs = total_jobs;
                let pages = start_offset..get_num_pages(jobs);
                let pages_with_jobs =
                    get_pages_with_jobs(jobs, max_page_size, pages, None);
                (0, jobs, pages_with_jobs)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::Job_query::job_queries::job_index::JobIndex;

    use super::*;

    #[test]
    fn first_page() {
        let size_options: SizeOptions = SizeOptions::Page {
            page_size: 3,
            page: 3,
        };
        let page_size = JobIndex::PAGE_SIZE;
        let total_jobs = 100;
        let start_offset = 1;
        let (offset, jobs, pages) =
            size_options.job_num_to_query(total_jobs, page_size, start_offset);

        assert_eq!(offset, 9);
        assert_eq!(jobs, 3);
        let iter = [(3, 1)];
        assert_eq!(
            pages
                .into_iter()
                .collect::<Vec<(usize, usize)>>()
                .as_slice(),
            iter.as_slice()
        );
    }

    #[test]

    fn between_two_pages() {
        let size_options: SizeOptions = SizeOptions::Page {
            page_size: 24,
            page: 3,
        };
        let page_size = JobIndex::PAGE_SIZE;
        let total_jobs = 200;
        let start_offset = 1;
        let (offset, jobs, pages) =
            size_options.job_num_to_query(total_jobs, page_size, start_offset);

        assert_eq!(offset, 12);
        assert_eq!(jobs, 24);
        let iter = [(8, 4), (16, 5)];
        assert_eq!(
            pages
                .into_iter()
                .collect::<Vec<(usize, usize)>>()
                .as_slice(),
            iter.as_slice()
        );
    }

    #[test]

    fn between_two_pages_less_than() {
        let size_options: SizeOptions = SizeOptions::Page {
            page_size: 24,
            page: 3,
        };
        let page_size = JobIndex::PAGE_SIZE;
        let total_jobs = 92;
        let start_offset = 1;
        let (offset, jobs, pages) =
            size_options.job_num_to_query(total_jobs, page_size, start_offset);

        assert_eq!(offset, 12);
        assert_eq!(jobs, 20);
        let iter = [(8, 4), (12, 5)];
        assert_eq!(
            pages
                .into_iter()
                .collect::<Vec<(usize, usize)>>()
                .as_slice(),
            iter.as_slice()
        );
    }

    #[test]

    fn between_two_pages_one_element_from_page() {
        let size_options: SizeOptions = SizeOptions::Page {
            page_size: 19,
            page: 1,
        };
        let page_size = JobIndex::PAGE_SIZE;
        let total_jobs = 72;
        let start_offset = 1;
        let (offset, jobs, pages) =
            size_options.job_num_to_query(total_jobs, page_size, start_offset);

        assert_eq!(offset, 19);
        assert_eq!(jobs, 19);
        let iter = [(1, 1), (18, 2)];
        assert_eq!(
            pages
                .into_iter()
                .collect::<Vec<(usize, usize)>>()
                .as_slice(),
            iter.as_slice()
        );
    }
    #[test]

    fn between_three_pages_uneven() {
        let size_options: SizeOptions = SizeOptions::Page {
            page_size: 50,
            page: 1,
        };
        let page_size = JobIndex::PAGE_SIZE;
        let total_jobs = 300;
        let start_offset = 1;
        let (offset, jobs, pages) =
            size_options.job_num_to_query(total_jobs, page_size, start_offset);

        assert_eq!(offset, 10);
        assert_eq!(jobs, 50);
        let iter = [(10, 3), (20, 4), (20, 5)];
        assert_eq!(
            pages
                .into_iter()
                .collect::<Vec<(usize, usize)>>()
                .as_slice(),
            iter.as_slice()
        );
    }
    #[test]

    fn between_three_pages_even() {
        let size_options: SizeOptions = SizeOptions::Page {
            page_size: 50,
            page: 2,
        };
        let page_size = JobIndex::PAGE_SIZE;
        let total_jobs = 300;
        let start_offset = 1;
        let (offset, jobs, pages) =
            size_options.job_num_to_query(total_jobs, page_size, start_offset);

        assert_eq!(offset, 0);
        assert_eq!(jobs, 50);
        let iter = [(20, 6), (20, 7), (10, 8)];
        assert_eq!(
            pages
                .into_iter()
                .collect::<Vec<(usize, usize)>>()
                .as_slice(),
            iter.as_slice()
        );
    }

    #[test]

    fn out_of_bounds_start() {
        let size_options: SizeOptions = SizeOptions::Page {
            page_size: 23,
            page: 2,
        };
        let page_size = JobIndex::PAGE_SIZE;
        let total_jobs = 25;
        let start_offset = 1;
        let (offset, jobs, pages) =
            size_options.job_num_to_query(total_jobs, page_size, start_offset);

        assert_eq!(offset, 0);
        assert_eq!(jobs, 0);
        let iter = [];
        assert_eq!(
            pages
                .into_iter()
                .collect::<Vec<(usize, usize)>>()
                .as_slice(),
            iter.as_slice()
        );
    }

    #[test]

    fn one_element_second_page_from_index() {
        let size_options: SizeOptions = SizeOptions::Page {
            page_size: 1,
            page: 20,
        };
        let page_size = JobIndex::PAGE_SIZE;
        let total_jobs = 21;
        let start_offset = 1;
        let (offset, jobs, pages) =
            size_options.job_num_to_query(total_jobs, page_size, start_offset);

        assert_eq!(offset, 0);
        assert_eq!(jobs, 1);
        let iter = [(1, 2)];
        assert_eq!(
            pages
                .into_iter()
                .collect::<Vec<(usize, usize)>>()
                .as_slice(),
            iter.as_slice()
        );
    }
}
