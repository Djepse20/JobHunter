use std::pin::pin;

use crate::job_fetchers::preview::JobPreview;
use crate::services::database_service::DbDelete;
use crate::services::database_service::types::{
    CompanyInfo, ContactInfo, Job, JobInfo, JobTag, JobUrl, Location,
};
use futures::StreamExt;
use sqlx::Executor;
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

    pub async fn get_jobs<T>(
        &self,
        job_url: JobUrl,
    ) -> Result<i64, sqlx::Error> {
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
    pub async fn delete_jobs(
        &self,
        jobs: &[JobUrl],
    ) -> Result<(), sqlx::Error> {
        let mut tx: Transaction<'_, Postgres> = self.database.begin().await?;
        Job::delete(&mut *tx, jobs).await?;
        tx.commit().await?;
        Ok(())
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
}

mod fetch {
    use super::*;
    // impl DataBase {
    //     pub async fn get_job_by_job_url(
    //         &self,
    //         job_url: JobUrl,
    //     ) -> Result<Job, sqlx::Error> {
    //         let mut tx: Transaction<'_, Postgres> =
    //             self.database.begin().await?;

    //         let job = sqlx::query!(
    //             "SELECT *
    //         FROM job INNER JOIN company ON
    //         job.company_id = company.id AND job.job_url= $1",
    //             job_url.0
    //         )
    //         .fetch_one(&mut *tx)
    //         .await?;
    //         tx.commit().await?;
    //         Ok(())
    //     }
    // }
    impl DataBase {}
}

mod insert {
    use sqlx::{Acquire, PgConnection};

    use super::*;
    use crate::services::database_service::database::DataBase;
    impl DataBase {
        pub async fn insert_job(&self, job: &Job) -> Result<i64, sqlx::Error> {
            let mut tx: Transaction<'_, Postgres> =
                self.database.begin().await?;
            // if ANY OF THESE FAIL, WE ROLL BACK :)

            //COMPANY
            let company_id =
                Self::insert_company_with_executor(&job.company_info, &mut tx)
                    .await?;
            //JOB
            let job_id = Self::insert_job_with_executor(
                &job.job_info,
                company_id,
                &mut *tx,
            )
            .await?;
            //JOB TAGS
            let job_tag_ids = Self::insert_job_tags_with_executor(
                &job.job_info.job_tags,
                &mut *tx,
            )
            .await?;

            Self::insert_job_tag_relations_with_executor(
                job_tag_ids,
                job_id,
                &mut *tx,
            )
            .await?;

            // JOB LOCATIONS
            let location_ids =
                Self::insert_job_locations(&job.locations, &mut *tx).await?;

            Self::insert_job_location_relations_with_executor(
                location_ids,
                job_id,
                &mut *tx,
            )
            .await?;

            Ok(job_id)
        }
    }
    impl DataBase {
        async fn insert_company_with_executor<'a>(
            company_info: &'a CompanyInfo,
            executor: &'a mut PgConnection,
        ) -> Result<i64, sqlx::Error> {
            let company_id = sqlx::query!(
                r#"--sql
            INSERT INTO company (name, logo_url)
            VALUES ($1, $2)
            ON CONFLICT (name)
            DO NOTHING
            RETURNING company.id
            "#,
                company_info.name,
                company_info.logo_url,
            )
            .fetch_one(&mut *executor)
            .await?
            .id;
            Ok(company_id)
        }

        async fn insert_job_with_executor<'a>(
            job_info: &'a JobInfo,
            company_id: i64,
            executor: &'a mut PgConnection,
        ) -> Result<i64, sqlx::Error> {
            let job_id = sqlx::query!(
                r#"--sql
            INSERT INTO job (title, description,job_url,company_id)
            VALUES ($1, $2,$3,$4)
            RETURNING id
            "#,
                job_info.title.0,
                job_info.description.0,
                job_info.job_url.0,
                company_id,
            )
            .fetch_one(&mut *executor)
            .await?
            .id;
            Ok(job_id)
        }

        async fn insert_job_tags_with_executor<'a>(
            job_tags: &'a [JobTag],
            executor: &'a mut PgConnection,
        ) -> Result<Vec<i64>, sqlx::Error> {
            let job_tag_names = job_tags
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
            .fetch_all(&mut *executor)
            .await?;
            Ok(job_tags.iter().map(|j| j.id).collect())
        }

        async fn insert_job_tag_relations_with_executor<'a>(
            job_tag_ids: Vec<i64>,
            job_id: i64,
            executor: &'a mut PgConnection,
        ) -> Result<(), sqlx::Error> {
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
            .execute(&mut *executor)
            .await?;
            Ok(())
        }

        async fn insert_job_locations<'a>(
            locations: &'a [Location],
            executor: &'a mut PgConnection,
        ) -> Result<Vec<i64>, sqlx::Error> {
            let (job_location_addresses, job_location_geo): (Vec<_>, Vec<_>) =
                locations
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
        .fetch_all(&mut* executor)
        .await?;

            let location_ids = locations
                .into_iter()
                .map(|loc| loc.id)
                .collect::<Vec<i64>>();

            Ok(location_ids)
        }

        async fn insert_job_location_relations_with_executor<'a>(
            location_ids: Vec<i64>,
            job_id: i64,
            executor: &'a mut PgConnection,
        ) -> Result<(), sqlx::Error> {
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
            .execute(&mut *executor)
            .await?;
            Ok(())
        }
    }
}
