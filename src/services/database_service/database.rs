use std::pin::pin;

use crate::job_fetchers::preview::JobPreview;
use crate::services::database_service::types::{
    CompanyInfo, ContactInfo, Job, JobTag, JobUrl,
};
use futures::StreamExt;
use sqlx::{
    Database, Postgres, QueryBuilder, Transaction, query_builder::Separated,
};

#[derive(Debug, Clone)]
pub struct DataBase {
    database: sqlx::PgPool,
}

impl Default for DataBase {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBase {
    pub fn new() -> Self {
        DataBase { database: todo!() }
    }
    pub async fn get_newest_job(&self) -> Result<Job, sqlx::Error> {
        todo!()
    }
    pub async fn insert_jobs<'a, T>(
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

    pub async fn get_jobs(&self, job_url: JobUrl) -> Result<i64, sqlx::Error> {
        let mut tx: Transaction<'_, Postgres> = self.database.begin().await?;

        let job = sqlx::query!(
            "SELECT job.*, company.name, company.logo_url
            FROM job INNER JOIN company ON 
            job.company_id = company.id AND job.job_url= $1",
            job_url.0
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        todo!()
    }
    pub async fn delete_job(&self, job: &[JobUrl]) -> Result<(), sqlx::Error> {
        let mut tx: Transaction<'_, Postgres> = self.database.begin().await?;
        let job_urls: Vec<String> =
            job.into_iter().map(|job| job.0.to_owned()).collect();
        let jobs = sqlx::query!(
            r#"--sql
            SELECT id FROM job
            WHERE job_url IN (
                SELECT UNNEST($1::text[])
            )
        "#,
            &job_urls
        )
        .fetch_all(&mut *tx)
        .await?;

        let job_ids: Vec<i64> = jobs.into_iter().map(|job| job.id).collect();
        let tags = sqlx::query!(
            r#"--sql
          DELETE FROM tags_for_job 
          WHERE job_id in (
            SELECT UNNEST($1::bigint[])
          )
          RETURNING job_tag_id
        "#,
            &job_ids
        )
        .fetch_all(&mut *tx)
        .await?;

        let locations = sqlx::query!(
            r#"--sql
          DELETE FROM location_for_job 
          WHERE job_id in (
            SELECT UNNEST($1::bigint[])
          )
          RETURNING location_id
        "#,
            &job_ids
        )
        .fetch_all(&mut *tx)
        .await?;

        let location_ids: Vec<i64> =
            locations.into_iter().map(|loc| loc.location_id).collect();
        let tag_ids: Vec<i64> =
            tags.into_iter().map(|tag| tag.job_tag_id).collect();
        //TODO - Implemet removal of tag ids.

        let jobs = sqlx::query!(
            r#"--sql
            WITH deleted_jobs AS (
            DELETE FROM job
            WHERE job_url IN (
                SELECT UNNEST($1::text[])
            )
            RETURNING company_id
            )
            SELECT DISTINCT company_id FROM deleted_jobs;
        "#,
            &job_urls
        )
        .fetch_all(&mut *tx)
        .await;
        // sqlx::query!(
        //     r#"--sql
        //     WITH deleted_jobs AS (
        //     DELETE FROM job
        //     WHERE job_url IN (
        //         SELECT UNNEST($1::text[])
        //     )
        //     RETURNING id,company_id
        //     )
        //     SELECT DISTINCT id ,company_id FROM deleted_jobs;
        // "#,
        //     &job_ids,
        //     &location_ids
        // )
        // .fetch_all(&mut *tx)
        // .await;

        tx.commit();
        Ok(())
    }
    pub async fn insert_job(&self, job: &Job) -> Result<i64, sqlx::Error> {
        let mut tx: Transaction<'_, Postgres> = self.database.begin().await?;
        // if ANY OF THESE FAIL, WE ROLL BACK :)

        //COMPANY
        let company_id = sqlx::query!(
            r#"--sql
            INSERT INTO company (name, logo_url)
            VALUES ($1, $2)
            ON CONFLICT (name)
            DO NOTHING
            RETURNING company.id
            "#,
            job.company_info.name,
            job.company_info.logo_url,
        )
        .fetch_one(&mut *tx)
        .await?
        .id;
        //JOB
        let job_id = sqlx::query!(
            r#"--sql
            INSERT INTO job (title, description,job_url,company_id)
            VALUES ($1, $2,$3,$4)
            RETURNING id
            "#,
            job.job_info.title.0,
            job.job_info.description.0,
            job.job_info.job_url.0,
            company_id,
        )
        .fetch_one(&mut *tx)
        .await?
        .id;

        //JOB TAGS
        let job_tag_names = job
            .job_info
            .job_tags
            .iter()
            .map(|job| job.name.to_owned())
            .collect::<Vec<String>>();
        let job_tags = sqlx::query!(
            r#"--sql
            INSERT INTO job_tag (tag)
            SELECT  UNNEST($1::varchar(255)[])
            RETURNING job_tag.id
            "#,
            &job_tag_names
        )
        .fetch_all(&mut *tx)
        .await?;

        let job_tag_ids =
            job_tags.into_iter().map(|j| j.id).collect::<Vec<i64>>();
        sqlx::query!(
            r#"--sql
            INSERT INTO tags_for_job (job_id,job_tag_id)
            SELECT  $1, UNNEST($2::bigint[])
            ON CONFLICT (job_id,job_tag_id)
            DO NOTHING
            "#,
            job_id,
            &job_tag_ids,
        )
        .execute(&mut *tx)
        .await?;

        let (job_location_addresses, job_location_geo): (Vec<_>, Vec<_>) = job
            .locations
            .iter()
            .map(|loc| (loc.address.to_owned(), loc.geo_location))
            .unzip();
        let (x, y): (Vec<_>, Vec<_>) = job_location_geo
            .into_iter()
            .map(|loc| (loc.0, loc.1))
            .collect();

        let locations = sqlx::query!(
            r#"--sql
            INSERT INTO job_location (address,x,y)
            SELECT * FROM UNNEST($1::varchar(255)[], $2::double precision[], $3::double precision[])            
            ON CONFLICT (x,y)
            DO NOTHING
            RETURNING id
            "#,
            &job_location_addresses,
            &x,
            &y
        )
        .fetch_all(&mut *tx)
        .await?;

        let location_ids = locations
            .into_iter()
            .map(|loc| loc.id)
            .collect::<Vec<i64>>();

        sqlx::query!(
            r#"--sql
            INSERT INTO location_for_job (job_id,location_id)
            SELECT $1, UNNEST($2::bigint[])         
            ON CONFLICT (job_id,location_id)
            DO NOTHING
            "#,
            job_id,
            &location_ids,
        )
        .execute(&mut *tx)
        .await?;

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
        .bind(job_id)
        .fetch_all(&mut *(*tx))
        .await?;

        Ok(ids)
    }
}
