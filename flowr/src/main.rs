#![deny(missing_docs)]
#![warn(clippy::unwrap_used)]
//! `flowr` is the flow runner. It reads a `flow` `Manifest` produced
//! by a flow compiler, such as `flowc`` that describes the network of collaborating functions
//! that are executed to execute the flow.
//!
//! Use `flowr` or `flowr --help` or `flowr -h` at the comment line to see the command line options

use std::env;
use std::process::exit;

use clap::{App, AppSettings, Arg, ArgMatches};
use log::{error, info, warn};
use simpath::Simpath;
//use simpdiscoverylib::{BeaconListener, BeaconSender};
use simplog::simplog::SimpleLogger;
use url::Url;

use errors::*;
use flowcore::url_helper::url_from_string;
//use simple_dns::rdata::{RData, A, SRV};
//use simple_dns::{Name, ResourceRecord, CLASS};
//use simple_mdns::{OneShotMdnsResolver, SimpleMdnsResponder};
//use std::net::Ipv4Addr;
use flowrlib::client_server::ClientConnection;
use flowrlib::coordinator::{Coordinator, Mode, Submission};
use flowrlib::info as flowrlib_info;

#[cfg(feature = "debugger")]
use crate::cli_debug_client::CliDebugClient;
use crate::cli_runtime_client::CliRuntimeClient;

//use std::time::Duration;

#[cfg(feature = "debugger")]
mod cli_debug_client;
mod cli_runtime_client;

//const BEACON_PORT: u16 = 9001;
//const FLOW_SERVICE_NAME: &str = "_flowr._tcp.local";

/// We'll put our errors in an `errors` module, and other modules in this crate will
/// `use crate::errors::*;` to get access to everything `error_chain` creates.
pub mod errors;

fn main() {
    match run() {
        Err(ref e) => {
            error!("{}", e);

            for e in e.iter().skip(1) {
                error!("caused by: {}", e);
            }

            // The backtrace is not always generated. Try to run this example
            // with `RUST_BACKTRACE=1`.
            if let Some(backtrace) = e.backtrace() {
                error!("backtrace: {:?}", backtrace);
            }

            exit(1);
        }
        Ok(_) => exit(0),
    }
}

/// For the lib provider, libraries maybe installed in multiple places in the file system.
/// In order to find the content, a FLOW_LIB_PATH environment variable can be configured with a
/// list of directories in which to look for the library in question.
///
/// Using the "FLOW_LIB_PATH" environment variable attempt to locate the library's root folder
/// in the file system.
pub fn set_lib_search_path(search_path_additions: &[String]) -> Result<Simpath> {
    let mut lib_search_path = Simpath::new_with_separator("FLOW_LIB_PATH", ',');

    if env::var("FLOW_LIB_PATH").is_err() && search_path_additions.is_empty() {
        warn!("'FLOW_LIB_PATH' is not set, and no LIB_DIRS supplied, so it is possible libraries referenced will not be found");
    }

    for additions in search_path_additions {
        lib_search_path.add(additions);
        info!("'{}' added to the Library Search Path", additions);
    }

    Ok(lib_search_path)
}

fn run() -> Result<()> {
    info!(
        "'{}' version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    info!("'flowrlib' version {}", flowrlib_info::version());

    let matches = get_matches();

    SimpleLogger::init(matches.value_of("verbosity"));
    let debugger = matches.is_present("debugger");
    let native = matches.is_present("native");
    let lib_dirs = if matches.is_present("lib_dir") {
        matches
            .values_of("lib_dir")
            .chain_err(|| "Could not get the list of 'LIB_DIR' options specified")?
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![]
    };
    let lib_search_path = set_lib_search_path(&lib_dirs)?;

    let (mode, server_address) = if matches.is_present("client") {
        let s_address = match matches.value_of("address") {
            Some(address) => {
                info!("'SERVER_ADDRESS' set to '{}'", address);
                Some(address.to_string())
            }
            None => discover_server(),
        };

        if s_address.is_none() {
            bail!(
                "ClientOnly mode: Server address was not specified as an option and no server \
            could be discovered."
            );
        }
        (Mode::ClientOnly, s_address)
    } else if matches.is_present("server") {
        #[cfg(feature = "distributed")]
        enable_server_discovery()?;
        (Mode::ServerOnly, None)
    } else {
        (Mode::ClientAndServer, None) // the default if nothing specified
    };
    info!("Starting 'flowr' in {:?} mode", mode);

    if mode != Mode::ClientOnly {
        #[cfg(feature = "debugger")]
        Coordinator::server(
            num_threads(&matches, debugger),
            lib_search_path,
            native,
            mode.clone(),
            &server_address,
            5555,
            5556,
        )?;
    }

    if mode != Mode::ServerOnly {
        start_client(matches, debugger, server_address)?;
    }

    Ok(())
}

/*
   Start the client that talks to the server - whether another thread in this same process
   or to another process.
*/
fn start_client(
    matches: ArgMatches,
    debugger: bool,
    server_hostname: Option<String>,
) -> Result<()> {
    let flow_manifest_url = parse_flow_url(&matches)?;
    let flow_args = get_flow_args(&matches, &flow_manifest_url);
    let submission = Submission::new(
        &flow_manifest_url,
        num_parallel_jobs(&matches, debugger),
        #[cfg(feature = "debugger")]
        debugger,
    );

    let runtime_client_connection = ClientConnection::new(&server_hostname, 5555)?;
    #[cfg(feature = "debugger")]
    let control_c_connection = ClientConnection::new(&server_hostname, 5555)?;
    #[cfg(feature = "debugger")]
    let debug_client_connection = ClientConnection::new(&server_hostname, 5556)?;

    #[cfg(feature = "debugger")]
    if debugger {
        let debug_client = CliDebugClient::new(debug_client_connection);
        debug_client.event_loop_thread(); // TODO Broken
    }

    let runtime_client = CliRuntimeClient::new(
        flow_args,
        #[cfg(feature = "metrics")]
        matches.is_present("metrics"),
    );

    #[cfg(feature = "debugger")]
    runtime_client.event_loop(
        runtime_client_connection,
        control_c_connection,
        submission,
        debugger,
    )?;
    #[cfg(not(feature = "debugger"))]
    runtime_client.event_loop(runtime_connection, submission)?;

    Ok(())
}

/*
   Start a background thread that sends out beacons for server discovery by a client every second
*/
#[cfg(feature = "distributed")]
fn enable_server_discovery() -> Result<()> {
    // match BeaconSender::new(BEACON_PORT, FLOW_SERVICE_NAME) {
    //     Ok(beacon) => {
    //         info!(
    //             "Discovery beacon announcing service named '{}', on port: {}",
    //             FLOW_SERVICE_NAME, BEACON_PORT
    //         );
    //         std::thread::spawn(move || {
    //             let _ = beacon.send_loop(Duration::from_secs(1));
    //         });
    //     }
    //     Err(e) => bail!("Error starting discovery beacon: {}", e.to_string()),
    // }

    /*
        use simple_mdns::ServiceDiscovery;

        add_dns_responder();

        let mut discovery = ServiceDiscovery::new(FLOW_SERVICE_NAME, 60).expect("Invalid Service Name");
        let my_socket_address = "192.168.1.22:8090"
            .parse()
            .expect("Failed to parse socket address");
        discovery.add_socket_address(my_socket_address);
    */

    Ok(())
}

/*
fn add_dns_responder() {
    let mut responder = SimpleMdnsResponder::new(10);
    let srv_name = Name::new_unchecked(FLOW_SERVICE_NAME);

    responder.add_resource(ResourceRecord {
        class: CLASS::IN,
        name: srv_name.clone(),
        ttl: 10,
        rdata: RData::A(A {
            address: Ipv4Addr::LOCALHOST.into(),
        }),
    });

    responder.add_resource(ResourceRecord {
        class: CLASS::IN,
        name: srv_name.clone(),
        ttl: 10,
        rdata: RData::SRV(Box::new(SRV {
            port: 8080,
            priority: 0,
            weight: 0,
            target: srv_name,
        })),
    });
}
*/

/*
    try to discover a server that a client can send a submission to
*/
#[cfg(feature = "distributed")]
fn discover_server() -> Option<String> {
    // let listener = BeaconListener::new(BEACON_PORT, Some(FLOW_SERVICE_NAME.into())).ok()?;
    // let beacon = listener.wait(None).ok()?;
    // info!("'flowr' server discovered at IP: {}", beacon.source_ip);
    // Some(beacon.source_ip)

    /*
    let resolver = OneShotMdnsResolver::new().expect("Failed to create resolver");
    // querying for IP Address
    let answer = resolver
        .query_service_address(FLOW_SERVICE_NAME)
        .expect("Failed to query service address")?;

    info!("{:?}", answer);
    // IpV4Addr or IpV6Addr, depending on what was returned

    //    let answer = resolver
    //        .query_service_address_and_port("_flowr._tcp.local")
    //        .expect("Failed to query service address and port");
    //    println!("{:?}", answer);

    Some(answer.to_string())
     */

    None
}

/*
    Determine the number of threads to use to execute flows, with a default of the number of cores
    in the device, or any override from the command line.

    If debugger=true, then default to 0 threads, unless overridden by an argument
*/
fn num_threads(matches: &ArgMatches, debugger: bool) -> usize {
    if debugger {
        info!("Due to debugger option being set, number of threads has been forced to 1");
        return 1;
    }

    match matches.value_of("threads") {
        Some(value) => match value.parse::<i32>() {
            Ok(mut threads) => {
                if threads < 1 {
                    error!("Minimum number of additional threads is '1', so option has been overridden to be '1'");
                    threads = 1;
                }
                threads as usize
            }
            Err(_) => {
                error!("Error parsing the value for number of threads '{}'", value);
                num_cpus::get()
            }
        },
        None => num_cpus::get(),
    }
}

/*
    Determine the number of parallel jobs to be run in parallel based on a default of 2 times
    the number of cores in the device, or any override from the command line.
*/
fn num_parallel_jobs(matches: &ArgMatches, debugger: bool) -> usize {
    match matches.value_of("jobs") {
        Some(value) => match value.parse::<i32>() {
            Ok(mut jobs) => {
                if jobs < 1 {
                    error!("Minimum number of parallel jobs is '0', so option of '{}' has been overridden to be '1'",
                               jobs);
                    jobs = 1;
                }
                jobs as usize
            }
            Err(_) => {
                error!(
                    "Error parsing the value for number of parallel jobs '{}'",
                    value
                );
                2 * num_cpus::get()
            }
        },
        None => {
            if debugger {
                info!("Due to debugger option being set, max number of parallel jobs has defaulted to 1");
                1
            } else {
                2 * num_cpus::get()
            }
        }
    }
}

/*
    Parse the command line arguments using clap
*/
fn get_matches<'a>() -> ArgMatches<'a> {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .setting(AppSettings::TrailingVarArg)
        .version(env!("CARGO_PKG_VERSION"));

    let app = app.arg(Arg::with_name("jobs")
        .short("j")
        .long("jobs")
        .takes_value(true)
        .value_name("MAX_JOBS")
        .help("Set maximum number of jobs that can be running in parallel)"))
        .arg(Arg::with_name("lib_dir")
            .short("L")
            .long("libdir")
            .number_of_values(1)
            .multiple(true)
            .value_name("LIB_DIR|BASE_URL")
            .help("Add a directory or base Url to the Library Search path"))
        .arg(Arg::with_name("threads")
            .short("t")
            .long("threads")
            .takes_value(true)
            .value_name("THREADS")
            .help("Set number of threads to use to execute jobs (min: 1, default: cores available)"))
        .arg(Arg::with_name("verbosity")
            .short("v")
            .long("verbosity")
            .takes_value(true)
            .value_name("VERBOSITY_LEVEL")
            .help("Set verbosity level for output (trace, debug, info, warn, error (default))"))
        .arg(Arg::with_name("flow-manifest")
            .help("the file path of the 'flow' manifest file")
            .required(false)
            .index(1))
        .arg(Arg::with_name("flow-arguments")
            .multiple(true)
            .help("A list of arguments to pass to the flow when executed."));

    #[cfg(feature = "distributed")]
    let app = app.arg(
        Arg::with_name("server")
            .short("s")
            .long("server")
            .help("Launch as flowr server"),
    );

    #[cfg(feature = "distributed")]
    let app = app.arg(
        Arg::with_name("client")
            .short("c")
            .long("client")
            .conflicts_with("server")
            .help("Start flowr as a client to connect to a flowr server"),
    );

    #[cfg(feature = "distributed")]
    let app = app.arg(
        Arg::with_name("address")
            .short("a")
            .long("address")
            .takes_value(true)
            .value_name("ADDRESS")
            .conflicts_with("server")
            .help("The IP address of the flowr server to connect to"),
    );

    #[cfg(feature = "debugger")]
    let app = app.arg(
        Arg::with_name("debugger")
            .short("d")
            .long("debugger")
            .help("Enable the debugger when running a flow"),
    );

    #[cfg(feature = "metrics")]
    let app = app.arg(
        Arg::with_name("metrics")
            .short("m")
            .long("metrics")
            .help("Calculate metrics during flow execution and print them out when done"),
    );

    #[cfg(feature = "native")]
    let app = app.arg(
        Arg::with_name("native")
            .short("n")
            .long("native")
            .conflicts_with("client")
            .help("Use native libraries when possible"),
    );

    app.get_matches()
}

/*
    Parse the command line arguments passed onto the flow itself
*/
fn parse_flow_url(matches: &ArgMatches) -> Result<Url> {
    let cwd = env::current_dir().chain_err(|| "Could not get current working directory value")?;
    let cwd_url = Url::from_directory_path(cwd)
        .map_err(|_| "Could not form a Url for the current working directory")?;

    url_from_string(&cwd_url, matches.value_of("flow-manifest"))
        .chain_err(|| "Unable to parse the URL of the manifest of the flow to run")
}

/*
    Set environment variable with the args this will not be unique, but it will be used very
    soon and removed
*/
fn get_flow_args(matches: &ArgMatches, flow_manifest_url: &Url) -> Vec<String> {
    // arg #0 is the flow url
    let mut flow_args: Vec<String> = vec![flow_manifest_url.to_string()];

    // append any other arguments for the flow passed from the command line
    if let Some(args) = matches.values_of("flow-arguments") {
        flow_args.extend(args.map(|arg| arg.to_string()));
    }

    flow_args
}
