#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("resource not found")]
    ResourceNotFound,
    #[error("render pipieline not found")]
    RenderPipelineNotFound,
    #[error("pipieline not match")]
    PipelineNotMatch,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("kind: {0}")]
    Kind(#[from] ErrorKind),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
