use crate::services::database_service::database::DataBase;
use crate::services::database_service::types::Job;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::StreamExt;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serializer;
use std::marker::PhantomData;
pub trait JobConstants {
    const DATE_FORMAT: &'static str;
    const DATE_FORMAT_SIZE: usize;
}

pub struct DateTimeSerde<T>(pub PhantomData<T>);

impl<T: JobConstants> DateTimeSerde<T> {
    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut buffer = String::with_capacity(T::DATE_FORMAT_SIZE);
        date.format(T::DATE_FORMAT)
            .write_to(&mut buffer)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&buffer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, T::DATE_FORMAT)
            .map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

use serde_json::value::RawValue;

#[derive(Debug, Clone)]
pub struct JobIntermediate<'a, T: JobConstants> {
    pub job_url: &'a str,
    pub date: DateTime<chrono::Utc>,
    pub full_raw: &'a RawValue,
    _phantom: PhantomData<T>,
}

impl<'de, T: JobConstants> Deserialize<'de> for JobIntermediate<'de, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Clone, Deserialize)]

        struct Tmp<'a, T: JobConstants> {
            job_url: &'a str,
            #[serde(deserialize_with = "DateTimeSerde::<T>::deserialize")]
            #[serde(rename(deserialize = "firstdate"))]
            first_date: DateTime<chrono::Utc>,
            #[serde(skip)]
            pub _phantom: PhantomData<T>,
        }
        let raw_value: &RawValue = <&RawValue>::deserialize(deserializer)?;

        let tmp: Tmp<'de, T> = serde_json::from_str(raw_value.get())
            .map_err(serde::de::Error::custom)?;

        Ok(JobIntermediate {
            job_url: tmp.job_url,
            date: tmp.first_date,
            full_raw: raw_value,
            _phantom: PhantomData,
        })
    }
}

impl<'a, T: JobConstants> Eq for JobIntermediate<'a, T> {}
impl<'a, T: JobConstants> PartialEq for JobIntermediate<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.job_url == other.job_url && self.date == other.date
    }
}
impl<'a, T: JobConstants> Ord for JobIntermediate<'a, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).expect("jobs should be comparable")
    }
}
impl<'a, T: JobConstants> PartialOrd for JobIntermediate<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        //in reverse, cause i want to sort i decending order (earlier dates comes first )
        self.date.partial_cmp(&other.date)
    }
}
use std::hash::Hash;
impl<'a, T: JobConstants> Hash for JobIntermediate<'a, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.job_url.hash(state);
    }
}
