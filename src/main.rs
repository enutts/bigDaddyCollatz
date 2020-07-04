use std::{env, thread, fs};
use std::sync::{Arc, Mutex, mpsc};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    let crange = args[1].as_str();
    let crange: u128  = match crange.trim().parse() {
        Ok(num) => num,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let tp_size = args[2].as_str();
    let tp_size: usize = match tp_size.trim().parse() {
        Ok(num) => num,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let answers = Arc::new(Mutex::new(vec![]));

    {
        let pool = ThreadPool::new(tp_size);
        let crange = crange + 1;

        for num in 2..crange {
            let answer = Arc::clone(&answers);
            pool.execute(move || {
                let a = collatz(num, None);
                answer.lock().unwrap().push((num, a));
            });
        }
    }

    answers.lock().unwrap().sort();

    let mut ans_string = String::new();

    for (num, ans) in answers.lock().unwrap().iter() {
        ans_string.push_str(&num.to_string());
        ans_string.push_str(",");
        ans_string.push_str(" ");
        ans_string.push_str(&ans.to_string());
        ans_string.push_str("\n");
    }

    fs::write("answers.txt", ans_string)?;
    Ok(())
}

fn collatz(i: u128, c: Option<u128>) -> u128 {
    let b = c.unwrap_or(0);

    if i == 1 {
        return b;
    }

    match i % 2 {
        0 => {
            let a = i / 2;
            let b = b + 1;
            return collatz(a, Some(b))
        }
        _ => {
            let a = 3 * i + 1;
            let b = b + 1;
            return collatz(a, Some(b))
        }
    }
}

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
                    break;
                }
            }
        });

        Worker { 
            id, 
            thread: Some(thread),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}
