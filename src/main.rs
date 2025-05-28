mod service;

use tonic::transport::Server;
use tonic_reflection::pb::v1::FILE_DESCRIPTOR_SET;
use crate::afaina::afaina_service_server::AfainaServiceServer;
use crate::service::afaina_service::Afaina;

pub mod afaina{
    tonic::include_proto!("afaina");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("chat_descriptor");
}

#[tokio::main]
async fn main() ->Result<(),Box<dyn std::error::Error>> {

    let afaina = Afaina::new("afaina".to_string());

    let addr = "0.0.0.0:5067".parse()?;
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1()?;
    Server::builder()
        .add_service(service)
        .add_service(AfainaServiceServer::new(afaina))
        .serve(addr)
        .await?;
    println!("\n user afaina service is up and running at port {}",addr);
    Ok(())
}
