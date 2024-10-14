use std::{collections::HashMap, future::Future, net::TcpListener};

use axum::http::StatusCode;
use bollard::{
    container::{Config, CreateContainerOptions, StartContainerOptions},
    secret::{HostConfig, PortBinding},
    Docker,
};

use crate::{
    routes::create_container::ContainerInfo,
    utils::{
        db::{self},
        res::{m_resp, CReturn},
        resources::ContainerResources,
    },
};

use super::res::Respond;

pub fn create_config(
    ports: HashMap<u16, u16>,
    resources: ContainerResources,
    image: impl Into<String>,
) -> Config<String> {
    let mut port_bindings = HashMap::new();
    for port in &ports {
        port_bindings.insert(
            format!("{}/tcp", port.0).to_string(),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(port.1.to_string()),
            }]),
        );
    }
    let exposed_ports: HashMap<String, HashMap<(), ()>> = ports
        .keys()
        .map(|exposed| (format!("{}/tcp", exposed), HashMap::new()))
        .collect();
    Config {
        image: Some(image.into()),
        exposed_ports: Some(exposed_ports),
        host_config: Some(HostConfig {
            port_bindings: Some(port_bindings),
            cpu_shares: Some(resources.cpu_shares),
            memory: Some(resources.memory),
            memory_swap: Some(resources.memory_swap),
            nano_cpus: Some(resources.cpu_cores * 1_000_000_000),
            ..Default::default()
        }),
        labels: None,
        ..Default::default()
    }
}
pub fn check_user_resources(id: impl Into<String>) -> Result<ContainerResources, rusqlite::Error> {
    let username = id.into();
    let containers = db::get_user_containers(&username)?;
    let mut resources = ContainerResources {
        cpu_cores: 0,
        memory: 0,
        memory_swap: 0,
        cpu_shares: 0,
    };
    for container in containers {
        resources.cpu_cores += container.cpu_cores;
        resources.memory += container.memory;
        resources.memory_swap += container.memory_swap;
        resources.cpu_shares += container.cpu_shares;
    }
    Ok(ContainerResources {
        cpu_cores: resources.cpu_cores,
        memory: resources.memory,
        cpu_shares: resources.memory_swap,
        memory_swap: resources.cpu_shares,
    })
}
pub fn user_container_count(id: &String) -> Result<i32, Respond> {
    Ok(match db::count_containers_by_username(&id) {
        Ok(count) => count,
        Err(err) => match err {
            rusqlite::Error::QueryReturnedNoRows => 0,
            _ => {
                eprintln!("Error counting user's containers: {}", err);
                return Err(m_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Please contact support for help.",
                ));
            }
        },
    })
}
pub fn validate_container_resources(
    credits: i64,
    resources: &ContainerResources,
    revoke: Option<&str>,
) -> Result<bool, rusqlite::Error> {
    let needed_credits = resources.calculate_price();
    let revoked_credits = &credits - &needed_credits;
    if needed_credits < credits {
        if revoke.is_some() {
            db::set_user_credits(revoke.unwrap(), revoked_credits)?;
        }
        return Ok(true);
    } else {
        return Ok(false);
    }
}
pub fn get_available_port() -> Option<u16> {
    (59001..60000).find(|&port| TcpListener::bind(("127.0.0.1", port)).is_ok())
}

pub fn create_container(
    resources: ContainerResources,
    container_info: ContainerInfo,
    name: String,
    username: String,
) -> impl Future<Output = Respond> {
    return async move {
        let docker = match Docker::connect_with_local_defaults() {
            Ok(docker) => docker,
            Err(err) => {
                eprintln!("Error connecting to Docker: {}", err);
                return m_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Please contact support for help.",
                );
            }
        };
        let container_port = match get_available_port() {
            Some(port) => port,
            None => {
                return m_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "All ports are being used!",
                )
            }
        };
        let mut ports: HashMap<u16, u16> = HashMap::new();
        ports.insert(80, container_port);
        let config = create_config(ports, resources, container_info.image);
        let create_options = CreateContainerOptions {
            name: &name,
            platform: None,
        };

        println!("Container name: {}", &name);

        let container = match docker
            .create_container(Some(create_options), config.clone())
            .await
        {
            Ok(container) => container,
            Err(e) => {
                eprintln!("Error creating container: {}", e);
                return m_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed while creating container.",
                );
            }
        };

        println!("Created container with ID: {:?}", container.id);

        if let Err(e) = docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
        {
            eprintln!("Error starting container: {}", e);
            return m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed while starting container.",
            );
        }

        println!("Container started successfully.");

        match db::insert_container(&container.id, username, name, config, container_port).await {
            Ok(updated) if updated > 0 => Respond::Container(
                StatusCode::OK,
                CReturn {
                    id: container.id,
                    port: container_port,
                },
            ),
            Ok(_) => m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed while inserting container into DB.",
            ),
            Err(_) => m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed while inserting container into DB.",
            ),
        }
    };
}
