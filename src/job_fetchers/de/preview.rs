use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use serde::Deserializer;
use std::marker::PhantomData;

use crate::job_fetchers::job_index::preview::JobPreview;
use crate::job_fetchers::job_index::preview::JobPreviews;
use crate::services::database_service::types::Job;
pub trait DeserializableJob {
    const JOB_URL_ALIAS: &'static str;
    const CREATED_AT_ALIAS: &'static str;

    const DATE_FORMAT: &'static str;
    const DATE_FORMAT_SIZE: usize;
}
impl<J: DeserializableJob> JobPreview<'_, J> {
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, J::DATE_FORMAT)
            .map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

impl<'a, J: DeserializableJob> JobPreview<'a, J> {
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
impl<'de, T: DeserializableJob> Deserialize<'de> for JobPreview<'de, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Clone, Deserialize)]

        struct Tmp<'a, T: DeserializableJob> {
            job_url: &'a str,
            #[serde(deserialize_with = "JobPreview::<T>::deserialize")]
            #[serde(alias = "T::")]
            created_at: DateTime<chrono::Utc>,
            #[serde(skip)]
            pub _phantom: PhantomData<T>,
        }
        let raw_value: &[u8] = <&[u8]>::deserialize(deserializer)?;

        let tmp: Tmp<'de, T> = serde_json::from_slice(raw_value)
            .map_err(serde::de::Error::custom)?;

        Ok(JobPreview {
            job_url: tmp.job_url,
            date: tmp.created_at,
            full_post: raw_value,
            _phantom: PhantomData,
        })
    }
}
impl<T: DeserializableJob> JobPreviews<T> for JobPreview<'_, T> {
    fn unique_previews<'b, 'c>(
        (json, jobs_to_take, offset): (&'c str, usize, usize),
        newest_job: &'b Job,
    ) -> Option<Vec<JobPreview<'c, T>>> {
        let jobs_iter = serde_json::Deserializer::from_str(json)
            .into_iter_seq::<JobPreview<T>>()
            .skip(offset)
            .take(jobs_to_take)
            .take_while(|job| match job {
                Ok(job) => newest_job.created_at > job.date,
                Err(_) => false,
            })
            .flatten();

        Some(jobs_iter.collect())
    }
}
