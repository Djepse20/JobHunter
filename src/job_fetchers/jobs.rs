use crate::{
    job_fetchers::{
        de::preview::DeserializableJob,
        job_index::preview::{JobPreview, JobPreviews},
    },
    services::database_service::{database::DataBase, types::Job},
};
use futures::StreamExt;

pub async fn get_all_unique_job<'c, T, J>(
    database: &DataBase,
    (offset, mut job_pages): (
        usize,
        impl StreamExt<Item = (usize, String)> + Unpin,
    ),
) -> Option<impl StreamExt<Item = JobPreview<J>>>
where
    T: JobPreviews<J>,
{
    let newest_job = database.get_newest_job().await.ok()?;

    let (jobs_to_take, html) = job_pages.next().await?;

    let mut newer_jobs: Vec<JobPreview<J>> = Vec::new();

    Some(job_pages.flat_map(|(_, html)| {
        T::unique_previews((&html, jobs_to_take, 0), &newest_job)
    }))
}
