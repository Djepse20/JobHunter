use chrono::DateTime;
use futures::StreamExt;

use crate::{
    job_fetchers::{de::preview::DateFormat, job_index::fetcher::JobIndex},
    services::database_service::types::Job,
};

#[derive(Debug, Clone)]
pub struct JobPreview<'a, J> {
    pub job_url: &'a str,
    pub date: DateTime<chrono::Utc>,
    pub full_post: &'a [u8],
    pub _phantom: PhantomData<J>,
}

pub trait JobPreviews<T> {
    fn unique_previews<'b, 'c>(
        jobs_data: (&'c str, usize, usize),
        newest_job: &'b Job,
    ) -> Option<impl StreamExt<Item = JobPreview<'c, T>>>;
}

impl<'a, T> Eq for JobPreview<'a, T> {}
impl<'a, T> PartialEq for JobPreview<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.job_url == other.job_url && self.date == other.date
    }
}
impl<'a, T> Ord for JobPreview<'a, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).expect("jobs should be comparable")
    }
}
impl<'a, T> PartialOrd for JobPreview<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        //in reverse, cause i want to sort i decending order (earlier dates comes first )
        self.date.partial_cmp(&other.date)
    }
}
use std::{hash::Hash, marker::PhantomData};
impl<'a, T> Hash for JobPreview<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.job_url.hash(state);
    }
}
