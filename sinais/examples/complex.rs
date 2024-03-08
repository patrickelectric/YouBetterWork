use sinais::{Signal, _spawn};
use tokio::time::{sleep, Duration};

struct ComplexGenerator {
    index: u8,
    __signal_index: Signal<u8>,
}

impl ComplexGenerator {
    pub fn new() -> Self {
        Self {
            index: 0,
            __signal_index: Signal::new(),
        }
    }

    pub fn run(&mut self) {
        self.index += 1;
        self.emit_index();
    }

    pub fn on_index_changed(&self) -> &Signal<u8> {
        &self.__signal_index
    }

    fn emit_index(&self) {
        self.__signal_index.emit(self.index);
    }
}

fn main() {
    _spawn("Main loop".into(), async {
        let mut generator = ComplexGenerator::new();

        generator
            .on_index_changed()
            .connect(|msg| println!("Slot1 received: {}", msg));
        generator.run();
        sleep(Duration::from_millis(100)).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
}
