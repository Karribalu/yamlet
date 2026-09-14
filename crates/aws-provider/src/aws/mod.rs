pub mod credentials;
pub mod internal;
pub mod ec2;

#[derive(Debug, Clone)]
pub enum AWSClient {
    EC2Client(aws_sdk_ec2::Client),
}
