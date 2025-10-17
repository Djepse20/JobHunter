use chrono::{DateTime, Utc};
use memchr::memmem;
use serde::Deserialize;
use serde::de::Error;

use crate::job_fetchers::preview::{JobPreview, parse_date};
use crate::{
    job_fetchers::{JOB_TAGS, job_index::fetcher::JobIndex},
    services::database_service::types::{
        CompanyInfo, Description, Job, JobInfo, JobTag, JobUrl, Location, Title,
    },
};

pub struct JobIndexHtmlInfo((Vec<JobTag>, Description));

impl JobIndexHtmlInfo {
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

impl<'de> Deserialize<'de> for JobIndexHtmlInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let start_seq = b"</div>\\n\\n<p>";
        let end_seq = b"</p>\\n";
        let string = <&str>::deserialize(deserializer)?;

        let start_idx = memmem::find(string.as_bytes(), start_seq)
            .ok_or(serde::de::Error::custom("start seq dosent exist"))?
            + start_seq.len();

        let end_idx = memmem::find(string.as_bytes(), end_seq)
            .ok_or(serde::de::Error::custom("start seq dosent exist"))?;

        let description: String = string[start_idx..end_idx]
            .split("<p>")
            .flat_map(|x| x.split("</p>"))
            .filter(|x| !x.is_empty())
            .collect();
        Ok(JobIndexHtmlInfo((
            JobIndexHtmlInfo::extract_jobs_tags(&description),
            Description(description),
        )))
    }
}

struct JobIndexLocation(Vec<Location>);

impl JobIndexLocation {
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
impl<'de> Deserialize<'de> for JobIndexLocation {
    fn deserialize<D>(mut deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Addresses<'a> {
            city: &'a str,
            #[serde(
                deserialize_with = "JobIndexLocation::deserialize_location"
            )]
            coordinates: (f64, f64),
            #[serde(rename(deserialize = "simple_string"))]
            address: &'a str,
            #[serde(rename(deserialize = "zipcode"))]
            zip_code: &'a str,
        }

        Ok(JobIndexLocation(
            Vec::<Addresses>::deserialize(deserializer)?
                .into_iter()
                .map(|loc| Location {
                    address: loc.address.to_owned(),
                    geo_location: loc.coordinates,
                })
                .collect(),
        ))
    }
}
struct JobIndexCompany(CompanyInfo);

impl<'de> Deserialize<'de> for JobIndexCompany {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Company {
            name: String,
            get_logo_company: String,
        }
        let company_info: Company = Company::deserialize(deserializer)?;
        Ok(JobIndexCompany(CompanyInfo {
            name: company_info.name,
            logo_url: company_info.get_logo_company,
        }))
    }
}
struct JobIndexJobUrl(JobUrl);
impl<'de> Deserialize<'de> for JobIndexJobUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let job_url: String = String::deserialize(deserializer)?;
        Ok(JobIndexJobUrl(JobUrl(job_url)))
    }
}

struct JobIndexTitle(Title);
impl<'de> Deserialize<'de> for JobIndexTitle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let title: String = String::deserialize(deserializer)?;
        Ok(JobIndexTitle(Title(title)))
    }
}

struct JobIndexDate(DateTime<Utc>);

impl<'de> Deserialize<'de> for JobIndexDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let date = <&str>::deserialize(deserializer)?;
        Ok(JobIndexDate(
            parse_date::<JobIndex>(date).map_err(serde::de::Error::custom)?,
        ))
    }
}

#[derive(Deserialize)]
struct JobIndexData {
    html: JobIndexHtmlInfo,
    company: JobIndexCompany,
    title: JobIndexTitle,
    job_url: JobIndexJobUrl,

    #[serde(rename(deserialize = "addresses"))]
    locations: JobIndexLocation,
    #[serde(rename(deserialize = "lastdate"))]
    last_date: JobIndexDate,
}

impl<'de> TryFrom<&'de [u8]> for JobPreview<'de, JobIndex> {
    type Error = ();
    fn try_from(full_post: &'de [u8]) -> Result<Self, Self::Error> {
        #[derive(Deserialize)]

        struct Tmp<'a> {
            job_url: &'a str,
            created_at: JobIndexDate,
        }

        let Tmp {
            job_url,
            created_at: JobIndexDate(date),
        } = serde_json::from_slice(full_post).map_err(|_| ())?;
        Ok(JobPreview::new(job_url, date, full_post))
    }
}

impl<'a> TryFrom<JobPreview<'a, JobIndex>> for Job {
    type Error = ();
    fn try_from(value: JobPreview<'a, JobIndex>) -> Result<Self, Self::Error> {
        let JobIndexData {
            html: JobIndexHtmlInfo((job_tags, description)),
            company: JobIndexCompany(company_info),
            title: JobIndexTitle(title),
            job_url: JobIndexJobUrl(job_url),
            locations: JobIndexLocation(locations),
            last_date: JobIndexDate(last_date),
        } = serde_json::from_slice(value.full_post).map_err(|_| ())?;

        Ok(Job {
            job_info: JobInfo {
                job_url,
                title,
                description,
                job_tags,
            },
            created_at: value.date,
            last_date: Some(last_date),
            company_info,
            locations,
            contact_info: None,
        })
    }
}
