use std::any::Any;

use aws_sdk_ec2::{error::ProvideErrorMetadata, types as ec2_types};
use tracing::info;

use crate::aws::{
    AWSClient,
    internal::wait_and_refresh::{RefreshFunctionReturn, StateChangeConfig, WaitError},
};

#[derive(Debug, Clone)]
pub struct InstanceOpts {
    block_device_mappings: Option<Vec<ec2_types::BlockDeviceMapping>>,
    capacity_reservation_specification: Option<ec2_types::CapacityReservationSpecification>,
    client_token: Option<String>,
    cpu_options: Option<ec2_types::CpuOptionsRequest>,
    credit_specification: Option<ec2_types::CreditSpecificationRequest>,
    disable_api_termination: Option<bool>,
    ebs_optimized: Option<bool>,
    enclave_options: Option<ec2_types::EnclaveOptionsRequest>,
    enable_primary_ipv6: Option<bool>,
    hibernation_options: Option<ec2_types::HibernationOptionsRequest>,
    iam_instance_profile: Option<ec2_types::IamInstanceProfileSpecification>,
    pub(crate) image_id: String,
    instance_initiated_shutdown_behavior: Option<String>,
    instance_market_options: Option<ec2_types::InstanceMarketOptionsRequest>,
    pub(crate) instance_type: ec2_types::InstanceType,
    ipv6_address_count: Option<i32>,
    ipv6_addresses: Option<Vec<ec2_types::InstanceIpv6Address>>,
    key_name: Option<String>,
    launch_template: Option<ec2_types::LaunchTemplateSpecification>,
    maintenance_options: Option<ec2_types::InstanceMaintenanceOptionsRequest>,
    max_count: i32,
    metadata_options: Option<ec2_types::InstanceMetadataOptionsRequest>,
    min_count: i32,
    monitoring: Option<ec2_types::RunInstancesMonitoringEnabled>,
    network_interfaces: Option<Vec<ec2_types::InstanceNetworkInterfaceSpecification>>,
    placement: Option<ec2_types::Placement>,
    private_dns_name_options: Option<ec2_types::PrivateDnsNameOptionsRequest>,
    private_ip_address: Option<String>,
    security_group_ids: Option<Vec<String>>,
    security_groups: Option<Vec<String>>,
    subnet_id: Option<String>,
    tag_specifications: Option<Vec<ec2_types::TagSpecification>>,
    user_data: Option<String>,
}

#[derive(Clone)]
pub struct EC2Instance {
    client: aws_sdk_ec2::Client,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum EC2Error {
    #[error("EC2 Instance not found")]
    InstanceNotFound,
    #[error("Error while creating EC2 instance")]
    InstanceNotCreated,
    #[error("Options error: {0}")]
    OptionsError(String),
    #[error("AWS SDK error: {0}")]
    SdkError(String),
    #[error("Error while waiting for state to confirm the resource change: {0}")]
    StateError(#[from] WaitError),
}

impl<T: ProvideErrorMetadata + std::fmt::Display> From<T> for EC2Error {
    fn from(value: T) -> Self {
        match value.code() {
            Some(code) if code == "InvalidInstanceID.NotFound" => EC2Error::InstanceNotFound,
            _ => {
                let error_message = format!(
                    "AWS SDK error: {} (code: {:?}, message: {:?})",
                    value,
                    value.code(),
                    value.message()
                );
                println!(
                    "Converted AWS SDK error to EC2Error: {} (code: {:?}, message: {:?})",
                    value,
                    value.code(),
                    value.message()
                );
                EC2Error::SdkError(error_message)
            }
        }
    }
}

impl EC2Instance {
    pub fn new(client: aws_sdk_ec2::Client) -> Self {
        EC2Instance { client }
    }

    pub fn from_config(config: &aws_types::SdkConfig) -> Self {
        let client = aws_sdk_ec2::Client::new(config);
        EC2Instance { client }
    }

    pub fn opts_from_yaml(yaml: &serde_yaml::Value) -> Result<InstanceOpts, EC2Error> {
        // Helper function to get string from yaml
        let get_string = |yaml: &serde_yaml::Value, key: &str| -> Option<String> {
            yaml.get(key)?.as_str().map(|s| s.to_string())
        };

        // Helper function to get bool from yaml
        let get_bool =
            |yaml: &serde_yaml::Value, key: &str| -> Option<bool> { yaml.get(key)?.as_bool() };

        // Helper function to get i32 from yaml
        let get_i32 = |yaml: &serde_yaml::Value, key: &str| -> Option<i32> {
            yaml.get(key)?.as_i64().map(|v| v as i32)
        };

        // Helper function to get string vec from yaml
        let get_string_vec = |yaml: &serde_yaml::Value, key: &str| -> Option<Vec<String>> {
            yaml.get(key)?
                .as_sequence()?
                .iter()
                .map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        };

        // Parse required fields
        let image_id = get_string(yaml, "image_id")
            .or_else(|| get_string(yaml, "ami"))
            .ok_or_else(|| {
                EC2Error::OptionsError("Missing required field: image_id or ami".to_string())
            })?;

        let instance_type_str = get_string(yaml, "instance_type").ok_or_else(|| {
            EC2Error::OptionsError("Missing required field: instance_type".to_string())
        })?;

        let instance_type = ec2_types::InstanceType::from(instance_type_str.as_str());

        let min_count = get_i32(yaml, "min_count").unwrap_or(1);
        let max_count = get_i32(yaml, "max_count").unwrap_or(1);

        // Parse optional fields
        let key_name = get_string(yaml, "key_name");
        let subnet_id = get_string(yaml, "subnet_id");
        let private_ip_address = get_string(yaml, "private_ip_address");
        let user_data = get_string(yaml, "user_data");
        let client_token = get_string(yaml, "client_token");
        let disable_api_termination = get_bool(yaml, "disable_api_termination");
        let ebs_optimized = get_bool(yaml, "ebs_optimized");
        let enable_primary_ipv6 = get_bool(yaml, "enable_primary_ipv6");
        let ipv6_address_count = get_i32(yaml, "ipv6_address_count");
        let instance_initiated_shutdown_behavior =
            get_string(yaml, "instance_initiated_shutdown_behavior");
        let security_group_ids = get_string_vec(yaml, "security_group_ids");
        let security_groups = get_string_vec(yaml, "security_groups");

        // Parse nested optional structures
        let monitoring = yaml.get("monitoring").and_then(|m| {
            let enabled = m.get("enabled")?.as_bool()?;
            Some(
                ec2_types::RunInstancesMonitoringEnabled::builder()
                    .enabled(enabled)
                    .build(),
            )
        });

        let iam_instance_profile = yaml.get("iam_instance_profile").and_then(|iip| {
            let mut builder = ec2_types::IamInstanceProfileSpecification::builder();
            if let Some(arn) = get_string(iip, "arn") {
                builder = builder.arn(arn);
            }
            if let Some(name) = get_string(iip, "name") {
                builder = builder.name(name);
            }
            Some(builder.build())
        });

        let placement = yaml.get("placement").and_then(|p| {
            let mut builder = ec2_types::Placement::builder();
            if let Some(availability_zone) = get_string(p, "availability_zone") {
                builder = builder.availability_zone(availability_zone);
            }
            if let Some(affinity) = get_string(p, "affinity") {
                builder = builder.affinity(affinity);
            }
            if let Some(group_name) = get_string(p, "group_name") {
                builder = builder.group_name(group_name);
            }
            if let Some(host_id) = get_string(p, "host_id") {
                builder = builder.host_id(host_id);
            }
            if let Some(tenancy) = get_string(p, "tenancy") {
                builder = builder.tenancy(ec2_types::Tenancy::from(tenancy.as_str()));
            }
            if let Some(partition_number) = get_i32(p, "partition_number") {
                builder = builder.partition_number(partition_number);
            }
            Some(builder.build())
        });

        let cpu_options = yaml.get("cpu_options").and_then(|co| {
            let mut builder = ec2_types::CpuOptionsRequest::builder();
            if let Some(core_count) = get_i32(co, "core_count") {
                builder = builder.core_count(core_count);
            }
            if let Some(threads_per_core) = get_i32(co, "threads_per_core") {
                builder = builder.threads_per_core(threads_per_core);
            }
            Some(builder.build())
        });

        let credit_specification = yaml.get("credit_specification").and_then(|cs| {
            let cpu_credits = get_string(cs, "cpu_credits")?;
            Some(
                ec2_types::CreditSpecificationRequest::builder()
                    .cpu_credits(cpu_credits)
                    .build(),
            )
        });

        let hibernation_options = yaml.get("hibernation_options").and_then(|ho| {
            let configured = get_bool(ho, "configured")?;
            Some(
                ec2_types::HibernationOptionsRequest::builder()
                    .configured(configured)
                    .build(),
            )
        });

        let enclave_options = yaml.get("enclave_options").and_then(|eo| {
            get_bool(eo, "enabled").map(|enabled| {
                ec2_types::EnclaveOptionsRequest::builder()
                    .enabled(enabled)
                    .build()
            })
        });

        let metadata_options = yaml.get("metadata_options").and_then(|mo| {
            let mut builder = ec2_types::InstanceMetadataOptionsRequest::builder();
            if let Some(http_tokens) = get_string(mo, "http_tokens") {
                builder =
                    builder.http_tokens(ec2_types::HttpTokensState::from(http_tokens.as_str()));
            }
            if let Some(http_put_response_hop_limit) = get_i32(mo, "http_put_response_hop_limit") {
                builder = builder.http_put_response_hop_limit(http_put_response_hop_limit);
            }
            if let Some(http_endpoint) = get_string(mo, "http_endpoint") {
                builder = builder.http_endpoint(ec2_types::InstanceMetadataEndpointState::from(
                    http_endpoint.as_str(),
                ));
            }
            Some(builder.build())
        });

        let private_dns_name_options = yaml.get("private_dns_name_options").and_then(|pdno| {
            let mut builder = ec2_types::PrivateDnsNameOptionsRequest::builder();
            if let Some(hostname_type) = get_string(pdno, "hostname_type") {
                builder =
                    builder.hostname_type(ec2_types::HostnameType::from(hostname_type.as_str()));
            }
            if let Some(enable_resource_name_dns_a_record) =
                get_bool(pdno, "enable_resource_name_dns_a_record")
            {
                builder =
                    builder.enable_resource_name_dns_a_record(enable_resource_name_dns_a_record);
            }
            if let Some(enable_resource_name_dns_aaaa_record) =
                get_bool(pdno, "enable_resource_name_dns_aaaa_record")
            {
                builder = builder
                    .enable_resource_name_dns_aaaa_record(enable_resource_name_dns_aaaa_record);
            }
            Some(builder.build())
        });

        // Parse tag specifications
        let tag_specifications = yaml.get("tag_specifications").and_then(|ts| {
            ts.as_sequence().map(|specs| {
                specs
                    .iter()
                    .filter_map(|spec| {
                        let resource_type = get_string(spec, "resource_type")?;
                        let tags_yaml = spec.get("tags")?;

                        let tags: Vec<ec2_types::Tag> = if let Some(tags_map) =
                            tags_yaml.as_mapping()
                        {
                            tags_map
                                .iter()
                                .filter_map(|(k, v)| {
                                    let key = k.as_str()?.to_string();
                                    let value = v.as_str()?.to_string();
                                    Some(ec2_types::Tag::builder().key(key).value(value).build())
                                })
                                .collect()
                        } else {
                            vec![]
                        };

                        Some(
                            ec2_types::TagSpecification::builder()
                                .resource_type(ec2_types::ResourceType::from(
                                    resource_type.as_str(),
                                ))
                                .set_tags(Some(tags))
                                .build(),
                        )
                    })
                    .collect()
            })
        });

        // Parse block device mappings
        let block_device_mappings = yaml.get("block_device_mappings").and_then(|bdm| {
            bdm.as_sequence().map(|devices| {
                devices
                    .iter()
                    .filter_map(|device| {
                        let device_name = get_string(device, "device_name")?;
                        let mut builder =
                            ec2_types::BlockDeviceMapping::builder().device_name(device_name);

                        if let Some(ebs) = device.get("ebs") {
                            let mut ebs_builder = ec2_types::EbsBlockDevice::builder();

                            if let Some(volume_size) = get_i32(ebs, "volume_size") {
                                ebs_builder = ebs_builder.volume_size(volume_size);
                            }
                            if let Some(volume_type) = get_string(ebs, "volume_type") {
                                ebs_builder = ebs_builder
                                    .volume_type(ec2_types::VolumeType::from(volume_type.as_str()));
                            }
                            if let Some(delete_on_termination) =
                                get_bool(ebs, "delete_on_termination")
                            {
                                ebs_builder =
                                    ebs_builder.delete_on_termination(delete_on_termination);
                            }
                            if let Some(encrypted) = get_bool(ebs, "encrypted") {
                                ebs_builder = ebs_builder.encrypted(encrypted);
                            }
                            if let Some(iops) = get_i32(ebs, "iops") {
                                ebs_builder = ebs_builder.iops(iops);
                            }
                            if let Some(snapshot_id) = get_string(ebs, "snapshot_id") {
                                ebs_builder = ebs_builder.snapshot_id(snapshot_id);
                            }
                            if let Some(kms_key_id) = get_string(ebs, "kms_key_id") {
                                ebs_builder = ebs_builder.kms_key_id(kms_key_id);
                            }

                            builder = builder.ebs(ebs_builder.build());
                        }

                        if let Some(virtual_name) = get_string(device, "virtual_name") {
                            builder = builder.virtual_name(virtual_name);
                        }

                        if let Some(no_device) = get_string(device, "no_device") {
                            builder = builder.no_device(no_device);
                        }

                        Some(builder.build())
                    })
                    .collect()
            })
        });

        // Parse network interfaces
        let network_interfaces = yaml.get("network_interfaces").and_then(|ni| {
            ni.as_sequence().map(|interfaces| {
                interfaces
                    .iter()
                    .filter_map(|iface| {
                        let mut builder =
                            ec2_types::InstanceNetworkInterfaceSpecification::builder();

                        if let Some(device_index) = get_i32(iface, "device_index") {
                            builder = builder.device_index(device_index);
                        }
                        if let Some(subnet_id) = get_string(iface, "subnet_id") {
                            builder = builder.subnet_id(subnet_id);
                        }
                        if let Some(description) = get_string(iface, "description") {
                            builder = builder.description(description);
                        }
                        if let Some(private_ip_address) = get_string(iface, "private_ip_address") {
                            builder = builder.private_ip_address(private_ip_address);
                        }
                        if let Some(groups) = get_string_vec(iface, "groups") {
                            builder = builder.set_groups(Some(groups));
                        }
                        if let Some(delete_on_termination) =
                            get_bool(iface, "delete_on_termination")
                        {
                            builder = builder.delete_on_termination(delete_on_termination);
                        }
                        if let Some(associate_public_ip_address) =
                            get_bool(iface, "associate_public_ip_address")
                        {
                            builder =
                                builder.associate_public_ip_address(associate_public_ip_address);
                        }

                        Some(builder.build())
                    })
                    .collect()
            })
        });

        // Parse IPv6 addresses
        let ipv6_addresses = yaml.get("ipv6_addresses").and_then(|ipv6| {
            ipv6.as_sequence().map(|addresses| {
                addresses
                    .iter()
                    .filter_map(|addr| {
                        let ipv6_address = addr.as_str()?.to_string();
                        Some(
                            ec2_types::InstanceIpv6Address::builder()
                                .ipv6_address(ipv6_address)
                                .build(),
                        )
                    })
                    .collect()
            })
        });

        let opts = InstanceOpts {
            block_device_mappings,
            capacity_reservation_specification: None, // Complex nested structure
            client_token,
            cpu_options,
            credit_specification,
            disable_api_termination,
            ebs_optimized,
            enclave_options,
            enable_primary_ipv6,
            hibernation_options,
            iam_instance_profile,
            image_id,
            instance_initiated_shutdown_behavior,
            instance_market_options: None, // Complex nested structure
            instance_type,
            ipv6_address_count,
            ipv6_addresses,
            key_name,
            launch_template: None,     // Complex nested structure
            maintenance_options: None, // Complex nested structure
            max_count,
            metadata_options,
            min_count,
            monitoring,
            network_interfaces,
            placement,
            private_dns_name_options,
            private_ip_address,
            security_group_ids,
            security_groups,
            subnet_id,
            tag_specifications,
            user_data,
        };

        Ok(opts)
    }

    pub async fn start_instance(&self, instance_id: &str) -> Result<(), EC2Error> {
        self.client
            .start_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }

    pub async fn stop_instance(&self, instance_id: &str) -> Result<(), EC2Error> {
        self.client
            .stop_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }

    pub async fn describe_instance(
        &self,
        instance_id: &str,
    ) -> Result<aws_sdk_ec2::types::Instance, EC2Error> {
        let resp = self
            .client
            .describe_instances()
            .instance_ids(instance_id)
            .send()
            .await?;

        if let Some(reservations) = resp.reservations {
            if let Some(instances) = reservations
                .into_iter()
                .flat_map(|r| r.instances)
                .flat_map(|f| f)
                .next()
            {
                return Ok(instances);
            }
        }

        Err(EC2Error::InstanceNotFound)
    }

    pub async fn terminate_instance(&self, instance_id: &str) -> Result<(), EC2Error> {
        self.client
            .terminate_instances()
            .instance_ids(instance_id)
            .send()
            .await?;
        Ok(())
    }

    pub async fn list_instances(&self) -> Result<Vec<aws_sdk_ec2::types::Instance>, EC2Error> {
        let resp = self.client.describe_instances().send().await?;
        let mut instances = Vec::new();

        if let Some(reservations) = resp.reservations {
            for reservation in reservations {
                if let Some(res) = reservation.instances {
                    instances.extend(res);
                }
            }
        }

        Ok(instances)
    }

    /**
     * Creates an EC2 instance with the given configuration options.
     * This will wait until the instance is in the 'running' state before returning.
     */
    pub async fn create_instance(
        &self,
        config: &InstanceOpts,
    ) -> Result<aws_sdk_ec2::types::Instance, EC2Error> {
        tracing::info!("Creating EC2 instance with config: {:?}", config);
        println!("Creating EC2 instance with config: {:?}", config);
        let config_clone = config.clone();
        let mut request = self
            .client
            .run_instances()
            .image_id(config_clone.image_id)
            .instance_type(config_clone.instance_type)
            .min_count(config_clone.min_count)
            .max_count(config_clone.max_count);

        // Add optional parameters
        if let Some(block_device_mappings) = config_clone.block_device_mappings {
            request = request.set_block_device_mappings(Some(block_device_mappings));
        }
        if let Some(capacity_reservation_specification) =
            config_clone.capacity_reservation_specification
        {
            request =
                request.capacity_reservation_specification(capacity_reservation_specification);
        }
        if let Some(client_token) = config_clone.client_token {
            request = request.client_token(client_token);
        }
        if let Some(cpu_options) = config_clone.cpu_options {
            request = request.cpu_options(cpu_options);
        }
        if let Some(credit_specification) = config_clone.credit_specification {
            request = request.credit_specification(credit_specification);
        }
        if let Some(disable_api_termination) = config_clone.disable_api_termination {
            request = request.disable_api_termination(disable_api_termination);
        }
        if let Some(ebs_optimized) = config_clone.ebs_optimized {
            request = request.ebs_optimized(ebs_optimized);
        }
        if let Some(enclave_options) = config_clone.enclave_options {
            request = request.enclave_options(enclave_options);
        }
        if let Some(enable_primary_ipv6) = config_clone.enable_primary_ipv6 {
            request = request.enable_primary_ipv6(enable_primary_ipv6);
        }
        if let Some(hibernation_options) = config_clone.hibernation_options {
            request = request.hibernation_options(hibernation_options);
        }
        if let Some(iam_instance_profile) = config_clone.iam_instance_profile {
            request = request.iam_instance_profile(iam_instance_profile);
        }
        if let Some(instance_initiated_shutdown_behavior) =
            config_clone.instance_initiated_shutdown_behavior
        {
            request = request.instance_initiated_shutdown_behavior(
                ec2_types::ShutdownBehavior::from(instance_initiated_shutdown_behavior.as_str()),
            );
        }
        if let Some(instance_market_options) = config_clone.instance_market_options {
            request = request.instance_market_options(instance_market_options);
        }
        if let Some(ipv6_address_count) = config_clone.ipv6_address_count {
            request = request.ipv6_address_count(ipv6_address_count);
        }
        if let Some(ipv6_addresses) = config_clone.ipv6_addresses {
            request = request.set_ipv6_addresses(Some(ipv6_addresses));
        }
        if let Some(key_name) = config_clone.key_name {
            request = request.key_name(key_name);
        }
        if let Some(launch_template) = config_clone.launch_template {
            request = request.launch_template(launch_template);
        }
        if let Some(maintenance_options) = config_clone.maintenance_options {
            request = request.maintenance_options(maintenance_options);
        }
        if let Some(metadata_options) = config_clone.metadata_options {
            request = request.metadata_options(metadata_options);
        }
        if let Some(monitoring) = config_clone.monitoring {
            request = request.monitoring(monitoring);
        }
        if let Some(network_interfaces) = config_clone.network_interfaces {
            request = request.set_network_interfaces(Some(network_interfaces));
        }
        if let Some(placement) = config_clone.placement {
            request = request.placement(placement);
        }
        if let Some(private_dns_name_options) = config_clone.private_dns_name_options {
            request = request.private_dns_name_options(private_dns_name_options);
        }
        if let Some(private_ip_address) = config_clone.private_ip_address {
            request = request.private_ip_address(private_ip_address);
        }
        if let Some(security_group_ids) = config_clone.security_group_ids {
            request = request.set_security_group_ids(Some(security_group_ids));
        }
        if let Some(security_groups) = config_clone.security_groups {
            request = request.set_security_groups(Some(security_groups));
        }
        if let Some(subnet_id) = config_clone.subnet_id {
            request = request.subnet_id(subnet_id);
        }
        if let Some(tag_specifications) = config_clone.tag_specifications {
            request = request.set_tag_specifications(Some(tag_specifications));
        }
        if let Some(user_data) = &config_clone.user_data {
            request = request.user_data(user_data);
        }
        info!("Creating EC2 instance with config: {:?}", &config);
        let resp = request.send().await?;

        let wait_state_config = StateChangeConfig::new(
            vec![ec2_types::InstanceStateName::Running.to_string()],
            vec![ec2_types::InstanceStateName::Pending.to_string()],
            Box::new(EC2Instance::wait_for_completion),
            None,
            None,
            None,
            None,
            None,
        );
        if let Some(instances) = resp.instances {
            let result = wait_state_config
                .wait_until_state(
                    AWSClient::EC2Client(self.client.clone()),
                    instances[0].instance_id.as_ref().unwrap().clone(),
                )
                .await?;
            if let Some(created_instances) = result {
                info!("EC2 instance created successfully: {:?}", created_instances);
                return Ok(*created_instances
                    .downcast::<aws_sdk_ec2::types::Instance>()
                    .unwrap());
            }
        }

        info!("EC2 instance creation failed: No instances returned");
        Err(EC2Error::InstanceNotCreated)
    }

    fn wait_for_completion(client: AWSClient, resource_id: String) -> RefreshFunctionReturn {
        Box::pin(async move {
            let ec2_client = match client {
                AWSClient::EC2Client(c) => c,
                _ => return Err("Invalid client type for EC2 instance".to_string()),
            };

            let resp = ec2_client
                .describe_instances()
                .instance_ids(resource_id.clone())
                .send()
                .await
                .map_err(|e| format!("Failed to describe instance: {}", e))?;

            if let Some(reservations) = resp.reservations {
                for reservation in reservations {
                    if let Some(instances) = reservation.instances {
                        if let Some(instance) = instances.into_iter().next() {
                            // Extract state before moving instance
                            let state = instance
                                .state
                                .as_ref()
                                .and_then(|s| s.name.as_ref())
                                .map(|n| n.as_str().to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            return Ok(Some((Box::new(instance) as Box<dyn Any>, vec![state])));
                        }
                    }
                }
            }

            Err("Instance not found".to_string())
        })
    }
}
