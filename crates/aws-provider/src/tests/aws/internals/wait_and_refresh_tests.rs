#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use aws_config::BehaviorVersion;
    use aws_types::region::Region;

    use crate::aws::internal::wait_and_refresh::{RefreshFunction, StateChangeConfig, WaitError};
    use crate::aws::AWSClient;

    fn test_client() -> AWSClient {
        let sdk_config = aws_types::SdkConfig::builder()
            .region(Region::new("us-east-1"))
            .behavior_version(BehaviorVersion::latest())
            .build();

        AWSClient::EC2Client(aws_sdk_ec2::Client::new(&sdk_config))
    }

    #[tokio::test]
    async fn wait_until_state_returns_resource_once_target_state_reached() {
        let client = test_client();
        let call_counter = Arc::new(AtomicUsize::new(0));

        // Mocking the behavior of the refresh function
        let call_counter_clone = Arc::clone(&call_counter);
        let refresh_fn: RefreshFunction = Box::new(move |_client, _resource_id| {
            let call_counter = Arc::clone(&call_counter_clone);
            Box::pin(async move {
                let call = call_counter.fetch_add(1, Ordering::SeqCst);
                let state = if call < 2 {
                    vec![String::from("pending")]
                } else {
                    vec![String::from("running")]
                };
                Ok(Some((
                    Box::new(String::from("resource")) as Box<dyn Any>,
                    state,
                )))
            })
        });

        let config = StateChangeConfig::new(
            vec![String::from("running")],
            vec![String::from("pending")],
            refresh_fn,
            Some(Duration::from_millis(0)),
            Some(Duration::from_millis(200)),
            Some(Duration::from_millis(1)),
            Some(Duration::from_millis(1)),
            Some(10),
        );

        let resource = config
            .wait_until_state(client, "res-1234".to_string())
            .await
            .expect("should reach target state");

        let resource = resource
            .unwrap()
            .downcast::<String>()
            .expect("resource should downcast to String");

        assert_eq!(resource.as_str(), "resource");
        assert!(call_counter.load(Ordering::SeqCst) >= 3);
    }

    #[tokio::test]
    async fn wait_until_state_returns_not_found_after_threshold() {
        let client = test_client();

        let refresh_fn: RefreshFunction = Box::new(|_, _| Box::pin(async { Ok(None) }));

        let config = StateChangeConfig::new(
            vec![String::from("running")],
            vec![String::from("pending")],
            refresh_fn,
            Some(Duration::from_millis(0)),
            Some(Duration::from_millis(20)),
            Some(Duration::from_millis(1)),
            Some(Duration::from_millis(1)),
            Some(2),
        );

        let err = config
            .wait_until_state(client, "res-1234".to_string())
            .await
            .expect_err("should exceed not found threshold");

        match err {
            WaitError::NotFound { retries } => assert_eq!(retries, 3),
            other => panic!("expected NotFound error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn wait_until_state_returns_unexpected_state_for_invalid_transition() {
        let client = test_client();

        let refresh_fn: RefreshFunction = Box::new(|_, _| {
            Box::pin(async {
                Ok(Some((
                    Box::new(String::from("resource")) as Box<dyn Any>,
                    vec![String::from("failed")],
                )))
            })
        });

        let config = StateChangeConfig::new(
            vec![String::from("running")],
            vec![String::from("pending")],
            refresh_fn,
            Some(Duration::from_millis(0)),
            Some(Duration::from_millis(100)),
            Some(Duration::from_millis(1)),
            Some(Duration::from_millis(1)),
            Some(5),
        );

        let err = config
            .wait_until_state(client, "res-1234".to_string())
            .await
            .expect_err("should return unexpected state");

        match err {
            WaitError::UnexpectedState {
                current_state,
                expected_states,
            } => {
                assert_eq!(current_state, "failed");
                assert_eq!(
                    expected_states,
                    vec!["running".to_string(), "pending".to_string()]
                );
            }
            other => panic!("expected UnexpectedState error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn wait_until_state_returns_timeout_when_too_short() {
        let client = test_client();

        let refresh_fn: RefreshFunction = Box::new(|_, _| Box::pin(async { Ok(None) }));

        let config = StateChangeConfig::new(
            vec![String::from("running")],
            vec![String::from("pending")],
            refresh_fn,
            Some(Duration::from_millis(0)),
            Some(Duration::from_millis(0)),
            Some(Duration::from_millis(1)),
            Some(Duration::from_millis(1)),
            Some(5),
        );

        let err = config
            .wait_until_state(client, "res-1234".to_string())
            .await
            .expect_err("should time out immediately");

        match err {
            WaitError::Timeout { timeout, .. } => assert_eq!(timeout, Duration::from_millis(0)),
            other => panic!("expected Timeout error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn wait_until_state_succeeds_when_target_absent() {
        let client = test_client();

        let refresh_fn: RefreshFunction = Box::new(|_, _| Box::pin(async { Ok(None) }));

        let config = StateChangeConfig::new(
            Vec::<String>::new(),
            Vec::<String>::new(),
            refresh_fn,
            Some(Duration::from_millis(0)),
            Some(Duration::from_millis(100)),
            Some(Duration::from_millis(1)),
            Some(Duration::from_millis(1)),
            Some(5),
        );

        let resource = config
            .wait_until_state(client, "res-1234".to_string())
            .await
            .expect("should treat absence as success");

        assert!(resource.is_none());
    }

    #[tokio::test]
    async fn wait_until_state_maps_refresh_errors() {
        let client = test_client();

        let refresh_fn: RefreshFunction =
            Box::new(|_, _| Box::pin(async { Err("boom".to_string()) }));

        let config = StateChangeConfig::new(
            vec![String::from("running")],
            vec![String::from("pending")],
            refresh_fn,
            Some(Duration::from_millis(0)),
            Some(Duration::from_millis(100)),
            Some(Duration::from_millis(1)),
            Some(Duration::from_millis(1)),
            Some(5),
        );

        let err = config
            .wait_until_state(client, "res-1234".to_string())
            .await
            .expect_err("should propagate refresh error");

        match err {
            WaitError::RefreshError(message) => assert_eq!(message, "Refresh timeout"),
            other => panic!("expected RefreshError, got {:?}", other),
        }
    }
}
