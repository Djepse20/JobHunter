use crate::{
    job_fetchers::job_previewer::{JobConstants, JobIntermediate},
    services::database_service::{database::DataBase, types::Job},
};
use futures::StreamExt;

pub async fn unique_jobs<'b, 'c, T: JobConstants>(
    (json, jobs_to_take, offset): (&'c str, usize, usize),
    newest_job: &'b Job,
) -> Option<Vec<JobIntermediate<'c, T>>> {
    let jobs_iter = serde_json::Deserializer::from_str(json)
        .into_iter_seq::<JobIntermediate<T>>()
        .skip(offset)
        .take(jobs_to_take)
        .take_while(|job| match job {
            Ok(job) => newest_job.created_at > job.date,
            Err(_) => false,
        })
        .flatten();

    Some(jobs_iter.collect())
}

pub async fn get_all_unique_job<T: JobConstants>(
    database: &DataBase,
    (offset, mut job_pages): (
        usize,
        impl StreamExt<Item = (usize, String)> + Unpin,
    ),
) -> Option<()> {
    let newest_job = database.get_newest_job().await.ok()?;

    let (jobs_to_take, html) = job_pages.next().await?;

    let mut newer_jobs: Vec<JobIntermediate<T>> = Vec::new();

    if let Some(mut new_jobs) =
        unique_jobs((&html, jobs_to_take, offset), &newest_job).await
    {
        newer_jobs.append(&mut new_jobs);
    }

    while let Some((_, html)) = job_pages.next().await {
        if let Some(mut new_jobs) =
            unique_jobs::<T>((&html, jobs_to_take, 0), &newest_job).await
        {
            // newer_jobs.append(&mut new_jobs);
        }
    }
    None
}
