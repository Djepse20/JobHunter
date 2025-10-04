pub use chrono::{DateTime, Utc};

pub struct JobApplications {
    pub applications: Vec<JobApplication>,
}

pub struct JobApplication {
    pub job: Job,
}

pub struct JobId(u64);
pub struct Job {
    pub job_info: JobInfo,
    pub created_at: DateTime<Utc>,

    pub company_info: CompanyInfo,
    pub job_tags: Vec<JobTag>,
    pub contact_info: ContactInfo,
}

pub struct JobTag {
    pub tag_id: i64,
    pub name: String,
}

pub struct JobInfo {
    pub job_id: JobId,
    pub job_url: String,

    pub title: String,
    pub description: String,
}

pub struct CompanyInfo {
    pub company_id: i64,
    pub name: String,
    pub email_address: String,
    pub address: String,
}

pub struct ContactInfo {
    pub contact_id: i64,
    pub name: String,
    pub phone_number: PhoneNumber,
    pub email: String,
}

pub struct PhoneNumber(pub String);
