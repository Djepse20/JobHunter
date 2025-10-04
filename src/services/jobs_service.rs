use std::{marker::PhantomData, mem::MaybeUninit};

use futures::{
    StreamExt,
    stream::{self, FuturesUnordered},
};
use serde::{Deserialize, Serialize};

use crate::services::database_service::database::DataBase;
use crate::{
    Job_query::job_queries::JobFetcher,
    services::database_service::dbtypes::Job,
    util::{
        equality::{IsEqualityOp, RecEqChecker, TupleLength},
        options::FetchOptions,
    },
};
pub struct Jobs<DB = (), T = ()> {
    database: DB,
    job_fetchers: T,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            database: (),
            job_fetchers: (),
        }
    }
}
impl<T> Jobs<(), T> {
    pub fn add_database(self, database: DataBase) -> Jobs<DataBase, T> {
        Jobs {
            database: database,
            job_fetchers: self.job_fetchers,
        }
    }
}

impl<DB> Jobs<DB, ()> {
    pub fn add_fetchers<const N: usize, T: JobFetcher>(
        self,
        fetchers: [T; N],
    ) -> Jobs<DB, [T; N]> {
        Jobs {
            database: self.database,

            job_fetchers: fetchers,
        }
    }
}
#[macro_export]

macro_rules! tuple_list_type {
    () => ( () );

    ($i:ty)  => ( ($i, ()) );
    ($i:ty,) => ( ($i, ()) );
    ($i:ty, $($e:ty),*)  => ( ($i, $crate::tuple_list_type!($($e),*)) );
    ($i:ty, $($e:ty),*,) => ( ($i, $crate::tuple_list_type!($($e),*)) );
}

#[macro_export]
macro_rules! fetchers {
    ($($variant:ident : $fetcher:ty = $constructor:expr  ),* $(,)?) => {
        {
          enum Fetchers {
            $($variant($fetcher)),*
          }
          use crate::services::database_service::dbtypes::Job;
          use crate::services::database_service::database::DataBase;
          use crate::services::jobs_service::FetcherBuilder;
          use crate::tuple_list_type;
          use crate::util::equality::TupleLength;
          use crate::services::jobs_service::AddFetcher;
          impl JobFetcher for Fetchers {
            async fn fetch_all_jobs_with_options_and_db<'a>(
                &'a self,
                options: &'a FetchOptions,
                database: Option<&'a DataBase>,
            ) -> Option<Vec<Job>> {
                match self {
                    $(Fetchers::$variant(inner) => inner.fetch_all_jobs_with_options_and_db(options,database).await,)*
                    _ => {todo!()}
                }
            }

          }
          const LEN : usize =<tuple_list_type!($($fetcher),*)>::LENGTH;


          FetcherBuilder::<LEN,(),Fetchers>::new()
          $(.add_fetcher(Fetchers::$variant($constructor )))*
          .build_array()
        }
    };
}

impl<const N: usize, T: JobFetcher> Jobs<DataBase, [T; N]> {
    async fn fetch_jobs(&self, options: FetchOptions) -> Option<Vec<Job>> {
        let jobs_from_sites = self
            .job_fetchers
            .fetch_all_jobs_with_options_and_db(&options, Some(&self.database))
            .await;
        todo!();
    }
}

pub trait AddFetcher<const N: usize, J: JobFetcher, U, Markers> {
    type OutPut;
    fn add_fetcher(self, fetcher: J) -> FetcherBuilder<N, Self::OutPut, U>;
}

pub struct FetcherBuilder<const N: usize, T, U> {
    jobs: [MaybeUninit<U>; N],
    fetchers: PhantomData<T>,
}

impl<const N: usize, T, U> FetcherBuilder<N, T, U> {
    pub fn new() -> FetcherBuilder<N, T, U> {
        Self {
            jobs: [const { MaybeUninit::uninit() }; N],
            fetchers: PhantomData,
        }
    }
}

impl<const N: usize, U, T1, T2, J: JobFetcher, Mode, U1, U2, Eq1: IsEqualityOp>
    AddFetcher<N, J, U, (Mode, U1, U2, Eq1)> for FetcherBuilder<N, (T1, T2), U>
where
    (T1, T2): RecEqChecker<J, Mode, U1, U2, Eq1> + TupleLength,
    J: Into<U>,
{
    type OutPut = (J, (T1, T2));
    fn add_fetcher(
        mut self,
        fetcher: J,
    ) -> FetcherBuilder<N, (J, (T1, T2)), U> {
        const {
            if <(J, (T1, T2))>::LENGTH > N {
                panic!("Too big")
            }
        }
        self.jobs[<(J, (T1, T2))>::LENGTH - 1].write(fetcher.into());
        FetcherBuilder {
            jobs: self.jobs,
            fetchers: PhantomData,
        }
    }
}

impl<const N: usize, U, J: JobFetcher> AddFetcher<N, J, U, ()>
    for FetcherBuilder<N, (), U>
where
    J: Into<U>,
    (J, ()): TupleLength,
{
    type OutPut = (J, ());
    fn add_fetcher(mut self, fetcher: J) -> FetcherBuilder<N, (J, ()), U> {
        const {
            if <(J, ())>::LENGTH > N {
                panic!("Too big")
            }
        }
        self.jobs[0].write(fetcher.into());
        FetcherBuilder {
            jobs: self.jobs,
            fetchers: PhantomData,
        }
    }
}

impl<const N: usize, T, U> FetcherBuilder<N, T, U>
where
    T: TupleLength,
{
    pub fn build_array(self) -> [U; N] {
        const {
            if T::LENGTH != N {
                panic!("T::LENGTH is not equal to N")
            }
        }
        unsafe { self.jobs.as_ptr().cast::<[U; N]>().read() }
    }
}
