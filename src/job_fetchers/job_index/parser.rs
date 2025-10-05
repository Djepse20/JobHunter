use chrono::{DateTime, Utc};
use memchr::memmem;
use serde::Deserialize;

use crate::{
    job_fetchers::{
        JOB_TAGS, job_index::fetcher::JobIndex, job_previewer::JobIntermediate,
    },
    services::database_service::types::{Job, Location},
};

fn extract_jobs_tags(description: &str) -> Vec<&'static str> {
    JOB_TAGS
        .iter()
        .filter_map(|(tag, matchers)| {
            matchers
                .iter()
                .find_map(|needle: &&str| {
                    memmem::find(description.as_bytes(), needle.as_bytes())
                })
                .map(|_| *tag)
        })
        .collect()
}

fn extract_description(html: &RawValue) -> Option<String> {
    let start_seq = "</div>\\n\\n<p>";
    let end_seq = "</p>\\n";
    let string = html.get();

    let start_idx = string.find(start_seq)? + start_seq.len();

    let end_idx = string.find(end_seq)?;

    let string = string[start_idx..end_idx]
        .split("<p>")
        .flat_map(|x| x.split("</p>"))
        .filter(|x| !x.is_empty())
        .collect();
    Some(string)
}
use serde::de::Deserializer;
pub fn deserialize_location<'de, D>(
    deserializer: D,
) -> Result<(f64, f64), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Tmp {
        latitude: f64,
        longitude: f64,
    }

    let s = Tmp::deserialize(deserializer)?;
    Ok((s.latitude, s.longitude))
}

fn extract_addresses(addresses: &RawValue) -> Vec<Location> {
    #[derive(Deserialize)]
    struct Addresses<'a> {
        city: &'a str,
        #[serde(deserialize_with = "deserialize_location")]
        coordinates: (f64, f64),
        #[serde(rename(deserialize = "simple_string"))]
        address: &'a str,
        #[serde(rename(deserialize = "zipcode"))]
        zip_code: &'a str,
    }

    serde_json::Deserializer::from_str(addresses.get())
        .into_iter_seq::<Addresses>()
        .filter_map(|loc| {
            loc.ok().map(|loc| Location {
                address: loc.address.to_owned(),
                geo_location: loc.coordinates,
            })
        })
        .collect()
}

use crate::job_fetchers::job_previewer::DateTimeSerde;
use serde_json::value::RawValue;

impl<'a> TryFrom<JobIntermediate<'a, JobIndex>> for Job {
    type Error = ();
    fn try_from(
        value: JobIntermediate<'a, JobIndex>,
    ) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        struct JobIndexData<'a> {
            #[serde(borrow)]
            html: &'a RawValue,
            #[serde(borrow)]
            company: &'a RawValue,
            #[serde(borrow)]
            headline: &'a RawValue,
            #[serde(
                deserialize_with = "DateTimeSerde::<JobIndex>::deserialize"
            )]
            #[serde(rename(deserialize = "lastdate"))]
            last_date: DateTime<Utc>,
        }
        let job_index: JobIndexData =
            serde_json::from_str(value.full_raw.get()).map_err(|_| ())?;
        let description = extract_description(job_index.html).ok_or(())?;

        let tags = extract_jobs_tags(&description);
        todo!()
    }
}
