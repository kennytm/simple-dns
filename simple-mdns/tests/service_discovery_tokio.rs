#![cfg(feature = "async-tokio")]

use simple_mdns::{async_discovery::ServiceDiscovery, InstanceInformation};
use std::{collections::HashMap, error::Error, net::SocketAddr, str::FromStr, time::Duration};

#[allow(dead_code)]
fn init_log() {
    stderrlog::new()
        .verbosity(5)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();
}

#[tokio::test]
async fn service_discovery_can_find_services() -> Result<(), Box<dyn Error>> {
    // init_log();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut service_discovery_a = ServiceDiscovery::new("a", "_async3._tcp.local", 60)?;
    let mut service_discovery_b = ServiceDiscovery::new("b", "_async3._tcp.local", 60)?;
    let mut service_discovery_c = ServiceDiscovery::new("c", "_async3._tcp.local", 60)?;

    service_discovery_a
        .add_service_info(SocketAddr::from_str("192.168.1.2:8080")?.into())
        .await?;
    service_discovery_b
        .add_service_info(SocketAddr::from_str("192.168.1.3:8080")?.into())
        .await?;
    service_discovery_c
        .add_service_info(SocketAddr::from_str("192.168.1.4:8080")?.into())
        .await?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    let from_a: HashMap<String, SocketAddr> = service_discovery_a
        .get_known_services()
        .await
        .into_iter()
        .map(|(name, x)| (name, x.get_socket_addresses().next().unwrap()))
        .collect();

    let from_b: HashMap<String, SocketAddr> = service_discovery_b
        .get_known_services()
        .await
        .into_iter()
        .map(|(name, x)| (name, x.get_socket_addresses().next().unwrap()))
        .collect();

    let from_c: HashMap<String, SocketAddr> = service_discovery_c
        .get_known_services()
        .await
        .into_iter()
        .map(|(name, x)| (name, x.get_socket_addresses().next().unwrap()))
        .collect();

    assert_eq!(2, from_a.len());
    assert_eq!(2, from_b.len());
    assert_eq!(2, from_c.len());

    assert_eq!(&("192.168.1.3:8080".parse::<SocketAddr>()?), &from_a["b"]);
    assert_eq!(&("192.168.1.4:8080".parse::<SocketAddr>()?), &from_a["c"]);

    assert_eq!(&("192.168.1.2:8080".parse::<SocketAddr>()?), &from_b["a"]);
    assert_eq!(&("192.168.1.4:8080".parse::<SocketAddr>()?), &from_b["c"]);

    assert_eq!(&("192.168.1.2:8080".parse::<SocketAddr>()?), &from_c["a"]);
    assert_eq!(&("192.168.1.3:8080".parse::<SocketAddr>()?), &from_c["b"]);
    Ok(())
}

#[tokio::test]
async fn service_discovery_is_notified_on_discovery() -> Result<(), Box<dyn Error>> {
    // init_log();

    std::thread::sleep(Duration::from_secs(1));

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let mut service_discovery_a = ServiceDiscovery::new_with_scope(
        "a",
        "_async_notify._tcp.local",
        60,
        Some(tx),
        simple_mdns::NetworkScope::V4,
    )?;
    let mut service_discovery_b = ServiceDiscovery::new("b", "_async_notify._tcp.local", 60)?;
    let mut service_discovery_c = ServiceDiscovery::new("c", "_async_notify._tcp.local", 60)?;

    service_discovery_a
        .add_service_info(SocketAddr::from_str("192.168.1.2:8080")?.into())
        .await?;
    service_discovery_b
        .add_service_info(SocketAddr::from_str("192.168.1.3:8080")?.into())
        .await?;
    service_discovery_c
        .add_service_info(SocketAddr::from_str("192.168.1.4:8080")?.into())
        .await?;

    for _ in 0..2 {
        let Some((instance_name, service_info)) = rx.recv().await else {
            panic!("Did not receive enough packets");
        };

        let addr = service_info.get_socket_addresses().next().unwrap();
        match instance_name.as_str() {
            "b" => assert_eq!(("192.168.1.3:8080".parse::<SocketAddr>()?), addr),
            "c" => assert_eq!(("192.168.1.4:8080".parse::<SocketAddr>()?), addr),
            _ => panic!("Received unexpected packet"),
        }
    }

    Ok(())
}

#[tokio::test]
async fn service_discovery_receive_attributes() -> Result<(), Box<dyn Error>> {
    // init_log();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut service_discovery_d = ServiceDiscovery::new("d", "_srv4._tcp.local", 60)?;
    let mut service_discovery_e = ServiceDiscovery::new("e", "_srv4._tcp.local", 60)?;

    let mut service_info: InstanceInformation = SocketAddr::from_str("192.168.1.2:8080")?.into();
    service_info
        .attributes
        .insert("id".to_string(), Some("id_d".to_string()));

    service_discovery_d
        .add_service_info(service_info)
        .await
        .expect("Failed to add service info");
    let mut service_info: InstanceInformation = SocketAddr::from_str("192.168.1.3:8080")?.into();
    service_info
        .attributes
        .insert("id".to_string(), Some("id_e".to_string()));
    service_discovery_e
        .add_service_info(service_info)
        .await
        .expect("Failed to add service info");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let d_attr: HashMap<String, Option<String>> = service_discovery_d
        .get_known_services()
        .await
        .into_iter()
        .flat_map(|(_, x)| x.attributes)
        .collect();

    let e_attr: HashMap<String, Option<String>> = service_discovery_e
        .get_known_services()
        .await
        .into_iter()
        .flat_map(|(_, x)| x.attributes)
        .collect();

    assert_eq!(1, d_attr.len());
    assert_eq!(1, e_attr.len());

    assert_eq!("id_e", d_attr.get("id").as_ref().unwrap().as_ref().unwrap());
    assert_eq!("id_d", e_attr.get("id").as_ref().unwrap().as_ref().unwrap());

    Ok(())
}

#[tokio::test]
#[cfg(not(target_os = "macos"))]
async fn service_discovery_can_find_services_ipv6() -> Result<(), Box<dyn Error>> {
    // init_log();

    tokio::time::sleep(Duration::from_secs(1)).await;

    let mut service_discovery_a = ServiceDiscovery::new_with_scope(
        "a",
        "_async3._tcp.local",
        60,
        None,
        simple_mdns::NetworkScope::V6,
    )?;
    let mut service_discovery_b = ServiceDiscovery::new_with_scope(
        "b",
        "_async3._tcp.local",
        60,
        None,
        simple_mdns::NetworkScope::V6,
    )?;
    let mut service_discovery_c = ServiceDiscovery::new_with_scope(
        "c",
        "_async3._tcp.local",
        60,
        None,
        simple_mdns::NetworkScope::V6,
    )?;

    service_discovery_a
        .add_service_info(SocketAddr::from_str("[fe80::26fc:f50f:6755:7d67]:8080")?.into())
        .await
        .expect("Failed to add service info");
    service_discovery_b
        .add_service_info(SocketAddr::from_str("[fe80::26fc:f50f:6755:7d68]:8080")?.into())
        .await
        .expect("Failed to add service info");
    service_discovery_c
        .add_service_info(SocketAddr::from_str("[fe80::26fc:f50f:6755:7d69]:8080")?.into())
        .await
        .expect("Failed to add service info");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let from_a: HashMap<String, SocketAddr> = service_discovery_a
        .get_known_services()
        .await
        .into_iter()
        .map(|(name, x)| (name, x.get_socket_addresses().next().unwrap()))
        .collect();

    let from_b: HashMap<String, SocketAddr> = service_discovery_b
        .get_known_services()
        .await
        .into_iter()
        .map(|(name, x)| (name, x.get_socket_addresses().next().unwrap()))
        .collect();

    let from_c: HashMap<String, SocketAddr> = service_discovery_c
        .get_known_services()
        .await
        .into_iter()
        .map(|(name, x)| (name, x.get_socket_addresses().next().unwrap()))
        .collect();

    assert_eq!(2, from_a.len());
    assert_eq!(2, from_b.len());
    assert_eq!(2, from_c.len());

    assert_eq!(
        &("[fe80::26fc:f50f:6755:7d68]:8080".parse::<SocketAddr>()?),
        &from_a["b"]
    );
    assert_eq!(
        &("[fe80::26fc:f50f:6755:7d69]:8080".parse::<SocketAddr>()?),
        &from_a["c"]
    );

    assert_eq!(
        &("[fe80::26fc:f50f:6755:7d67]:8080".parse::<SocketAddr>()?),
        &from_b["a"]
    );
    assert_eq!(
        &("[fe80::26fc:f50f:6755:7d69]:8080".parse::<SocketAddr>()?),
        &from_b["c"]
    );

    assert_eq!(
        &("[fe80::26fc:f50f:6755:7d67]:8080".parse::<SocketAddr>()?),
        &from_c["a"]
    );
    assert_eq!(
        &("[fe80::26fc:f50f:6755:7d68]:8080".parse::<SocketAddr>()?),
        &from_c["b"]
    );
    Ok(())
}
