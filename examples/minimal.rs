use signal::{Signal, SignalNoClone, _spawn};
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
struct Potato {
    _number: i64,
}

fn main() {
    _spawn("Main loop".into(), async {
        let basic_signal = Signal::new();
        let complex_signal: Signal<Potato> = Signal::new();
        let mut no_clone_signal: SignalNoClone<Potato> = SignalNoClone::new();

        basic_signal.connect(|msg| println!("Slot1 received: {}", msg));
        basic_signal.connect(|msg| println!("Slot2 received: {}", msg));

        complex_signal.connect(|msg| println!("Complex Slot1 received: {:#?}", msg));
        complex_signal.connect(|msg| println!("Complex Slot2 received: {:#?}", msg));

        no_clone_signal.connect(|msg| println!("NoClone Slot1 received: {:#?}", msg));
        // No Clone channels should not be connected twice
        // no_clone_signal.connect(|msg| println!("NoClone Slot2 received: {:#?}", msg));

        basic_signal.emit(10);
        basic_signal.emit(20);
        complex_signal.emit(Potato { _number: 42 });
        complex_signal.emit(Potato { _number: 69 });
        no_clone_signal.emit(Potato { _number: 128 }).await;
        no_clone_signal.emit(Potato { _number: 256 }).await;
        sleep(Duration::from_millis(100)).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
}
