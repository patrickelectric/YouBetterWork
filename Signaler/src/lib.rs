use tokio::sync::{broadcast, mpsc};

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tracing::*;
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
        debug!("Starting task {}", name.clone());
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
        debug!("Task master is closing: {:#?}", self.tasks.keys());
        self.clear_finished();

        loop {
            let running_tasks = self.list_running_tasks();
            if running_tasks.is_empty() {
                break;
            }

            debug!("Waiting for tasks to finish: {:?}", running_tasks);
            std::thread::sleep(std::time::Duration::from_millis(2000));
        }
        debug!("Task master is closed.")
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

#[derive(Clone)]
pub struct Signal<T> {
    sender: broadcast::Sender<T>,
}

impl<T: Send + Clone + 'static> Signal<T> {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Signal { sender: tx }
    }

    pub fn connect(&self, slot: impl Fn(T) + Send + 'static) {
        self.connect_named(slot, Uuid::new_v4().into());
    }

    pub fn connect_named(&self, slot: impl Fn(T) + Send + 'static, name: String) {
        debug!("Channel {} created", name);
        let mut receiver = self.sender.subscribe();

        _spawn(name.clone(), async move {
            loop {
                match receiver.recv().await {
                    Ok(msg) => slot(msg),
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Channel {} is closed", name);
                        break;
                    }
                    Err(e) => {
                        debug!("Channel {} error {:#?}", name, e);
                    }
                }
            }
            debug!("Channel {} finished event loop", name);
        });
    }

    pub fn emit_result(&self, message: T) -> Result<usize, broadcast::error::SendError<T>> {
        self.sender.send(message)
    }

    pub fn emit(&self, message: T) {
        let _ = self.emit_result(message);
    }
}

// Move it to another file and use same traits and names as signal (No SignalNoClone)
pub struct SignalNoClone<T> {
    sender: mpsc::Sender<T>,
    receiver: Option<mpsc::Receiver<T>>,
}

impl<T: Send + 'static> SignalNoClone<T> {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        SignalNoClone {
            sender: tx,
            receiver: Some(rx),
        }
    }

    pub fn connect(&mut self, slot: impl Fn(T) + Send + 'static) {
        self.connect_named(slot, Uuid::new_v4().into());
    }

    pub fn connect_named(&mut self, slot: impl Fn(T) + Send + 'static, name: String) {
        debug!("Channel NoClone {} created", name);
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
            debug!("Closing NoClone channel {}", name);
        });
    }

    pub async fn emit_result(&self, message: T) -> Result<(), mpsc::error::SendError<T>> {
        self.sender.send(message).await
    }

    pub async fn emit(&self, message: T) {
        let _ = self.emit_result(message).await;
    }
}

pub struct SignalInner<T, K> {
    pub calls: Vec<fn(&mut T, K)>,
}

impl<T, K: Clone> SignalInner<T, K> {
    pub fn new() -> Self {
        Self {
            calls: vec![],
        }
    }

    pub fn add(&mut self, slot: fn(&mut T, K)) {
        self.calls.push(slot);
    }

    /*
    #[inline]
    pub fn run(&mut self, instance: &mut T, value: K) {
        let mut calls = std::mem::replace(&mut self.calls, Vec::new());
        for call in calls.iter_mut() {
            call(instance, value.clone());
        }
        self.calls = calls;
    }
    */
}