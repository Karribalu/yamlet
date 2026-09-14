/// Module for handling AWS credentials from environment variables
use std::env;

#[derive(Debug, Clone)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: Option<String>,
    pub session_token: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialsError {
    #[error("Missing AWS_ACCESS_KEY_ID environment variable")]
    MissingAccessKeyId,
    #[error("Missing AWS_SECRET_ACCESS_KEY environment variable")]
    MissingSecretAccessKey,
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] env::VarError),
}

impl AwsCredentials {
    /// Load AWS credentials from environment variables
    ///
    /// This function reads the following environment variables:
    /// - AWS_ACCESS_KEY_ID (required)
    /// - AWS_SECRET_ACCESS_KEY (required)
    /// - AWS_REGION (optional)
    /// - AWS_SESSION_TOKEN (optional)
    pub fn from_env() -> Result<Self, CredentialsError> {
        let access_key_id =
            env::var("AWS_ACCESS_KEY_ID").map_err(|_| CredentialsError::MissingAccessKeyId)?;

        let secret_access_key = env::var("AWS_SECRET_ACCESS_KEY")
            .map_err(|_| CredentialsError::MissingSecretAccessKey)?;

        let region = env::var("AWS_REGION").ok();
        let session_token = env::var("AWS_SESSION_TOKEN").ok();

        Ok(Self {
            access_key_id,
            secret_access_key,
            region,
            session_token,
        })
    }

    /// Check if credentials are set in environment
    pub fn are_env_vars_set() -> bool {
        env::var("AWS_ACCESS_KEY_ID").is_ok() && env::var("AWS_SECRET_ACCESS_KEY").is_ok()
    }

    /// Get the AWS region from environment or use default
    pub fn get_region_or_default(default: &str) -> String {
        env::var("AWS_REGION")
            .or_else(|_| env::var("AWS_DEFAULT_REGION"))
            .unwrap_or_else(|_| default.to_string())
    }

    /// Validate that credentials are not empty
    pub fn validate(&self) -> Result<(), String> {
        if self.access_key_id.is_empty() {
            return Err("AWS_ACCESS_KEY_ID cannot be empty".to_string());
        }
        if self.secret_access_key.is_empty() {
            return Err("AWS_SECRET_ACCESS_KEY cannot be empty".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn setup_full_credentials() {
        unsafe {
            env::set_var("AWS_ACCESS_KEY_ID", "AKIAIOSFODNN7EXAMPLE");
            env::set_var(
                "AWS_SECRET_ACCESS_KEY",
                "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            );
            env::set_var("AWS_REGION", "us-west-2");
            env::set_var("AWS_SESSION_TOKEN", "session_token_example");
        }
    }

    fn setup_minimal_credentials() {
        unsafe {
            env::set_var("AWS_ACCESS_KEY_ID", "test_access_key");
            env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret_key");
        }
    }

    fn cleanup_credentials() {
        unsafe {
            env::remove_var("AWS_ACCESS_KEY_ID");
            env::remove_var("AWS_SECRET_ACCESS_KEY");
            env::remove_var("AWS_REGION");
            env::remove_var("AWS_SESSION_TOKEN");
            env::remove_var("AWS_DEFAULT_REGION");
        }
    }

    #[test]
    #[serial]
    fn test_from_env_with_all_credentials() {
        setup_full_credentials();

        let credentials = AwsCredentials::from_env();
        assert!(credentials.is_ok(), "Should load credentials successfully");

        let creds = credentials.unwrap();
        assert_eq!(creds.access_key_id, "AKIAIOSFODNN7EXAMPLE");
        assert_eq!(
            creds.secret_access_key,
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
        );
        assert_eq!(creds.region, Some("us-west-2".to_string()));
        assert_eq!(
            creds.session_token,
            Some("session_token_example".to_string())
        );

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_from_env_with_minimal_credentials() {
        setup_minimal_credentials();

        let credentials = AwsCredentials::from_env();
        assert!(
            credentials.is_ok(),
            "Should load minimal credentials successfully"
        );

        let creds = credentials.unwrap();
        assert_eq!(creds.access_key_id, "test_access_key");
        assert_eq!(creds.secret_access_key, "test_secret_key");
        assert_eq!(creds.region, None);
        assert_eq!(creds.session_token, None);

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_from_env_missing_access_key() {
        cleanup_credentials();
        unsafe {
            env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret");
        }

        let credentials = AwsCredentials::from_env();
        assert!(
            credentials.is_err(),
            "Should fail when access key is missing"
        );

        match credentials {
            Err(CredentialsError::MissingAccessKeyId) => (),
            _ => panic!("Expected MissingAccessKeyId error"),
        }

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_from_env_missing_secret_key() {
        cleanup_credentials();
        unsafe {
            env::set_var("AWS_ACCESS_KEY_ID", "test_access");
        }

        let credentials = AwsCredentials::from_env();
        assert!(
            credentials.is_err(),
            "Should fail when secret key is missing"
        );

        match credentials {
            Err(CredentialsError::MissingSecretAccessKey) => (),
            _ => panic!("Expected MissingSecretAccessKey error"),
        }

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_from_env_missing_both_keys() {
        cleanup_credentials();

        let credentials = AwsCredentials::from_env();
        assert!(
            credentials.is_err(),
            "Should fail when both keys are missing"
        );

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_are_env_vars_set_true() {
        setup_minimal_credentials();

        assert!(
            AwsCredentials::are_env_vars_set(),
            "Should return true when credentials are set"
        );

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_are_env_vars_set_false() {
        cleanup_credentials();

        assert!(
            !AwsCredentials::are_env_vars_set(),
            "Should return false when credentials are not set"
        );
    }

    #[test]
    #[serial]
    fn test_are_env_vars_set_partial() {
        cleanup_credentials();
        unsafe {
            env::set_var("AWS_ACCESS_KEY_ID", "test_key");
        }

        assert!(
            !AwsCredentials::are_env_vars_set(),
            "Should return false when only one credential is set"
        );

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_get_region_or_default_with_aws_region() {
        cleanup_credentials();
        unsafe {
            env::set_var("AWS_REGION", "eu-west-1");
        }

        let region = AwsCredentials::get_region_or_default("us-east-1");
        assert_eq!(region, "eu-west-1", "Should return AWS_REGION when set");

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_get_region_or_default_with_aws_default_region() {
        cleanup_credentials();
        unsafe {
            env::set_var("AWS_DEFAULT_REGION", "ap-south-1");
        }

        let region = AwsCredentials::get_region_or_default("us-east-1");
        assert_eq!(
            region, "ap-south-1",
            "Should return AWS_DEFAULT_REGION when AWS_REGION is not set"
        );

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_get_region_or_default_fallback() {
        cleanup_credentials();

        let region = AwsCredentials::get_region_or_default("us-east-1");
        assert_eq!(
            region, "us-east-1",
            "Should return default when no region env vars are set"
        );
    }

    #[test]
    #[serial]
    fn test_get_region_or_default_prefers_aws_region() {
        cleanup_credentials();
        unsafe {
            env::set_var("AWS_REGION", "eu-west-1");
            env::set_var("AWS_DEFAULT_REGION", "us-west-2");
        }

        let region = AwsCredentials::get_region_or_default("us-east-1");
        assert_eq!(
            region, "eu-west-1",
            "Should prefer AWS_REGION over AWS_DEFAULT_REGION"
        );

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_validate_valid_credentials() {
        setup_minimal_credentials();

        let credentials = AwsCredentials::from_env().unwrap();
        let result = credentials.validate();

        assert!(result.is_ok(), "Valid credentials should pass validation");

        cleanup_credentials();
    }

    #[test]
    fn test_validate_empty_access_key() {
        let credentials = AwsCredentials {
            access_key_id: String::new(),
            secret_access_key: "secret".to_string(),
            region: None,
            session_token: None,
        };

        let result = credentials.validate();
        assert!(result.is_err(), "Empty access key should fail validation");
        assert_eq!(result.unwrap_err(), "AWS_ACCESS_KEY_ID cannot be empty");
    }

    #[test]
    fn test_validate_empty_secret_key() {
        let credentials = AwsCredentials {
            access_key_id: "access".to_string(),
            secret_access_key: String::new(),
            region: None,
            session_token: None,
        };

        let result = credentials.validate();
        assert!(result.is_err(), "Empty secret key should fail validation");
        assert_eq!(result.unwrap_err(), "AWS_SECRET_ACCESS_KEY cannot be empty");
    }

    #[test]
    #[serial]
    fn test_credentials_with_spaces() {
        cleanup_credentials();
        unsafe {
            env::set_var("AWS_ACCESS_KEY_ID", "  AKIAIOSFODNN7EXAMPLE  ");
            env::set_var("AWS_SECRET_ACCESS_KEY", "  secret_with_spaces  ");
        }

        let credentials = AwsCredentials::from_env().unwrap();
        assert_eq!(credentials.access_key_id, "  AKIAIOSFODNN7EXAMPLE  ");
        assert_eq!(credentials.secret_access_key, "  secret_with_spaces  ");

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_credentials_clone() {
        setup_minimal_credentials();

        let credentials = AwsCredentials::from_env().unwrap();
        let cloned = credentials.clone();

        assert_eq!(credentials.access_key_id, cloned.access_key_id);
        assert_eq!(credentials.secret_access_key, cloned.secret_access_key);

        cleanup_credentials();
    }

    #[test]
    fn test_credentials_debug_format() {
        let credentials = AwsCredentials {
            access_key_id: "test_access".to_string(),
            secret_access_key: "test_secret".to_string(),
            region: Some("us-west-2".to_string()),
            session_token: None,
        };

        let debug_str = format!("{:?}", credentials);
        assert!(debug_str.contains("test_access"));
        assert!(debug_str.contains("us-west-2"));
    }

    #[test]
    #[serial]
    fn test_session_token_handling() {
        cleanup_credentials();
        unsafe {
            env::set_var("AWS_ACCESS_KEY_ID", "access");
            env::set_var("AWS_SECRET_ACCESS_KEY", "secret");
            env::set_var("AWS_SESSION_TOKEN", "temporary_session_token_12345");
        }

        let credentials = AwsCredentials::from_env().unwrap();
        assert!(credentials.session_token.is_some());
        assert_eq!(
            credentials.session_token.unwrap(),
            "temporary_session_token_12345"
        );

        cleanup_credentials();
    }

    #[test]
    #[serial]
    fn test_multiple_sequential_loads() {
        // First load
        setup_minimal_credentials();
        let creds1 = AwsCredentials::from_env().unwrap();
        assert_eq!(creds1.access_key_id, "test_access_key");

        // Change credentials
        unsafe {
            env::set_var("AWS_ACCESS_KEY_ID", "new_access_key");
            env::set_var("AWS_SECRET_ACCESS_KEY", "new_secret_key");
        }

        // Second load
        let creds2 = AwsCredentials::from_env().unwrap();
        assert_eq!(creds2.access_key_id, "new_access_key");
        assert_ne!(creds1.access_key_id, creds2.access_key_id);

        cleanup_credentials();
    }
}
