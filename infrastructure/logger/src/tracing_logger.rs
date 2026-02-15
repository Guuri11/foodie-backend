use business::domain::logger::Logger;
use tracing::{debug, error, info, warn};

pub struct TracingLogger;

impl Logger for TracingLogger {
    fn info(&self, message: &str) {
        info!(target: "Backend -- ", "{}", message);
    }
    fn warn(&self, message: &str) {
        warn!(target: "Backend -- ", "{}", message);
    }
    fn error(&self, message: &str) {
        error!(target: "Backend -- ", "{}", message);
    }
    fn debug(&self, message: &str) {
        debug!(target: "Backend -- ", "{}", message);
    }
}
