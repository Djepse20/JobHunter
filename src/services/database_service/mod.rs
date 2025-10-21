pub mod database;
pub mod types;
use sqlx::{Executor, Postgres};

trait DbType {
    type Executor<'a>: Executor<'a, Database = Postgres>;
}
pub trait DbGet: Sized {
    type GetType<'a>;
    async fn get<'a, E: Executor<'a, Database = Postgres>>(
        executor: E,
        get: Self::GetType<'a>,
    ) -> Result<Self, sqlx::Error>;
}

pub trait DbInsert: Sized {
    type InsertType<'a>;

    async fn insert<'a, E: Executor<'a, Database = Postgres>>(
        executor: E,
        value: Self::InsertType<'a>,
    ) -> Result<Self, sqlx::Error>;
}
pub trait DbDelete: Sized {
    type DeleteType<'a>;
    type RetType;
    async fn delete<'a, E: Executor<'a, Database = Postgres>>(
        executor: E,
        delete: Self::DeleteType<'a>,
    ) -> Result<Self::RetType, sqlx::Error>;
}
