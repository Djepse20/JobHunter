use chrono::{DateTime, NaiveDateTime, Utc};
use futures::StreamExt;
use futures::stream;
use serde::Deserialize;
use serde::Deserializer;
use std::marker::PhantomData;

use crate::job_fetchers::job_index::fetcher::JobIndex;
use crate::job_fetchers::job_index::preview::JobPreview;
use crate::job_fetchers::job_index::preview::JobPreviews;
use crate::services::database_service::types::Job;
pub trait DateFormat {
    const DATE_FORMAT: &'static str;
}
impl<J: DateFormat> JobPreview<'_, J> {
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

macro_rules! preview_impl {
    (
  struct $job:ident {
            $(#[$job_url_meta:meta])*
            job_url: &'a str,
            $(#[$created_at_meta:meta])*
            created_at: chrono::DateTime<chrono::Utc>,
        }
    ) => {
            impl<'de> TryFrom<&'de [u8]> for JobPreview<'de, $job> {
        type Error = ();
            fn try_from(val: &'de [u8]) -> Result<Self, Self::Error>

            {
                #[derive(Debug, Clone, Deserialize)]

                struct Tmp<'a> {
                    $(#[$job_url_meta])*
                    job_url: &'a str,
                    $(#[$created_at_meta])*
                    created_at: DateTime<chrono::Utc>,
                }



                let tmp: Tmp<'de> = serde_json::from_slice(val)
                    .map_err(|_|())?;

                Ok(JobPreview {
                    job_url: tmp.job_url,
                    date: tmp.created_at,
                    full_post: val,
                    _phantom: PhantomData,
                })
            }
        }
    };
}

preview_impl!(
    struct JobIndex {
        #[serde(rename = "T::")]
        job_url: &'a str,
        #[serde(deserialize_with = "JobPreview::<JobIndex>::deserialize")]
        #[serde(rename = "haha")]
        created_at: chrono::DateTime<chrono::Utc>,
    }
);

impl<T> JobPreviews<T> for JobPreview<'_, T>
where
    for<'de> JobPreview<'de, T>: TryFrom<&'de [u8]>,
{
    fn unique_previews<'b, 'c>(
        (json, jobs_to_take, offset): (&'c str, usize, usize),
        newest_job: &'b Job,
    ) -> Option<impl StreamExt<Item = JobPreview<'c, T>>> {
        let jobs_iter = stream::iter(
            serde_json::Deserializer::from_str(json)
                .into_iter_seq::<&[u8]>()
                .flat_map(|val| Some(JobPreview::<T>::try_from(val.ok()?)))
                .skip(offset)
                .take(jobs_to_take)
                .take_while(|job| match job {
                    Ok(job) => newest_job.created_at > job.date,
                    Err(_) => false,
                })
                .flatten(),
        );

        Some(jobs_iter)
    }
}
