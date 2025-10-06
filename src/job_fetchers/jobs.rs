use crate::{
    job_fetchers::{
        de::preview::DeserializableJob,
        job_index::preview::{JobPreview, JobPreviews},
    },
    services::database_service::{database::DataBase, types::Job},
};
use futures::StreamExt;

pub async fn get_all_unique_job<'c, J, T>(
    database: &DataBase,
    (offset, mut job_pages): (
        usize,
        impl StreamExt<Item = (usize, String)> + Unpin,
    ),
) -> Option<()>
where
    T: JobPreviews<J>,
{
    let newest_job = database.get_newest_job().await.ok()?;

    let (jobs_to_take, html) = job_pages.next().await?;

    let mut newer_jobs: Vec<JobPreview<J>> = Vec::new();

    if let Some(mut new_jobs) =
        T::unique_previews((&html, jobs_to_take, offset), &newest_job)
    {
        newer_jobs.append(&mut new_jobs);
    }

    while let Some((_, html)) = job_pages.next().await {
        if let Some(mut new_jobs) =
            T::unique_previews((&html, jobs_to_take, 0), &newest_job)
        {
            // newer_jobs.append(&mut new_jobs);
        }
    }
    None
}
