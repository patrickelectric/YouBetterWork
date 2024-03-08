use sinais_macro::*;
use sinais::*;

use tokio::time::{sleep, Duration};

#[derive(Default, Signaler)]
struct Person {
    age: u32,
    #[property]
    name: String,
}

fn main() {
    _spawn("Main loop".into(), async {
        let mut obj = PersonSignaler::default();
        obj.on_name_changed()
            .connect(|new_value| println!("Name changed: {}", new_value));
        obj.set_name("Patrick".into());
        sleep(Duration::from_millis(100)).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(5));
}
