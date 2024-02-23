#![feature(test)] // Enable the 'test' feature
extern crate test;

#[derive(Clone, Debug, PartialEq)]
struct Potato {
    pub number: i64,
}

#[derive(Debug, PartialEq)]
struct Atom {
    pub number: i64,
}

use signal::{Signal, SignalNoClone};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

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
