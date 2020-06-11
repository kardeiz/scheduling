# scheduling

A very simple job scheduler. Runs one job (one-time or recurring) on one spawned thread.

## Usage

```rust
fn main() {
    let _once_handle = scheduling::Scheduler::once(|| println!("ONCE")).start();

    let recurring_handle = scheduling::Scheduler::delayed_recurring(
        std::time::Duration::from_secs(1),
        std::time::Duration::from_secs(1),
        || println!("1 SEC ELAPSED"),
    )
    .start();

    std::thread::sleep(std::time::Duration::from_secs(5));

    recurring_handle.cancel();

    std::thread::sleep(std::time::Duration::from_secs(5));
}
```

License: MIT
