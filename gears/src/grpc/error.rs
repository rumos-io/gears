use tonic::Status;

use crate::baseapp::errors::QueryError;

impl From<QueryError> for Status {
    fn from(err: QueryError) -> Self {
        match err {
            QueryError::Store(_) => {
                // The store can return a version not found error, however gRPC queries do not supply a version. Instead
                // we always query the latests version. Therefore, something has gone badly wrong if we get this error.
                Status::internal("An internal error occurred while querying the application state.")
            }
            _ => Status::invalid_argument("Invalid message."), // TODO: Don't forget to add more info later
        }
    }
}
