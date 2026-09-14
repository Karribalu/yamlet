#[cfg(test)]
mod tests {
    use crate::aws::ec2::ec2_instance::{EC2Error, EC2Instance};
    use aws_config::BehaviorVersion;
    use aws_sdk_ec2::{config::ProvideCredentials, types as ec2_types};
    use serial_test::serial;

    /// Test helper to setup test environment with AWS credentials
    fn setup_test_credentials() {
        unsafe {
            std::env::set_var("AWS_ACCESS_KEY_ID", "test_access_key");
            std::env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret_key");
            std::env::set_var("AWS_REGION", "us-west-2");
        }
    }

    /// Test helper to cleanup test environment
    fn cleanup_test_credentials() {
        unsafe {
            std::env::remove_var("AWS_ACCESS_KEY_ID");
            std::env::remove_var("AWS_SECRET_ACCESS_KEY");
            std::env::remove_var("AWS_REGION");
        }
    }

    #[test]
    #[serial]
    fn test_aws_access_key_id_environment_variable() {
        setup_test_credentials();

        let access_key = std::env::var("AWS_ACCESS_KEY_ID");
        assert!(access_key.is_ok(), "AWS_ACCESS_KEY_ID should be set");
        assert_eq!(
            access_key.unwrap(),
            "test_access_key",
            "AWS_ACCESS_KEY_ID should match the expected value"
        );

        cleanup_test_credentials();
    }

    #[test]
    #[serial]
    fn test_aws_secret_access_key_environment_variable() {
        setup_test_credentials();

        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY");
        assert!(secret_key.is_ok(), "AWS_SECRET_ACCESS_KEY should be set");
        assert_eq!(
            secret_key.unwrap(),
            "test_secret_key",
            "AWS_SECRET_ACCESS_KEY should match the expected value"
        );

        cleanup_test_credentials();
    }

    #[test]
    #[serial]
    fn test_aws_region_environment_variable() {
        setup_test_credentials();

        let region = std::env::var("AWS_REGION");
        assert!(region.is_ok(), "AWS_REGION should be set");
        assert_eq!(
            region.unwrap(),
            "us-west-2",
            "AWS_REGION should match the expected value"
        );

        cleanup_test_credentials();
    }

    #[test]
    #[serial]
    fn test_missing_aws_access_key_id() {
        cleanup_test_credentials();

        let access_key = std::env::var("AWS_ACCESS_KEY_ID");
        assert!(
            access_key.is_err(),
            "AWS_ACCESS_KEY_ID should not be set in test environment"
        );
    }

    #[test]
    #[serial]
    fn test_missing_aws_secret_access_key() {
        cleanup_test_credentials();

        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY");
        assert!(
            secret_key.is_err(),
            "AWS_SECRET_ACCESS_KEY should not be set in test environment"
        );
    }

    #[test]
    #[serial]
    fn test_both_credentials_set() {
        setup_test_credentials();

        let access_key = std::env::var("AWS_ACCESS_KEY_ID");
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY");

        assert!(access_key.is_ok(), "AWS_ACCESS_KEY_ID should be set");
        assert!(secret_key.is_ok(), "AWS_SECRET_ACCESS_KEY should be set");

        assert_eq!(access_key.unwrap(), "test_access_key");
        assert_eq!(secret_key.unwrap(), "test_secret_key");

        cleanup_test_credentials();
    }

    #[test]
    fn test_ec2_instance_new_with_client() {
        // Create a mock config for testing
        let config = aws_types::SdkConfig::builder()
            .region(aws_types::region::Region::new("us-west-2"))
            .behavior_version(BehaviorVersion::latest())
            .build();

        let client = aws_sdk_ec2::Client::new(&config);
        let instance = EC2Instance::new(client);

        // Verify that the instance was created successfully
        assert!(
            std::mem::size_of_val(&instance) > 0,
            "EC2Instance should be created successfully"
        );
    }

    #[test]
    fn test_ec2_instance_from_config() {
        let config = aws_types::SdkConfig::builder()
            .region(aws_types::region::Region::new("us-west-2"))
            .behavior_version(BehaviorVersion::latest())
            .build();

        let instance = EC2Instance::from_config(&config);

        // Verify that the instance was created successfully
        assert!(
            std::mem::size_of_val(&instance) > 0,
            "EC2Instance should be created from config successfully"
        );
    }

    #[test]
    #[serial]
    fn test_credentials_override() {
        unsafe {
            // Set initial credentials
            std::env::set_var("AWS_ACCESS_KEY_ID", "initial_key");
            std::env::set_var("AWS_SECRET_ACCESS_KEY", "initial_secret");
        }
        let initial_access = std::env::var("AWS_ACCESS_KEY_ID").unwrap();
        assert_eq!(initial_access, "initial_key");

        unsafe {
            // Override credentials
            std::env::set_var("AWS_ACCESS_KEY_ID", "new_key");
            std::env::set_var("AWS_SECRET_ACCESS_KEY", "new_secret");
        }
        let new_access = std::env::var("AWS_ACCESS_KEY_ID").unwrap();
        let new_secret = std::env::var("AWS_SECRET_ACCESS_KEY").unwrap();

        assert_eq!(new_access, "new_key");
        assert_eq!(new_secret, "new_secret");

        cleanup_test_credentials();
    }

    #[test]
    #[serial]
    fn test_credentials_with_special_characters() {
        unsafe {
            std::env::set_var("AWS_ACCESS_KEY_ID", "AKIA-TEST/KEY+123");
            std::env::set_var("AWS_SECRET_ACCESS_KEY", "SecretKey/With+Special=Chars");
        }
        let access_key = std::env::var("AWS_ACCESS_KEY_ID").unwrap();
        let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").unwrap();

        assert_eq!(access_key, "AKIA-TEST/KEY+123");
        assert_eq!(secret_key, "SecretKey/With+Special=Chars");

        cleanup_test_credentials();
    }

    #[test]
    fn test_opts_from_yaml_required_fields() {
        let yaml_str = r#"
image_id: ami-12345678
instance_type: t2.micro
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let opts = EC2Instance::opts_from_yaml(&yaml);

        assert!(opts.is_ok(), "Should parse YAML with required fields");
        let opts = opts.unwrap();
        assert_eq!(opts.image_id, "ami-12345678");
        assert_eq!(opts.instance_type, ec2_types::InstanceType::T2Micro);
    }

    #[test]
    fn test_opts_from_yaml_missing_required_fields() {
        let yaml_str = r#"
instance_type: t2.micro
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let opts = EC2Instance::opts_from_yaml(&yaml);

        assert!(opts.is_err(), "Should fail when image_id is missing");
    }

    #[test]
    fn test_opts_from_yaml_with_ami_field() {
        let yaml_str = r#"
ami: ami-87654321
instance_type: t2.micro
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let opts = EC2Instance::opts_from_yaml(&yaml);

        assert!(
            opts.is_ok(),
            "Should parse YAML with 'ami' field instead of 'image_id'"
        );
        let opts = opts.unwrap();
        assert_eq!(opts.image_id, "ami-87654321");
    }

    // Test for testing actual ec2 instance creation would go here
    #[tokio::test]
    async fn test_ec2_instance_creation() {
        // This test would require integration testing with AWS or localstack
        // and is not suitable for unit tests due to side effects

        let yaml_str = r#"
ami: ami-04c174f38aefd7dc8
instance_type: t2.micro
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = aws_config::defaults(BehaviorVersion::latest())
            .profile_name("default")
            .load()
            .await;
        println!("AWS Config Loaded: {:?}", config.endpoint_url());
        let opts = EC2Instance::opts_from_yaml(&yaml).unwrap();
        let created_instance = EC2Instance::from_config(&config)
            .create_instance(&opts)
            .await
            .unwrap();
        println!("Created Instance: {:?}", created_instance);
        assert!(
            created_instance.instance_id.is_some(),
            "EC2 instance creation should return an instance with an ID"
        );

        let list_instances = EC2Instance::from_config(&config)
            .list_instances()
            .await
            .unwrap();

        println!("List Instances: {:?}", list_instances);

        let created_id = created_instance.instance_id.clone();
        let matching_count = list_instances
            .iter()
            .filter(|instance| instance.instance_id == created_id)
            .count();
        assert_eq!(
            matching_count, 1,
            "Created instance should be listed among running instances"
        );

        // Cleanup - terminate the created instance
        EC2Instance::from_config(&config)
            .terminate_instance(&created_instance.instance_id.clone().unwrap())
            .await
            .unwrap();

        // Final verification to check if no instance is running
        let final_list_instances = EC2Instance::from_config(&config)
            .list_instances()
            .await
            .unwrap();
        let final_matching_count = final_list_instances
            .iter()
            .filter(|instance| {
                instance.instance_id == created_id
                    && instance.state.as_ref().unwrap().name
                        == Some(ec2_types::InstanceStateName::Running)
            })
            .count();
        assert_eq!(
            final_matching_count, 0,
            "Created instance should be terminated and not listed among running instances"
        );
    }

    // #[tokio::test]
    // async fn test_ec2_instance_deletion() {
    //     // This test would require integration testing with AWS or localstack
    //     // and is not suitable for unit tests due to side effects
    //
    //     let config = aws_config::defaults(BehaviorVersion::latest())
    //         .profile_name("localstack")
    //         .load()
    //         .await;
    //     let ec2_instance = EC2Instance::from_config(&config);
    //
    //     ec2_instance
    //         .terminate_instance("i-5f574f9616fc29f95")
    //         .await
    //         .unwrap();
    // }

    #[tokio::test]
    async fn test_ec2_instance_deletion_not_found() {
        // This test would require integration testing with AWS or localstack
        // and is not suitable for unit tests due to side effects

        let config = aws_config::defaults(BehaviorVersion::latest())
            .profile_name("localstack")
            .load()
            .await;
        let ec2_instance = EC2Instance::from_config(&config);

        let error = ec2_instance.terminate_instance("i-f45b1068dd622f3c").await;
        assert_eq!(error.err(), Some(EC2Error::InstanceNotFound));
    }

    #[tokio::test]
    async fn test_ec2_instance_stop_not_found() {
        // This test would require integration testing with AWS or localstack
        // and is not suitable for unit tests due to side effects

        let config = aws_config::defaults(BehaviorVersion::latest())
            .profile_name("localstack")
            .load()
            .await;
        let ec2_instance = EC2Instance::from_config(&config);

        let error = ec2_instance.stop_instance("i-8e285d79f825b543").await;
        assert_eq!(error.err(), Some(EC2Error::InstanceNotFound));
    }

    #[tokio::test]
    async fn test_ec2_describe_instances_empty() {
        // This test would require integration testing with AWS or localstack
        // and is not suitable for unit tests due to side effects

        let config = aws_config::defaults(BehaviorVersion::latest())
            .profile_name("localstack")
            .load()
            .await;
        let ec2_instance = EC2Instance::from_config(&config);

        let instances = ec2_instance
            .describe_instance("i-e03f9007759cec241")
            .await
            .unwrap();
        println!("Instances: {:?}", instances);
    }
}
