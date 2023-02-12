pub type BoxedErrorResult<T> = Result<T, Box<dyn std::error::Error>>;
