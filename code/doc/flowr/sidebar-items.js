window.SIDEBAR_ITEMS = {"enum":[["Mode","Mode of execution of the [Coordinator][flowrlib::coordinator::Coordinator] of flow execution"]],"fn":[["client","Start the clients that talks to the server thread or process"],["client_and_server","Start a Server by running a [Coordinator][flowrlib::coordinator::Coordinator] in a background thread, then start clients in the calling thread"],["client_only","Start only a client in the calling thread. Since we are only starting a client in this process, we don’t have server information, so we create a set of ServerInfo from command line options for the server address and known service names and ports."],["get_bind_addresses","Return addresses to bind to for"],["get_connect_addresses","Return addresses and ports to be used for each of the three queues"],["get_flow_args","Set environment variable with the args this will not be unique, but it will be used very soon and removed"],["get_four_ports","Return four free ports to use for client-server message queues"],["get_matches","Parse the command line arguments using clap"],["main","Main for flowr binary - call `run()` and print any error that results or exit silently if OK"],["num_threads","Determine the number of threads to use to execute flows"],["parse_flow_url","Parse the command line arguments passed onto the flow itself"],["run","Run `flowr`. After setting up logging and parsing the command line arguments invoke `flowrlib` and return any errors found."],["server","Create a new `Coordinator`, pre-load any libraries in native format that we want to have before loading a flow and it’s library references, then enter the `submission_loop()` accepting and executing flows submitted for execution, executing each one using the `Coordinator`"],["server_only","Start just a server - by running a Coordinator in the calling thread."],["set_lib_search_path","For the lib provider, libraries maybe installed in multiple places in the file system. In order to find the content, a FLOW_LIB_PATH environment variable can be configured with a list of directories in which to look for the library in question."]],"mod":[["cli","provides the `context functions` for interacting with the execution environment from a flow, plus client-server implementations of [flowrlib::protocols] for executing them on different threads from the [Coordinator][flowrlib::coordinator::Coordinator]"],["errors","provides [Error][errors::Error] that other modules in this crate will `use crate::errors::*;` to get access to everything `error_chain` creates."]]};