window.SIDEBAR_ITEMS = {"constant":[["DONT_RUN_AGAIN","Use `DONT_RUN_AGAIN` to indicate that a function should not be executed more times"],["RUN_AGAIN","Use `RUN_AGAIN` to indicate that a function can be executed more times"]],"mod":[["content","contains the file and http content provider implementations Content provider trait. It defines methods for for getting content of flows from files, http or library references."],["deserializers","a set of serializers to read definition files from various text formats based on file extension deserializer modules provides a number of deserializers from different formats and also help methods to get a deserializer based on the file extension of a file referred to by a Url"],["errors","contains [errors::Error] that other modules in this crate will `use errors::*;` to get access to everything `error_chain` creates."],["meta_provider","is used to resolve library references of the type “lib://” and “context://” using lib search path"],["model","defines many of the core data structures used across libraries and binaries `model` module defines a number of core data structures that are used across the compiler and the runtime and macros."],["provider","is a trait definition that providers of content must implement"],["url_helper","Utility functions related to [Urls][url::Url]"]],"trait":[["Implementation","A function’s implementation must implement this trait with a single `run()` method that takes as input an array of values and it returns a `Result` tuple with an Optional output `Value` plus a [RunAgain] indicating if it should be run again. i.e. it has not “completed”, in which case it should not be called again."]],"type":[["RunAgain","Implementations should return a value of type `RunAgain` to indicate if it should be executed more times in the future."]]};