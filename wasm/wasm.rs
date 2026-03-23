#[link(wasm_import_module = "nats")]
unsafe extern "C" {
    #[link_name = "publish"]
    fn nats_publish(
        subject_ptr: *const u8,
        subject_len: usize,
        payload_ptr: *const u8,
        payload_len: usize,
    );
}

pub fn publish(subject: &str, payload: &str) {
    unsafe {
        nats_publish(
            subject.as_ptr(),
            subject.len(),
            payload.as_ptr(),
            payload.len(),
        );
    }
}

fn main() {
    let name = std::env::var("NAME").unwrap();
    println!("Hello, world! My name is {name}");
    publish("hello", "Hello from Rust!");
    std::thread::sleep(std::time::Duration::from_secs(1));
    publish("goodbye", "Goodbye from Rust!");
    println!("Goodbye from {name}");
}
