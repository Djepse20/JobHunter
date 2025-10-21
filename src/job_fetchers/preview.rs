use std::marker::PhantomData;

use chrono::{DateTime, NaiveDateTime, ParseError, Utc};
use futures::{StreamExt, stream};
use serde::Deserialize;

pub fn parse_date<J: DateFormat>(
    date: &str,
) -> Result<DateTime<Utc>, ParseError> {
    let dt = NaiveDateTime::parse_from_str(&date, J::DATE_FORMAT)?;

    Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
}
pub trait DateFormat {
    const DATE_FORMAT: &'static str;
}

#[derive(Debug, Clone)]
pub struct JobPreview<'a, J> {
    pub job_url: &'a str,
    pub date: DateTime<chrono::Utc>,
    pub full_post: &'a [u8],
    _phantom: PhantomData<J>,
}

impl<'a, J> JobPreview<'a, J> {
    pub fn new(
        job_url: &'a str,
        date: DateTime<chrono::Utc>,
        full_post: &'a [u8],
    ) -> Self {
        Self {
            job_url,
            date,
            full_post,
            _phantom: PhantomData,
        }
    }
}

use crate::services::database_service::types::Job;

impl<'a, J> PartialEq<Job> for JobPreview<'a, J> {
    fn eq(&self, other: &Job) -> bool {
        self.date == other.created_at
    }
}

impl<'a, J> PartialEq<JobPreview<'a, J>> for Job {
    fn eq(&self, other: &JobPreview<'a, J>) -> bool {
        self.created_at == other.date
    }
}

impl<'a, J> PartialOrd<Job> for JobPreview<'a, J> {
    fn partial_cmp(&self, other: &Job) -> Option<std::cmp::Ordering> {
        self.date.partial_cmp(&other.created_at)
    }
}
impl<'a, J> PartialOrd<JobPreview<'a, J>> for Job {
    fn partial_cmp(
        &self,
        other: &JobPreview<'a, J>,
    ) -> Option<std::cmp::Ordering> {
        self.created_at.partial_cmp(&other.date)
    }
}
pub trait UniqueJobs
where
    Self: Sized,
{
    fn unique_jobs<'c>(
        jobs: &'c [u8],
        jobs_to_take: usize,
        offset: usize,
        newest_job: Option<&Job>,
    ) -> Option<impl StreamExt<Item = JobPreview<'c, Self>>>;
}

pub fn unique_job<'c, T, U>(
    iter: U,
    jobs_to_take: usize,
    offset: usize,
    newest_job: Option<&Job>,
) -> Option<impl StreamExt<Item = JobPreview<'c, T>>>
where
    U: IntoIterator<Item = &'c [u8]>,
    for<'de> JobPreview<'de, T>: TryFrom<&'de [u8]>,
{
    let jobs_iter = stream::iter(
        iter.into_iter()
            .flat_map(|val| Some(JobPreview::<T>::try_from(val)))
            .skip(offset)
            .take(jobs_to_take)
            .take_while(move |job| match job {
                Ok(job) => {
                    matches!(newest_job, Some(newest_job) if newest_job > job)
                }
                Err(_) => false,
            })
            .flatten(),
    );

    Some(jobs_iter)
}
