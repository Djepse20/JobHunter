pub use chrono::{DateTime, Utc};

pub struct JobApplications {
    pub applications: Vec<JobApplication>,
}

pub struct JobApplication {
    pub job: Job,
}
#[allow(unused)]
pub struct JobId(u64);

pub struct Job {
    pub job_info: JobInfo,
    pub created_at: DateTime<Utc>,
    pub last_date: Option<DateTime<Utc>>,

    pub company_info: CompanyInfo,
    pub job_tags: Vec<JobTag>,
    pub contact_info: Option<ContactInfo>,
}

pub struct JobTag {
    pub name: String,
}

pub struct JobInfo {
    pub job_url: String,
    pub title: String,
    pub description: String,
}

pub struct CompanyInfo {
    pub name: String,
    pub locations: Vec<Location>,
}

pub struct Location {
    pub address: String,
    pub geo_location: (f64, f64),
}

pub struct ContactInfo {
    pub name: String,
    pub phone_number: PhoneNumber,
    pub email: String,
}

pub struct PhoneNumber(pub String);
