use btleplug::api::{Central, Manager as _, Peripheral as _, WriteType};
use btleplug::platform::Manager;
use uuid::Uuid;
use anyhow::{Result, anyhow};
use tokio::time::{sleep, Duration};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let service_uuid = Uuid::parse_str("0000180d-0000-1000-8000-00805f9b34fb")?; // Heart Rate Service
    let characteristic_uuid = Uuid::parse_str("00002a39-0000-1000-8000-00805f9b34fb")?; // Heart Rate Control Point

    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let adapter = adapters.into_iter().next().ok_or_else(|| anyhow!("No Bluetooth adapter found"))?;

    println!("🔍 Scanning for BLE devices...");
    adapter.start_scan(Default::default()).await?;
    sleep(Duration::from_secs(5)).await;
    let peripherals = adapter.peripherals().await?;
    adapter.stop_scan().await?;

    if peripherals.is_empty() {
        println!("❌ No BLE devices found.");
        return Ok(());
    }

    println!("\n📡 Discovered Devices:");
    for (i, p) in peripherals.iter().enumerate() {
        if let Ok(Some(props)) = p.properties().await {
            let name = props.local_name.unwrap_or_else(|| "<unknown>".to_string());
            println!("{}: {} [{}]", i, name, props.address);
        }
    }

    print!("\n👉 Enter device number to connect: ");
    io::stdout().flush()?;
    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;
    let index: usize = selection.trim().parse().map_err(|_| anyhow!("Invalid input"))?;
    let peripheral = peripherals.get(index).ok_or_else(|| anyhow!("Selection out of range"))?;

    peripheral.connect().await?;
    println!("✅ Connected to: {:?}", peripheral.properties().await?.unwrap().local_name);
    peripheral.discover_services().await?;

    let characteristics = peripheral.characteristics();
    let characteristic = characteristics
        .iter()
        .find(|c| c.uuid == characteristic_uuid && c.service_uuid == service_uuid)
        .ok_or_else(|| anyhow!("Target characteristic not found"))?;

    let message = [0x01u8]; // Example command: start HR measurement
    peripheral.write(characteristic, &message, WriteType::WithoutResponse).await?;
    println!("📨 Message sent to characteristic!");

    peripheral.disconnect().await?;
    println!("🔌 Disconnected.");
    Ok(())
}