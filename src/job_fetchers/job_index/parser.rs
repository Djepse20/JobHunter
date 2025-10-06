use core::error;

use chrono::{DateTime, Utc};
use memchr::memmem;
use serde::Deserialize;

use crate::{
    job_fetchers::{
        JOB_TAGS, job_index::fetcher::JobIndex, job_previewer::JobIntermediate,
    },
    services::database_service::types::{
        CompanyInfo, Description, Job, JobInfo, JobTag, JobUrl, Location, Title,
    },
};

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct RawHtml<'a>(#[serde(borrow)] &'a RawValue);

impl<'a> RawHtml<'a> {
    fn extract_jobs_tags(description: &str) -> Vec<JobTag> {
        JOB_TAGS
            .iter()
            .filter_map(|(tag, matchers)| {
                matchers
                    .iter()
                    .find_map(|needle: &&str| {
                        memmem::find(description.as_bytes(), needle.as_bytes())
                    })
                    .map(|_| JobTag { name: tag })
            })
            .collect()
    }
}

impl<'a> TryFrom<RawHtml<'a>> for (Vec<JobTag>, Description) {
    type Error = ();
    fn try_from(value: RawHtml<'a>) -> Result<Self, Self::Error> {
        let start_seq = "</div>\\n\\n<p>";
        let end_seq = "</p>\\n";
        let string = value.0.get();

        let start_idx = string.find(start_seq).ok_or(())? + start_seq.len();

        let end_idx = string.find(end_seq).ok_or(())?;

        let description: String = string[start_idx..end_idx]
            .split("<p>")
            .flat_map(|x| x.split("</p>"))
            .filter(|x| !x.is_empty())
            .collect();
        Ok((
            RawHtml::extract_jobs_tags(&description),
            Description(description),
        ))
    }
}

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct RawAddresses<'a>(#[serde(borrow)] &'a RawValue);

impl<'a> RawAddresses<'a> {
    fn deserialize_location<'de, D>(
        deserializer: D,
    ) -> Result<(f64, f64), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Tmp {
            latitude: f64,
            longitude: f64,
        }

        let s = Tmp::deserialize(deserializer)?;
        Ok((s.latitude, s.longitude))
    }
}
impl<'a> TryFrom<RawAddresses<'a>> for Vec<Location> {
    type Error = ();
    fn try_from(value: RawAddresses<'a>) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        struct Addresses<'a> {
            city: &'a str,
            #[serde(deserialize_with = "RawAddresses::deserialize_location")]
            coordinates: (f64, f64),
            #[serde(rename(deserialize = "simple_string"))]
            address: &'a str,
            #[serde(rename(deserialize = "zipcode"))]
            zip_code: &'a str,
        }
        Ok(serde_json::Deserializer::from_str(value.0.get())
            .into_iter_seq::<Addresses>()
            .filter_map(|loc| {
                loc.ok().map(|loc| Location {
                    address: loc.address.to_owned(),
                    geo_location: loc.coordinates,
                })
            })
            .collect())
    }
}
#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct RawCompany<'a>(#[serde(borrow)] &'a RawValue);
impl<'a> TryFrom<RawCompany<'a>> for CompanyInfo {
    type Error = ();

    fn try_from(value: RawCompany<'a>) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]
        struct Company {
            name: String,
            get_logo_company: String,
        }
        let company_info: Company =
            serde_json::from_str(value.0.get()).map_err(|_| ())?;
        Ok(CompanyInfo {
            name: company_info.name,
            logo_url: company_info.get_logo_company,
        })
    }
}
#[derive(Debug, Deserialize)]
struct RawHeadlineAndUrl<'a> {
    #[serde(borrow)]
    url: &'a RawValue,
    #[serde(borrow)]
    headline: &'a RawValue,
}

impl<'a> TryFrom<RawHeadlineAndUrl<'a>> for (JobUrl, Title) {
    type Error = ();

    fn try_from(value: RawHeadlineAndUrl<'a>) -> Result<Self, Self::Error> {
        Ok((
            JobUrl(value.url.get().to_owned()),
            Title(value.headline.get().to_owned()),
        ))
    }
}

use crate::job_fetchers::job_previewer::DateTimeSerde;
use serde_json::value::RawValue;
#[derive(Deserialize)]
struct JobIndexData<'a> {
    #[serde(borrow)]
    html: RawHtml<'a>,
    #[serde(borrow)]
    company: RawCompany<'a>,
    #[serde(borrow, deserialize_with = "RawHeadlineAndUrl::deserialize")]
    headline_and_url: RawHeadlineAndUrl<'a>,
    #[serde(borrow)]
    addresses: RawAddresses<'a>,

    #[serde(deserialize_with = "DateTimeSerde::<JobIndex>::deserialize")]
    #[serde(rename(deserialize = "lastdate"))]
    last_date: DateTime<Utc>,
}

impl<'a> TryFrom<JobIntermediate<'a, JobIndex>> for Job {
    type Error = ();
    fn try_from(
        value: JobIntermediate<'a, JobIndex>,
    ) -> Result<Self, Self::Error> {
        let job_index: JobIndexData =
            serde_json::from_str(value.full_raw.get()).map_err(|_| ())?;
        let (job_tags, description) = job_index.html.try_into()?;
        let (job_url, title) = job_index.headline_and_url.try_into()?;

        let locations: Vec<Location> = job_index.addresses.try_into()?;

        let company_info: CompanyInfo = job_index.company.try_into()?;

        Ok(Job {
            job_info: JobInfo {
                job_url,
                title,
                description,
                job_tags,
            },
            created_at: value.date,
            last_date: Some(job_index.last_date),
            company_info,
            locations,
            contact_info: None,
        })
    }
}
