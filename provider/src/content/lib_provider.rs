use std::env;

use simpath::Simpath;
use url::Url;

use flowrlib::errors::*;
use flowrlib::provider::Provider;

pub struct LibProvider;

/*
    Urls for library flows and functions and values will be of the form:
        "lib://flowstdlib/stdio/stdout.toml"

    Where 'flowstdlib' is the library name and 'stdio/stdout.toml' the path of the definition
    file within the library.

    For the lib provider, libraries maybe installed in multiple places in the file system.
    In order to find the content, a FLOW_LIB_PATH environment variable can be configured with a
    list of directories in which to look for the library in question.

    Once the library in question is found in the file system, then a "file:" Url is constructed
    that refers to the actual content, and this is returned.

    As the scheme of this Url is "file:" then a different content provider will be used to actually
    provide the content. Hence the "get" method for this provider is not imlemented and should
    never be called.
*/
impl Provider for LibProvider {
    /*
        Take the "lib:" Url (such as "lib://runtime/stdio/stdout") and extract the library
         name ("runtime")

        Using the "FLOW_LIB_PATH" environment variable attempt to locate the library's root folder
        in the file system.

        If located, then construct a PathBuf to refer to the definition file:
            - either "stdio/stdout.toml" or
            - "stdio/stdout/stdout.toml"

        within the library (using knowledge of library file structure).

        If the file exists, then create a "file:" Url that points to the file, for the file provider
        to use later to read the content.

        Also, construct a string that is a reference to that module in the library, such as:
            "runtime/stdio/stdout" and return that also.
    */
    fn resolve(&self, url_str: &str, default_filename: &str) -> Result<(String, Option<String>)> {
        let url = Url::parse(url_str)
            .chain_err(|| format!("Could not convert '{}' to valid Url", url_str))?;
        let lib_name = url.host_str().expect(
            &format!("'lib_name' could not be extracted from host part of url '{}'", url));

        if let Err(_) = env::var("FLOW_LIB_PATH") {
            let parent_dir = std::env::current_dir().unwrap();
            debug!("Setting 'FLOW_LIB_PATH' to '{}'", parent_dir.to_string_lossy().to_string());
            env::set_var("FLOW_LIB_PATH", parent_dir.to_string_lossy().to_string());
        }

        let flow_lib_search_path = Simpath::new("FLOW_LIB_PATH");
        let mut lib_path = flow_lib_search_path.find(lib_name)
            .chain_err(|| format!("Could not find lib named '{}' in FLOW_LIB_PATH", lib_name))?;
        lib_path.push(&url.path()[1..]);

        // Drop the file extension off the lib definition file path to get a lib reference
        let module = url.join("./").unwrap().join(lib_path.file_stem().unwrap().to_str().unwrap());
        let lib_ref = format!("{}{}", lib_name, module.unwrap().path());

        // See if the directory with that name exists
        if lib_path.exists() {
            if !lib_path.is_dir() {
                // It's a file so just return the path
                let lib_path_url = Url::from_file_path(&lib_path)
                    .map_err(|_| format!("Could not create Url from '{:?}'", &lib_path))?;
                return Ok((lib_path_url.to_string(), Some(lib_ref.to_string())));
            }

            debug!("'{:?}' is a directory, so looking for default file name '{}'", lib_path, default_filename);
            let mut default_path = lib_path.clone();
            default_path.push(default_filename);
            if default_path.exists() {
                let default_path_url = Url::from_file_path(&default_path)
                    .map_err(|_| format!("Could not create Url from '{:?}'", &default_path))?;
                return Ok((default_path_url.to_string(), Some(lib_ref.to_string())));
            }

            // This could be for a provided implementation, so look for a file named the same
            // as the directory, with a toml extension
            let filename = lib_path.file_name().unwrap().to_str().unwrap();
            let mut filename_path = lib_path.clone();
            filename_path.push(filename);
            filename_path.set_extension("toml");

            if filename_path.exists() {
                let file_path_url = Url::from_file_path(&filename_path)
                    .map_err(|_| format!("Could not create Url from '{:?}'", &filename_path))?;
                return Ok((file_path_url.to_string(), Some(lib_ref.to_string())));
            }
            bail!("Could not locate url '{}' in libraries in 'FLOW_LIB_PATH'", url)
        } else {
            // See if the file, with a .toml extension exists
            lib_path.set_extension("toml");
            if lib_path.exists() {
                let lib_path_url = Url::from_file_path(&lib_path)
                    .map_err(|_| format!("Could not create Url from '{:?}'", &lib_path))?;
                return Ok((lib_path_url.to_string(), Some(lib_ref.to_string())));
            }
            bail!("Could not locate url '{}' in libraries in 'FLOW_LIB_PATH'", url)
        }
    }

    // All Urls that start with "lib://" should resource to a different Url with "http(s)" or "file"
    // and so we should never get a request to get content from a Url with such a scheme
    fn get(&self, _url: &str) -> Result<Vec<u8>> {
        unimplemented!();
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use flowrlib::provider::Provider;

    use super::LibProvider;

    #[test]
    fn resolve_path() {
        let provider: &dyn Provider = &LibProvider;
        let mut root = env::current_dir().unwrap();
        root.pop();
        let root_str: String = root.as_os_str().to_str().unwrap().to_string();
        env::set_var("FLOW_LIB_PATH", &root_str);
        let lib_url = "lib://flowstdlib/control/tap";
        match provider.resolve(&lib_url, "".into()) {
            Ok((url, lib_ref)) => {
                assert_eq!(url, format!("file://{}/flowstdlib/control/tap", root_str));
                assert_eq!(lib_ref, Some("flowstdlib/control/tap".to_string()));
            }
            Err(e) => assert!(false, e.to_string())
        }
    }
}
