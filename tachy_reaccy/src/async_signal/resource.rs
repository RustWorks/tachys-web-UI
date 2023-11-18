use super::{ArcAsyncDerived, AsyncDerivedFuture, AsyncState};
#[cfg(feature = "miniserde")]
use crate::serialization::Miniserde;
#[cfg(feature = "rkyv")]
use crate::serialization::Rkyv;
#[cfg(feature = "serde-lite")]
use crate::serialization::SerdeLite;
use crate::{
    arena::Owner,
    prelude::SignalWithUntracked,
    serialization::{SerdeJson, SerializableData, Serializer, Str},
};
use core::{fmt::Debug, marker::PhantomData};
use futures::Future;
use std::{future::IntoFuture, ops::Deref};

pub struct Resource<T, Ser = Str> {
    ser: PhantomData<Ser>,
    data: ArcAsyncDerived<T>,
}

impl<T, Ser> Deref for Resource<T, Ser> {
    type Target = ArcAsyncDerived<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Resource<T, Str>
where
    T: SerializableData<Str>,
    T::SerErr: Debug,
    T::DeErr: Debug,
{
    pub fn new<Fut>(fun: impl FnMut() -> Fut + Send + Sync + 'static) -> Self
    where
        T: Send + Sync + 'static,
        Fut: Future<Output = T> + Send + Sync + 'static,
    {
        Self::new_with_serializer(fun)
    }
}

impl<T> Resource<T, SerdeJson>
where
    T: SerializableData<SerdeJson>,
    T::SerErr: Debug,
    T::DeErr: Debug,
{
    pub fn serde<Fut>(fun: impl FnMut() -> Fut + Send + Sync + 'static) -> Self
    where
        T: Send + Sync + 'static,
        Fut: Future<Output = T> + Send + Sync + 'static,
    {
        Self::new_with_serializer(fun)
    }
}

#[cfg(feature = "miniserde")]
impl<T> Resource<T, Miniserde>
where
    T: SerializableData<Miniserde>,
    T::SerErr: Debug,
    T::DeErr: Debug,
{
    pub fn miniserde<Fut>(
        fun: impl FnMut() -> Fut + Send + Sync + 'static,
    ) -> Self
    where
        T: Send + Sync + 'static,
        Fut: Future<Output = T> + Send + Sync + 'static,
    {
        Self::new_with_serializer(fun)
    }
}

#[cfg(feature = "serde-lite")]
impl<T> Resource<T, SerdeLite>
where
    T: SerializableData<SerdeLite>,
    T::SerErr: Debug,
    T::DeErr: Debug,
{
    pub fn serde_lite<Fut>(
        fun: impl FnMut() -> Fut + Send + Sync + 'static,
    ) -> Self
    where
        T: Send + Sync + 'static,
        Fut: Future<Output = T> + Send + Sync + 'static,
    {
        Self::new_with_serializer(fun)
    }
}

#[cfg(feature = "rkyv")]
impl<T> Resource<T, Rkyv>
where
    T: SerializableData<Rkyv>,
    T::SerErr: Debug,
    T::DeErr: Debug,
{
    pub fn rkyv<Fut>(fun: impl FnMut() -> Fut + Send + Sync + 'static) -> Self
    where
        T: Send + Sync + 'static,
        Fut: Future<Output = T> + Send + Sync + 'static,
    {
        Self::new_with_serializer(fun)
    }
}

impl<T, Ser> Resource<T, Ser>
where
    Ser: Serializer,
    T: SerializableData<Ser>,
    T::SerErr: Debug,
    T::DeErr: Debug,
{
    pub fn new_with_serializer<Fut>(
        fun: impl FnMut() -> Fut + Send + Sync + 'static,
    ) -> Resource<T, Ser>
    where
        T: Send + Sync + 'static,
        Fut: Future<Output = T> + Send + Sync + 'static,
    {
        let initial = Self::initial_value();

        let data = ArcAsyncDerived::new_with_initial(initial, fun);

        if let Some(shared_context) = Owner::shared_context() {
            let value = data.clone();
            let ready_fut = data.ready();

            shared_context.write_async(Box::pin(async move {
                ready_fut.await;
                value
                    .with_untracked(|data| match &data {
                        AsyncState::Complete(val) => val.ser(),
                        _ => unreachable!(),
                    })
                    .unwrap() // TODO handle
            }));
        }

        Resource {
            ser: PhantomData,
            data,
        }
    }

    #[inline(always)]
    fn initial_value() -> AsyncState<T> {
        #[cfg(feature = "hydration")]
        {
            if let Some(shared_context) = Owner::shared_context() {
                let id = shared_context.next_id();
                let value = shared_context.read_data(id);
                if let Some(value) = value {
                    match T::de(&value) {
                        Ok(value) => AsyncState::Complete(value),
                        Err(e) => {
                            crate::log(&format!("couldn't deserialize: {e:?}"));
                            AsyncState::Loading
                        }
                    }
                } else {
                    AsyncState::Loading
                }
            } else {
                AsyncState::Loading
            }
        }
        // without hydration enabled, always starts Loading
        #[cfg(not(feature = "hydration"))]
        {
            AsyncState::Loading
        }
    }
}

impl<T, Ser> IntoFuture for Resource<T, Ser>
where
    T: Clone + 'static,
{
    type Output = T;
    type IntoFuture = AsyncDerivedFuture<T>;

    fn into_future(self) -> Self::IntoFuture {
        self.data.into_future()
    }
}