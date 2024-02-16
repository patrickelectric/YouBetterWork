use tokio::sync::broadcast;
use std::thread;

use std::collections::HashMap;
use std::future::Future;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use std::sync::Mutex;
use lazy_static::lazy_static;
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

lazy_static ! {
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
        Signal {
            sender: tx,
        }
    }

    fn connect(&self, slot: impl Fn(T) + Send + 'static + Clone, name: Option<String>) {
        let mut receiver = self.sender.subscribe();

        let name = name.unwrap_or(Uuid::new_v4().into());
        _spawn(name.clone() ,async move {
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

    fn emit(&self, message: T) {
        let _ = self.sender.send(message);
    }
}

#[derive(Clone, Debug)]
struct Potato {
    pub number: i64
}

fn main() {
    _spawn("Main loop".into(), async {
        let basic_signal = Signal::new();
        let complex_signal: Signal<Potato> = Signal::new();

        basic_signal.connect(|msg| println!("Slot1 received: {}", msg), None);
        basic_signal.connect(|msg| println!("Slot2 received: {}", msg), None);

        complex_signal.connect(|msg| println!("Complex Slot1 received: {:#?}", msg), None);
        complex_signal.connect(|msg| println!("Complex Slot2 received: {:#?}", msg), None);

        basic_signal.emit(10);
        basic_signal.emit(20);
        complex_signal.emit(Potato { number: 42 });
        complex_signal.emit(Potato { number: 69 });
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
}