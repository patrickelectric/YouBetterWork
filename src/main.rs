use std::thread;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use uuid::Uuid;

// More information about this can be detailed explained here:
// https://www.youtube.com/watch?v=tP0ZrX-2EiE
pub struct TaskMaster {
    runtime: Runtime,
    tasks: HashMap<String, JoinHandle<()>>,
}

impl TaskMaster {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            tasks: HashMap::new(),
        }
    }

    pub fn spawn<F>(&mut self, name: String, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
        F::Output: Send + 'static,
    {
        println!("Starting {}", name.clone());
        let task = self.runtime.spawn(f);
        self.tasks.insert(name, task);
    }

    pub fn clear_finished(&mut self) {
        self.tasks.retain(|_, task| !task.is_finished());
    }

    pub fn get_task(&self, name: &str) -> Option<&JoinHandle<()>> {
        self.tasks.get(name)
    }

    pub fn list_running_tasks(&mut self) -> Vec<String> {
        self.clear_finished();
        self.tasks.keys().cloned().collect()
    }
}

impl Drop for TaskMaster {
    fn drop(&mut self) {
        println!("Task master is closing: {:#?}", self.tasks.keys());
        self.clear_finished();

        loop {
            let running_tasks = self.list_running_tasks();
            if running_tasks.is_empty() {
                break;
            }

            println!("Waiting for tasks to finish: {:?}", running_tasks);
            std::thread::sleep(std::time::Duration::from_millis(2000));
        }
        todo!("We need to ensure that drop is being called")
    }
}

lazy_static! {
    static ref TASK_MASTER: Mutex<TaskMaster> = Mutex::new(TaskMaster::new());
}

pub fn _spawn<F>(name: String, f: F)
where
    F: Future<Output = ()> + Send + 'static,
    F::Output: Send + 'static,
{
    TASK_MASTER.lock().unwrap().spawn(name, f);
}

struct Signal<T> {
    sender: broadcast::Sender<T>,
}

impl<T: Send + Clone + 'static> Signal<T> {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Signal { sender: tx }
    }

    fn connect(&self, slot: impl Fn(T) + Send + 'static + Clone) {
        self.connect_named(slot, Uuid::new_v4().into());
    }

    fn connect_named(&self, slot: impl Fn(T) + Send + 'static + Clone, name: String) {
        let mut receiver = self.sender.subscribe();

        _spawn(name.clone(), async move {
            loop {
                match receiver.recv().await {
                    Ok(msg) => slot(msg),
                    Err(broadcast::error::RecvError::Closed) => {
                        println!("Closing channel for {}", name);
                        break;
                    }
                    Err(e) => {
                        dbg!(e);
                    }
                }
            }
        });
    }

    fn emit_result(&self, message: T) -> Result<usize, broadcast::error::SendError<T>> {
        self.sender.send(message)
    }

    fn emit(&self, message: T) {
        let _ = self.emit_result(message);
    }
}

// Move it to another file and use same traits and names as signal (No SignalNoClone)
struct SignalNoClone<T> {
    sender: mpsc::Sender<T>,
    receiver: Option<mpsc::Receiver<T>>,
}

impl<T: Send + 'static> SignalNoClone<T> {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        SignalNoClone {
            sender: tx,
            receiver: Some(rx),
        }
    }

    fn connect(&mut self, slot: impl Fn(T) + Send + 'static + Clone) {
        self.connect_named(slot, Uuid::new_v4().into());
    }

    fn connect_named(&mut self, slot: impl Fn(T) + Send + 'static + Clone, name: String) {
        if self.receiver.is_none() {
            todo!("You can't connect twice in a no clone channel. Return error here");
        }
        let mut receiver = self.receiver.take().unwrap();
        _spawn(name.clone(), async move {
            // This method returns `None` if the channel has been closed and there are
            // no remaining messages in the channel's buffer. This indicates that no
            // further values can ever be received from this `Receiver`. The channel is
            // closed when all senders have been dropped, or when [`close`] is called.
            while let Some(msg) = receiver.recv().await {
                slot(msg)
            }
            println!("Closing NoClone channel for {}", name);
        });
    }

    async fn emit_result(&self, message: T) -> Result<(), mpsc::error::SendError<T>> {
        self.sender.send(message).await
    }

    async fn emit(&self, message: T) {
        let _ = self.emit_result(message).await;
    }
}

#[derive(Clone, Debug)]
struct Potato {
    pub number: i64,
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
        //no_clone_signal.connect(|msg| println!("NoClone Slot2 received: {:#?}", msg));

        basic_signal.emit(10);
        basic_signal.emit(20);
        complex_signal.emit(Potato { number: 42 });
        complex_signal.emit(Potato { number: 69 });
        no_clone_signal.emit(Potato { number: 128 }).await;
        no_clone_signal.emit(Potato { number: 256 }).await;
        sleep(Duration::from_millis(100)).await;
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
}
