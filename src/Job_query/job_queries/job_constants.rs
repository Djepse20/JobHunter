use std::marker::PhantomData;
use std::marker::PhantomPinned;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserialize;
use serde::Deserializer;
use serde::Serializer;
pub trait JobConstants {
    const DATE_FORMAT: &'static str;
}

pub struct DateTimeSerde<T>(pub PhantomData<T>);

impl<T: JobConstants> DateTimeSerde<T> {
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(T::DATE_FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt =
            NaiveDateTime::parse_from_str(&s, T::DATE_FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

#[derive(Debug, Clone)]
pub struct JobIntermediate<'a, T: JobConstants> {
    pub job_url: &'a str,
    pub date: DateTime<chrono::Utc>,

    pub _phantom: PhantomData<T>,
}

pub struct JobIntermediateWithString<'a, T>
where
    T: JobConstants,
{
    pub job_info: JobIntermediate<'a, T>,
    pub job_string: &'a str,
}

impl<'a, T: JobConstants> TryFrom<&'a str> for JobIntermediateWithString<'a, T>
where
    JobIntermediate<'a, T>: Deserialize<'a>,
{
    type Error = serde_json::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let intermediate: JobIntermediate<T> = serde_json::from_str(&value)?;
        Ok(JobIntermediateWithString {
            job_info: intermediate,
            job_string: value,
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
        self.partial_cmp(other)
            .expect("jobindexjobs should be comparable")
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
