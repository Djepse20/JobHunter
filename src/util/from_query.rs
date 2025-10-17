use futures::StreamExt;

pub trait CreateQuery<From>
where
    Self: Sized,
{
    type Error;
    type Item;
    type Output<S>;
    async fn create_query(
        &self,
        val: From,
    ) -> Result<Self::Output<impl StreamExt<Item = Self::Item>>, Self::Error>;
}
