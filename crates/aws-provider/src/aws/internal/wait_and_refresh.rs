use std::{any::Any, future::Future, pin::Pin, time::Duration};

use tokio::time::{Instant, sleep, timeout};

use crate::aws::AWSClient;
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WaitError {
    #[error(
        "Timeout error after {timeout:?}. Last known state: {last_state}. Expected states: {expected_states:?}"
    )]
    Timeout {
        last_state: String,
        timeout: Duration,
        expected_states: Vec<String>,
    },
    #[error("Resource not found after {retries} retries")]
    NotFound { retries: u32 },
    #[error("Unexpected state: {current_state}. Expected states: {expected_states:?}")]
    UnexpectedState {
        current_state: String,
        expected_states: Vec<String>,
    },
    #[error("Error refreshing resource state: {0}")]
    RefreshError(String),
}
pub type RefreshFunctionReturn =
    Pin<Box<dyn Future<Output = Result<Option<(Box<dyn Any>, Vec<String>)>, String>>>>;
pub type RefreshFunction = Box<dyn Fn(AWSClient, String) -> RefreshFunctionReturn>;
/**
 * Configuration for waiting on a resource to reach a desired state.
 * target_state: The desired state to wait for.
 * pending_state: The intermediate state indicating the resource is still in transition.
 * refresh_fn: A function that refreshes the resource state. It takes an AWS EC2 client and a resource identifier,
 * and returns a Future that resolves to the current state of the resource.
 */
pub struct StateChangeConfig {
    pub target_state: Vec<String>,
    pub pending_state: Vec<String>,
    pub refresh_fn: RefreshFunction,
    pub initial_delay: Duration, // Initial delay before starting the refresh attempts
    pub timeout: Duration,       // Maximum time to wait for the desired state
    pub min_delay: Duration,     // Minimum delay between refresh attempts
    pub max_delay: Duration, // Maximum delay between refresh attempts, Used for exponential backoff
    pub not_found_checks: u32, // Number of consecutive not found checks before giving up
}

impl StateChangeConfig {
    pub fn new(
        target_state: Vec<String>,
        pending_state: Vec<String>,
        refresh_fn: RefreshFunction,
        start_delay: Option<Duration>,
        timeout: Option<Duration>,
        min_delay: Option<Duration>,
        max_delay: Option<Duration>,
        not_found_checks: Option<u32>,
    ) -> Self {
        StateChangeConfig {
            target_state,
            pending_state,
            refresh_fn,
            initial_delay: start_delay.unwrap_or(Duration::from_secs(0)), // default 0 seconds
            timeout: timeout.unwrap_or(Duration::from_secs(300)),         // default 5 minutes
            min_delay: min_delay.unwrap_or(Duration::from_secs(5)),
            // default 5 seconds
            max_delay: max_delay.unwrap_or(Duration::from_secs(60)), // default 1 minute
            not_found_checks: not_found_checks.unwrap_or(20),        // default 20 checks
        }
    }

    pub async fn wait_until_state(
        &self,
        client: AWSClient,
        resource_id: String,
    ) -> Result<Option<Box<dyn Any>>, WaitError> {
        let start_time = Instant::now(); // Track the start time for timeout calculation
        let mut not_found_count = 0u32;
        let mut current_delay = self.min_delay;
        let mut last_state = String::new();
        let mut last_resource: Option<Box<dyn Any>>;
        let mut i: u32 = 0;

        // Initial delay
        if self.initial_delay > Duration::from_secs(0) {
            sleep(self.initial_delay).await;
        }

        loop {
            tracing::info!(
                "Waiting for EC2 instance creation, elapsed: {:?}",
                start_time.elapsed()
            );
            println!(
                "Waiting for EC2 instance creation, elapsed: {:?}",
                start_time.elapsed()
            );
            i += 1;
            // Check for timeout
            if start_time.elapsed() >= self.timeout {
                return Err(WaitError::Timeout {
                    last_state,
                    timeout: self.timeout,
                    expected_states: self.target_state.iter().map(|s| s.to_string()).collect(),
                });
            }

            // Refresh the resource state with timeout
            let remaining_time = self.timeout.saturating_sub(start_time.elapsed());
            let refresh_result = timeout(
                remaining_time.min(Duration::from_secs(30)), // Cap individual refresh timeout
                (self.refresh_fn)(client.clone(), resource_id.clone()),
            )
            .await
            .unwrap_or(Ok(None));

            let (resource, current_state) = match refresh_result {
                Ok(Some((res, state))) => (Some(res), state),
                Ok(None) => (None, vec![]),
                Err(_) => return Err(WaitError::RefreshError("Refresh timeout".to_string())),
            };

            last_state = current_state
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n");

            // Handle case where resource is not found
            if resource.is_none() {
                // If we're waiting for the absence of a thing, return success
                if self.target_state.is_empty() {
                    return Ok(None);
                }

                // Resource not found, increment counter
                not_found_count += 1;
                if not_found_count > self.not_found_checks {
                    return Err(WaitError::NotFound {
                        retries: not_found_count,
                    });
                }
            } else {
                // Resource found, reset not found counter
                not_found_count = 0;
                last_resource = resource;

                // Check if current state is a target state
                if self.target_state == current_state {
                    return Ok(last_resource);
                }

                // Check if current state is a pending state
                if self.pending_state != current_state && !self.pending_state.is_empty() {
                    return Err(WaitError::UnexpectedState {
                        current_state: current_state
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                            .join("\n"),
                        expected_states: self
                            .target_state
                            .iter()
                            .chain(self.pending_state.iter())
                            .map(|s| s.to_string())
                            .collect(),
                    });
                }
            }

            // Sleep with exponential backoff
            sleep(current_delay).await;
            // Increase the delay once for every 3 iterations
            if i % 3 == 2 {
                current_delay = (current_delay * 2).min(self.max_delay);
            }
        }
    }
}
