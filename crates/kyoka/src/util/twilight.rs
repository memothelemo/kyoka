/// Convenient macro for performing an HTTP request from
/// [`twilight_http`] crate but it returns [`error_stack::Result`].
#[macro_export]
macro_rules! perform_request {
    ($expr:expr, $($ctx:tt)*) => {{
        use ::error_stack::FutureExt;
        use ::futures::TryFutureExt;
        use ::std::future::IntoFuture;

        ($expr)
            .into_future()
            .change_context($($ctx)*)
            .and_then(|v| v.model().change_context($($ctx)*))
    }};
}
