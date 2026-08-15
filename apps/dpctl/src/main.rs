use kd3b_device::TargetIdentity;

fn main() {
    let target = TargetIdentity::default();
    println!(
        "dpctl baseline: target {:04x}:{:04x}; hardware access is not implemented yet",
        target.vendor_id, target.product_id
    );
}
