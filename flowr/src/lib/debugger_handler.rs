use serde_json::Value;

use flowcore::errors::*;
use flowcore::model::input::Input;
use flowcore::model::output_connection::OutputConnection;
use flowcore::model::runtime_function::RuntimeFunction;

use crate::block::Block;
use crate::debug_command::DebugCommand;
use crate::job::Job;
use crate::run_state::RunState;
use crate::run_state::State;

/// Programs that wish to offer a debugger user interface (such as a CLI or UI) should implement
/// this trait. The [Coordinator][crate::coordinator::Coordinator] uses it to interact with the
/// client for debugging.
pub trait DebuggerHandler {
    /// Start the debugger - which swallows the first message to initialize the connection
    fn start(&mut self);
    /// a breakpoint has been hit on a Job being created
    fn job_breakpoint(&mut self, job: &Job, function: &RuntimeFunction, states: Vec<State>);
    /// A breakpoint set on creation of a `Block` matching `block` has been hit
    fn block_breakpoint(&mut self, block: &Block);
    /// A breakpoint set on the unblocking of a flow has been hit
    fn flow_unblock_breakpoint(&mut self, flow_id: usize);
    /// A breakpoint on sending a value from a specific function or to a specific function was hit
    #[allow(clippy::too_many_arguments)]
    fn send_breakpoint(&mut self, source_function_name: &str, source_function_id: usize,
                       output_route: &str, value: &Value, destination_id: usize,
                       destination_name: &str, io_name: &str, input_number: usize);
    /// A job error occurred during execution of the flow
    fn job_error(&mut self, job: &Job);
    /// A specific job completed
    fn job_completed(&mut self, job: &Job);
    /// returns a set of blocks
    fn blocks(&mut self, blocks: Vec<Block>);
    /// returns an output's connections
    fn outputs(&mut self, output: Vec<OutputConnection>);
    /// returns an inputs state
    fn input(&mut self, input: Input);
    /// lists all functions
    fn function_list(&mut self, functions: &[RuntimeFunction]);
    /// returns the state of a function
    fn function_states(&mut self, function: RuntimeFunction, function_states: Vec<State>);
    /// returns the global run state
    fn run_state(&mut self, run_state: &RunState);
    /// a string message from the Debugger
    fn message(&mut self, message: String);
    /// a panic occurred during execution
    fn panic(&mut self, state: &RunState, error_message: String);
    /// the debugger is exiting
    fn debugger_exiting(&mut self);
    /// The debugger is resetting the runtime state
    fn debugger_resetting(&mut self);
    /// An error occurred in the debugger
    fn debugger_error(&mut self, error: String);
    /// execution of the flow is starting
    fn execution_starting(&mut self);
    /// Execution of the flow fn execution_ended(&mut self, state: &RunState) {
    fn execution_ended(&mut self);
    /// Get a command for the debugger to perform
    fn get_command(&mut self, state: &RunState) -> Result<DebugCommand>;
}