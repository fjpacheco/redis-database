mod redis_type;
mod simple_string;
mod bulk_string;
mod integer;
mod error;
mod array;

pub use error::ErrorStruct;
pub use error::RError;
pub use simple_string::RSimpleString;   
pub use bulk_string::RBulkString;   
pub use redis_type::RedisType;