use std::pin::pin;

use crate::job_fetchers::de::preview::DeserializableJob;
use crate::job_fetchers::job_index::preview::JobPreview;
use crate::services::database_service::types::{
    CompanyInfo, ContactInfo, Job, JobTag,
};
use futures::StreamExt;
use sqlx::{
    Database, Postgres, QueryBuilder, Transaction, query_builder::Separated,
};

#[derive(Debug, Clone)]
pub struct DataBase {
    database: sqlx::PgPool,
}

impl DataBase {
    pub fn new() -> Self {
        DataBase { database: todo!() }
    }
    pub async fn get_newest_job(&self) -> Result<Job, sqlx::Error> {
        todo!()
    }
    pub async fn insert_jobs<'a, T: DeserializableJob>(
        &self,
        jobs: &[Job],
    ) -> Result<Vec<i64>, sqlx::Error> {
        let mut job_ids: Vec<i64> = Vec::with_capacity(jobs.len());
        // these are assumed to be in the correct order. Ie if any one fails that must mean the next ones will also fail.

        for job in jobs {
            if let Ok(id) = self.insert_job(job).await {
                job_ids.push(id);
            } else {
                break;
            }
        }
        Ok(job_ids)
    }

    pub async fn delete_jobs<'a, T: DeserializableJob>(
        &self,
        jobs: &[JobPreview<'a, T>],
    ) -> Result<Vec<i64>, sqlx::Error> {
        let mut job_ids: Vec<i64> = Vec::with_capacity(jobs.len());
        // these are assumed to be in the correct order. Ie if any one fails that must mean the next ones will also fail.

        for job in jobs {
            if let Ok(id) = self.delete_job(&job).await {
                job_ids.push(id);
            } else {
                break;
            }
        }
        Ok(job_ids)
    }

    pub async fn delete_job<'a, T: DeserializableJob>(
        &self,
        jobs: &JobPreview<'a, T>,
    ) -> Result<i64, sqlx::Error> {
        todo!()
    }

    pub async fn insert_job(&self, job: &Job) -> Result<i64, sqlx::Error> {
        let mut tx: Transaction<'_, Postgres> = self.database.begin().await?;
        // if ANY OF THESE FAIL, WE ROLL BACK :)

        let company_id =
            self.insert_company_info(&job.company_info, &mut tx).await?;
        // let contact_id =
        //     self.insert_contact_info(&job.contact_info, &mut tx).await?;
        // let contact_id =
        //     self.insert_contact_info(&job.contact_info, &mut tx).await?;
        let job_info = &job.job_info;

        let job_id: i64 = sqlx::query_scalar(
            "INSERT INTO job_info
        (title,description,job_url,company_id,contact_id)
        VALUES ($1,$2,$3,$4,$5) RETURNING job_id",
        )
        // .bind(&job_info.title)
        // .bind(&job_info.description)
        // .bind(&job_info.job_url)
        // .bind(&company_id)
        .fetch_one(&mut *tx)
        .await?;

        let tag_ids =
            self.insert_job_relations(todo!(), job_id, &mut tx).await?;

        tx.commit().await?;
        Ok(job_id)

        // self.insert_job_urls(job_id, &job.job_urls).await;
    }

    async fn insert_contact_info(
        &self,
        contact_info: &ContactInfo,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<i64, sqlx::Error> {
        let contact_id: i64 = sqlx::query_scalar(
            "INSERT INTO company_info
        (name, phone_number,email)
        VALUES ($1,$2,$3)
        ON CONFLICT (name) DO NOTHING
        RETURNING contact_id",
        )
        .bind(&contact_info.name)
        .bind(&contact_info.phone_number.0)
        .bind(&contact_info.email)
        .fetch_one(&mut *(*tx))
        .await?;
        Ok(contact_id)
    }

    async fn insert_company_info(
        &self,
        company_info: &CompanyInfo,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<i64, sqlx::Error> {
        let company_id: i64 = sqlx::query_scalar(
            "INSERT INTO company_info
        (name, address)
        VALUES ($1,$2)
        ON CONFLICT (name) DO NOTHING
        RETURNING company_id",
        )
        .bind(&company_info.name)
        .fetch_one(&mut *(*tx))
        .await?;
        Ok(company_id)
    }
    async fn push_stream_values<'args, I, F, DB: Database>(
        query_builder: &'_ mut QueryBuilder<'args, DB>,
        tuples: I,
        mut push_tuple: F,
    ) where
        I: StreamExt,
        F: FnMut(&mut Separated<'_, 'args, DB, &'static str>, I::Item),
    {
        query_builder.push("VALUES ");

        let mut tuples = pin!(tuples);
        let mut is_first = true;
        while let Some(tuple) = tuples.next().await {
            if is_first {
                is_first = false;
                query_builder.push("(");
            } else {
                query_builder.push(", (");
            }
            let mut separated = query_builder.separated(", ");

            push_tuple(&mut separated, tuple);

            query_builder.push(")");
        }
    }

    async fn insert_job_relations(
        &self,
        job_tags: &[JobTag],
        job_id: i64,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<i64>, sqlx::Error> {
        let tags: Vec<&str> = job_tags.iter().map(|tag| tag.name).collect();

        let ids: Vec<i64> = sqlx::query_scalar(
            r#"--sql
    WITH
      new_tags AS (
        INSERT INTO job_tags (name)
        SELECT unnest($1::text[])
        ON CONFLICT (name) DO NOTHING
        RETURNING id
      ),
        all_tags AS (
            SELECT id FROM new_tags
            UNION ALL
            SELECT id FROM job_tags
            WHERE name = ANY($1::text[])
        ),
      inserted AS (
        INSERT INTO job_tag_relations (job_id, tag_id)
        SELECT $2, id
          FROM all_tags
        ON CONFLICT DO NOTHING
      )
    SELECT id FROM all_tags
    "#,
        )
        .bind(&tags)
        .bind(&job_id)
        .fetch_all(&mut *(*tx))
        .await?;

        Ok(ids)
    }
}
