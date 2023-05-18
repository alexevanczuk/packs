use error_stack::{Context, IntoReport, Report, Result, ResultExt};

/// Extension trait to shorten repretitive calls, `into_report().change_context(NewError) => `into_context(NewError)`
pub trait IntoContext: Sized {
    /// Type of the [`Ok`] value in the [`Result`]
    type Ok;

    /// Type of the resulting [`Err`] variant wrapped inside a [`Report<E>`].
    type Err;

    /// Converts the [`Err`] variant of the [`Result`] to a [`Report<C>`]
    fn into_context<C>(self, context: C) -> Result<Self::Ok, C>
    where
        C: Context;
}

impl<T, E> IntoContext for core::result::Result<T, E>
where
    Report<E>: From<E>,
{
    type Err = E;
    type Ok = T;

    #[track_caller]
    fn into_context<C>(self, context: C) -> Result<T, C>
    where
        C: Context,
    {
        self.into_report().change_context(context)
    }
}
