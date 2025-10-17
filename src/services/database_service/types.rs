pub use chrono::{DateTime, Utc};

pub struct JobApplications {
    pub applications: Vec<JobApplication>,
}

pub struct JobApplication {
    pub job: Job,
}
#[allow(unused)]
pub struct JobId(u64);
#[derive(Debug)]
pub struct Job {
    pub job_info: JobInfo,
    pub created_at: DateTime<Utc>,
    pub last_date: Option<DateTime<Utc>>,

    pub company_info: CompanyInfo,
    pub locations: Vec<Location>,

    pub contact_info: Option<ContactInfo>,
}
#[derive(Debug)]

pub struct JobTag {
    pub name: &'static str,
}
#[derive(Debug)]

pub struct JobInfo {
    pub job_url: JobUrl,
    pub title: Title,
    pub description: Description,
    pub job_tags: Vec<JobTag>,
}
#[derive(Debug)]

pub struct JobUrl(pub String);

#[derive(Debug)]

pub struct Title(pub String);

#[derive(Debug)]

pub struct Description(pub String);
#[derive(Debug)]

pub struct CompanyInfo {
    pub name: String,
    pub logo_url: String,
}
#[derive(Debug)]

pub struct Location {
    pub address: String,
    pub geo_location: (f64, f64),
}
#[derive(Debug)]

pub struct ContactInfo {
    pub name: String,
    pub phone_number: PhoneNumber,
    pub email: String,
}
#[derive(Debug)]

pub struct PhoneNumber(pub String);
