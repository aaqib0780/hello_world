use btleplug::api::{Central, Manager as _, Peripheral as _, WriteType, CharPropFlags};
use btleplug::platform::Manager;
use tokio::time;
use uuid::Uuid;
use anyhow::Result;
use std::time::Duration;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Get the target address from command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <TARGET_BLUETOOTH_ADDRESS>", args[0]);
        return Ok(());
    }
    let target_address = &args[1];

    // Define the service and characteristic UUIDs
    let service_uuid = Uuid::parse_str("0000180d-0000-1000-8000-00805f9b34fb")?;
    let characteristic_uuid = Uuid::parse_str("00002a39-0000-1000-8000-00805f9b34fb")?;

    // Set up the BLE manager
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().next().ok_or_else(|| anyhow::anyhow!("No Bluetooth adapter found"))?;

    // Start scanning for devices
    adapter.start_scan(Default::default()).await?;
    println!("Scanning for devices...");

    // Wait a bit to discover devices
    time::sleep(Duration::from_secs(5)).await;

    // Find the target device (modify to match your device's name or address)
    let peripherals = adapter.peripherals().await?;
    println!("Discovered devices:");
    for p in &peripherals {
        if let Ok(Some(props)) = p.properties().await {
            println!("Device properties: {:?}", props);
        }
    }
    let mut target_peripheral = None;
    for p in peripherals {
        if let Ok(Some(props)) = p.properties().await {
            println!("Comparing {} to {}", props.address.to_string(), target_address);
            if props.address.to_string().to_uppercase() == target_address.to_uppercase() {
                target_peripheral = Some(p);
                break;
            }
        }
    }
    let peripheral = target_peripheral.ok_or_else(|| anyhow::anyhow!("No target device found"))?;

    // Connect to the device
    peripheral.connect().await?;
    println!("Connected to device: {:?}", peripheral.properties().await?.unwrap().local_name);

    // Discover services and characteristics
    peripheral.discover_services().await?;
    let characteristics = peripheral.characteristics();
    println!("Discovered services and characteristics:");
    for c in &characteristics {
        println!("Service: {:?}, Characteristic: {:?}, Properties: {:?}", c.service_uuid, c.uuid, c.properties);
    }
    // Find the specific writable characteristic
    let characteristic = characteristics
        .iter()
        .find(|c| c.uuid == characteristic_uuid && c.service_uuid == service_uuid)
        .ok_or_else(|| anyhow::anyhow!("Characteristic 00002a39-0000-1000-8000-00805f9b34fb not found in service 0000180d-0000-1000-8000-00805f9b34fb"))?;

    // Send a single byte message
    let message = [0x01u8];
    match peripheral.write(characteristic, &message, WriteType::WithoutResponse).await {
        Ok(_) => println!("Message sent: {:?}", message),
        Err(e) => println!("Failed to write to characteristic: {:?}", e),
    }

    // Disconnect
    peripheral.disconnect().await?;
    println!("Disconnected");

    Ok(())
}