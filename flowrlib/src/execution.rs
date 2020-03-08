use std::panic;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use log::{error, trace};

use crate::errors::*;
use crate::run_state::{Job, Output};

/*
    Start a number of executor threads that all listen on the 'job_rx' channel for
    Jobs to execute and return the Outputs on the 'output_tx' channel
*/
pub fn start_executors(number_of_executors: usize,
                       job_rx: &Arc<Mutex<Receiver<Job>>>,
                       output_tx: &Sender<Output>) {
    for executor_number in 0..number_of_executors {
        create_executor(format!("Executor #{}", executor_number),
                        job_rx.clone(), output_tx.clone());
    }
}

/*
    Replace the standard panic hook with one that just outputs the file and line of any process's
    run-time panic.
*/
pub fn set_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        /* Only available on 'nightly'
        if let Some(message) = panic_info.message() {
            error!("Message: {:?}", message);
        }
        */

        if let Some(location) = panic_info.location() {
            error!("Panic in file '{}' at line {}", location.file(), location.line());
        }
    }));
}

fn create_executor(name: String, job_rx: Arc<Mutex<Receiver<Job>>>, output_tx: Sender<Output>) {
    let executor_name = name.clone();
    let builder = thread::Builder::new().name(name);
    let _ = builder.spawn( move || {
        set_panic_hook();

        loop {
            let _ = get_and_execute_job(&job_rx, &output_tx, &executor_name);
        }
    });
}

fn get_and_execute_job(job_rx: &Arc<Mutex<Receiver<Job>>>,
                       output_tx: &Sender<Output>,
                       name: &str) -> Result<String> {
    // TODO write a convert method so I can chain this error too?
    let guard = job_rx.lock().map_err(|e| e.to_string())?;
    match guard.recv() {
        Ok(job) => execute(job, output_tx, name),
        Err(_) => Ok("Probably channel closure".into())
    }
}

fn execute(job: Job, output_tx: &Sender<Output>, name: &str) -> Result<String> {
    // Run the job and catch the execution result
    let (result, error) = match panic::catch_unwind(|| {
        trace!("Job #{} Executing on Executor '{}'", job.job_id, name);
        job.implementation.run(job.input_set.clone())
    }) {
        Ok(result) => (result, None),
        Err(_) => ((None, false), Some("Execution panicked".into())),
    };

    let output = Output {
        job_id: job.job_id,
        function_id: job.function_id,
        flow_id: job.flow_id,
        input_values: job.input_set,
        result,
        destinations: job.destinations,
        error,
    };

    output_tx.send(output).unwrap();

    Ok("Job Executed".into())
}