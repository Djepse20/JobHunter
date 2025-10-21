pub use chrono::{DateTime, Utc};
use sqlx::Postgres;

use crate::services::database_service::DbDelete;

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
impl DbDelete for Job {
    type DeleteType<'a> = &'a [JobUrl];
    type RetType = ();
    async fn delete<'a, E: sqlx::Executor<'a, Database = Postgres>>(
        executor: E,
        jobs: Self::DeleteType<'a>,
    ) -> Result<Self::RetType, sqlx::Error> {
        let job_urls: Vec<String> =
            jobs.into_iter().map(|job| job.0.to_owned()).collect();
        let jobs = sqlx::query!(
            r#"--sql
            SELECT id FROM job
            WHERE job_url IN (
                SELECT UNNEST($1::text[])
            )
        "#,
            &job_urls
        )
        .fetch_all(executor)
        .await?;

        // let job_ids: Vec<i64> = jobs.into_iter().map(|job| job.id).collect();
        // let tags = sqlx::query!(
        //     r#"--sql
        //   DELETE FROM tags_for_job
        //   WHERE job_id in (
        //     SELECT UNNEST($1::bigint[])
        //   )
        //   RETURNING job_tag_id
        // "#,
        //     &job_ids
        // )
        // .fetch_all(executor)
        // .await?;

        // let locations = sqlx::query!(
        //     r#"--sql
        //   DELETE FROM location_for_job
        //   WHERE job_id in (
        //     SELECT UNNEST($1::bigint[])
        //   )
        // "#,
        //     &job_ids
        // )
        // .execute(executor)
        // .await?;
        // Ok(())
        Ok(())
    }
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
