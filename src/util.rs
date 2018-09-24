use chrono::{
    DateTime,
    Utc,
};

////////////////////////////////////////////////////////////////////////////////

pub fn time_now() -> String {
    Utc::now().to_rfc3339()
}
