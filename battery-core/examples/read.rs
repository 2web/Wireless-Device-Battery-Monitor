//! Hardware check: discover supported mice and read their battery the way the GUI does.
//! `cargo run -p battery-protocol --example read`
fn main() {
    let inventory = battery_protocol::hid::discover_inventory_with_diagnostics()
        .expect("HID discovery failed");
    if inventory.targets.is_empty() {
        println!("No supported device found.");
    }
    for target in inventory.targets {
        println!("{} [{}]", target.info.name, target.info.id);
        match target.open().and_then(|mut s| s.read(0)) {
            Ok(r) => println!("  {:?} {:?} via {:?}", r.percentage(), r.charge, r.source),
            Err(e) => println!("  error: {e}"),
        }
    }
}
