use crate::{
    job_fetchers::preview::{JobPreview, UniqueJobs},
    services::database_service::{database::DataBase, types::Job},
};
use futures::StreamExt;

pub async fn get_all_unique_job<T, J>(
    database: &DataBase,
    (offset, mut job_pages): (
        usize,
        impl StreamExt<Item = (usize, Vec<u8>)> + Unpin,
    ),
) -> Option<Vec<Job>>
where
    J: UniqueJobs,
    for<'a> JobPreview<'a, J>: TryInto<Job>,
{
    let newest_job = database.get_newest_job().await.ok()?;

    let mut newer_jobs: Vec<Job> = Vec::new();
    while let Some((jobs_to_take, html)) = job_pages.next().await {
        if let Some(jobs) =
            J::unique_jobs(&html, jobs_to_take, offset, &newest_job).map(|s| {
                s.filter_map(async |job| job.try_into().ok())
                    .then(async |s| s)
            })
        {
            newer_jobs.extend(jobs.collect::<Vec<Job>>().await);
        }
    }
    Some(newer_jobs)
}
