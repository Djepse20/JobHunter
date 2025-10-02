use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::Job_query::{
    equality::{IsEqualityOp, RecEqChecker},
    job_queries::{Job, JobFetcher, database::DataBase, options::FetchOptions},
};

pub struct Jobs<T = ()> {
    database: Option<DataBase>,
    job_fetcher: T,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            database: None,
            job_fetcher: (),
        }
    }
}

pub trait Fetch {
    async fn fetch_jobs(&self, options: FetchOptions);
}

impl<T: JobFetcher> Fetch for Jobs<T> {
    async fn fetch_jobs(&self, options: FetchOptions) {
        self.job_fetcher
            .fetch_all_jobs_with_options_and_db(
                &options,
                self.database.as_ref(),
            )
            .await;
    }
}

impl Fetch for Jobs<()> {
    async fn fetch_jobs(&self, options: FetchOptions) {
        todo!()
    }
}

pub trait AddFetcher<J: JobFetcher, Markers> {
    type OutPut;
    fn add_fetcher(self, fetcher: J) -> Jobs<Self::OutPut>;
}

impl<T1, T2, J: JobFetcher, Mode, U1, U2, Eq1: IsEqualityOp>
    AddFetcher<J, (Mode, U1, U2, Eq1)> for Jobs<(T1, T2)>
where
    (T1, T2): RecEqChecker<J, Mode, U1, U2, Eq1>,
{
    type OutPut = (J, (T1, T2));

    fn add_fetcher(self, fetcher: J) -> Jobs<(J, (T1, T2))> {
        Jobs {
            database: self.database,
            job_fetcher: (fetcher, self.job_fetcher),
        }
    }
}

impl<J: JobFetcher> AddFetcher<J, ()> for Jobs {
    type OutPut = (J, ());
    fn add_fetcher(self, fetcher: J) -> Jobs<(J, ())> {
        Jobs {
            database: self.database,
            job_fetcher: (fetcher, ()),
        }
    }
}

impl<T1> Jobs<T1> {
    pub fn add_database(mut self, database: DataBase) -> Jobs<T1> {
        self.database = Some(database);
        self
    }
}
