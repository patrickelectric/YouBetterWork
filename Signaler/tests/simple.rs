#[derive(Clone, Debug, PartialEq)]
struct Potato {
    pub number: i64,
}

#[derive(Debug, PartialEq)]
struct Atom {
    pub number: i64,
}

use decorators::*;
use signal::*;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

use test_log::test;

struct Tester<T> {
    captured: Arc<Mutex<Vec<T>>>,
    values: Vec<T>,
}

impl<T: PartialEq> Tester<T> {
    fn new(values: Vec<T>) -> Self {
        Self {
            captured: Arc::new(Mutex::new(Vec::new())),
            values,
        }
    }

    fn clone(&self) -> Arc<std::sync::Mutex<Vec<T>>> {
        self.captured.clone()
    }

    fn is_valid(&self) -> bool {
        self.values
            .iter()
            .all(|value| self.captured.lock().unwrap().contains(&value))
    }
}

#[test]
fn test_simple_signal_behavior() {
    let runtime = Runtime::new().unwrap();

    let basic_signal = Signal::new();

    let basic_values = vec![10, 20, 30];
    let captured_basic_signal1 = Tester::new(basic_values.clone());
    let captured_basic_signal2 = Tester::new(basic_values.clone());

    let a = captured_basic_signal1.clone();
    let b = captured_basic_signal2.clone();
    basic_signal.connect(move |msg| a.lock().unwrap().push(msg));
    basic_signal.connect(move |msg| b.lock().unwrap().push(msg));

    runtime.block_on(async move {
        assert!(!captured_basic_signal1.is_valid());
        assert!(!captured_basic_signal2.is_valid());

        for value in basic_values {
            basic_signal.emit(value);
        }
        sleep(Duration::from_millis(100)).await;

        assert!(captured_basic_signal1.is_valid());
        assert!(captured_basic_signal2.is_valid());
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
}

#[test]
fn test_complex_signal_behavior() {
    let runtime = Runtime::new().unwrap();

    let complex_signal: Signal<Potato> = Signal::new();
    let mut no_clone_signal: SignalNoClone<Atom> = SignalNoClone::new();

    let complex_value = Potato { number: 42 };
    let captured_complex_signal = Tester::new(vec![complex_value.clone()]);
    let captured_no_clone_signal = Arc::new(Mutex::new(vec![]));

    let a = captured_complex_signal.clone();
    complex_signal.connect(move |msg| a.lock().unwrap().push(msg));
    let a = captured_no_clone_signal.clone();
    no_clone_signal.connect(move |msg| a.lock().unwrap().push(msg));

    runtime.block_on(async move {
        assert!(!captured_complex_signal.is_valid());
        assert!(captured_no_clone_signal.lock().unwrap().is_empty());

        complex_signal.emit(complex_value);
        no_clone_signal.emit(Atom { number: 69 }).await;

        sleep(Duration::from_millis(100)).await;

        assert!(captured_complex_signal.is_valid());
        assert_eq!(
            captured_no_clone_signal.lock().unwrap()[0],
            Atom { number: 69 }
        );
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
}

use std::time::Instant;

#[derive(Default, Signaler)]
struct SimpleTalker {
    #[property]
    value: u64,
}

#[derive(Signaler)]
struct Talker {
    #[property]
    values: Vec<u8>,
}

impl Default for Talker {
    fn default() -> Self {
        const BYTES_TO_GENERATE: usize = 1 * 2_usize.pow(20); // 1MB
        Self {
            values: vec![0; BYTES_TO_GENERATE],
        }
    }
}

#[test]
fn test_joao_hypothesis() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        const SIZE: usize = 2000;
        let mut tasks = [(); SIZE].map(|_| TalkerSignaler::default());
        let start = Instant::now();
        for mut task in tasks {
            task.emit_values();
        }
        println!(
            "Time elapsed in {} series emission: {:?}",
            SIZE,
            start.elapsed()
        );
        sleep(Duration::from_millis(100)).await;
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
}

#[test]
fn test_joao_example() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        const MINIMUM_MESSAGES_TO_RECEIVE: u64 = 1000;
        let mut task = SimpleTalkerSignaler::default();
        let should_wait = Arc::new(Mutex::new(true));
        let cloned_should_wait = should_wait.clone();

        task.on_value_changed().connect(move |value| {
            if value == MINIMUM_MESSAGES_TO_RECEIVE {
                *cloned_should_wait.lock().unwrap() = false;
            }
        });

        let start = Instant::now();
        for value in 0..=MINIMUM_MESSAGES_TO_RECEIVE {
            task.set_value(value)
        }

        loop {
            sleep(Duration::from_millis(1)).await;
            if *should_wait.lock().unwrap() == false {
                break;
            }
        }

        println!(
            "Time elapsed in sending {} emissions: {:?}",
            MINIMUM_MESSAGES_TO_RECEIVE,
            start.elapsed()
        );
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
}

#[test]
fn test_joao_chain_hypothesis() {
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async move {
        const SIZE: usize = 4000;
        let tasks = [(); SIZE].map(|_| Arc::new(Mutex::new(TalkerSignaler::default())));

        for pair in tasks.windows(2) {
            let first = pair[0].clone();
            let second = pair[1].clone();
            let first_lock = first.lock().unwrap();
            first_lock
                .on_values_changed()
                .connect(move |values| second.lock().unwrap().set_values(values))
        }
        let should_wait = Arc::new(Mutex::new(true));
        let cloned_should_wait = should_wait.clone();
        tasks
            .last()
            .unwrap()
            .lock()
            .unwrap()
            .on_values_changed()
            .connect(move |_| *cloned_should_wait.lock().unwrap() = false);

        tasks.first().unwrap().lock().unwrap().emit_values();
        let start = Instant::now();
        loop {
            sleep(Duration::from_millis(1)).await;
            if *should_wait.lock().unwrap() == false {
                break;
            }
        }
        println!(
            "Time elapsed in {} chain events: {:?}",
            SIZE,
            start.elapsed()
        );
        sleep(Duration::from_millis(100)).await;
    });

    std::thread::sleep(std::time::Duration::from_secs(1));
}
