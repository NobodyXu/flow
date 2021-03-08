//! The Provider trait must be implemented by Providers of content to the compiler or runtime
use simpath::{FoundType, Simpath};
use url::Url;

use crate::content::file_provider::FileProvider;
use crate::content::http_provider::HttpProvider;
use crate::errors::*;

/// A content provider is responsible with interfacing with the environment and doing IO
/// or what is required to supply content related with flows - isolating other libraries
/// from the File SSystem or IO. It must implement the `Provider` trait
pub trait Provider {
    /// Take a URL and uses it to determine a url where actual content can be read from
    /// using some provider specific logic. This may involve looking for default files in a
    /// directory (a file provider) or a server path (an http provider), or it may involve
    /// translating a virtual URL into a real on where content can be found (lib provider).
    /// It also returns an optional String which is a library reference in case that applies.
    fn resolve_url(&self, url: &str, default_file: &str, extensions: &[&str]) -> Result<(String, Option<String>)>;

    /// Fetches content from a URL. It resolves the URL internally before attempting to
    /// fetch actual content
    fn get_contents(&self, url: &str) -> Result<Vec<u8>>;
}

const FILE_PROVIDER: &dyn Provider = &FileProvider as &dyn Provider;
const HTTP_PROVIDER: &dyn Provider = &HttpProvider as &dyn Provider;

/// The `MetaProvider` implements the `Provider` trait and based on the url and it's
/// resolution to a real location for content invokes one of the child providers it has
/// to fetch the content (e.g. File or Http).
pub struct MetaProvider {
    lib_search_path: Simpath
}

/// Instantiate MetaProvider and then use the Provider trait methods on it to resolve and fetch
/// content depending on the URL scheme.
/// ```
/// use provider::content::provider::{Provider, MetaProvider};
/// use simpath::Simpath;
/// let lib_search_path = Simpath::new_with_separator("FLOW_LIB_PATH", ',');
/// let meta_provider = &MetaProvider::new(lib_search_path) as &dyn Provider;
/// let url = "file://directory";
/// match meta_provider.resolve_url(url, "default", &["toml"]) {
///     Ok((resolved_url, lib_ref)) => {
///         match meta_provider.get_contents(&resolved_url) {
///             Ok(contents) => println!("Content: {:?}", contents),
///             Err(e) => println!("Got error '{}'", e)
///         }
///     }
///     Err(e) => {
///         println!("Found error '{}'", e);
///     }
/// };
/// ```
impl MetaProvider {
    pub fn new(lib_search_path: Simpath) -> Self {
        MetaProvider {
            lib_search_path
        }
    }

    // Determine which specific provider should be used based on the scheme of the Url of the content
    fn get_provider(&self, scheme: &str) -> Result<&dyn Provider> {
        match scheme {
            "file" => Ok(FILE_PROVIDER),
            "http" | "https" => Ok(HTTP_PROVIDER),
            _ => bail!("Cannot determine which provider to use for url with scheme: '{}'", scheme)
        }
    }

    /// Urls for library flows and functions and values will be of the form:
    ///        "lib://flowstdlib/stdio/stdout.toml"
    ///
    ///    Where 'flowstdlib' is the library name and 'stdio/stdout.toml' the path of the definition
    ///    file within the library.
    ///
    ///   Find library in question is found in the file system or via Http using the provider's
    ///   search path (setup on provider creation).
    ///
    ///   Then return:
    ///    - a string representation of the Url (file: or http: or https:) where the file can be found
    ///    - a string that is a reference to that module in the library, such as:
    ///        "flowruntime/stdio/stdout/stdout"
    fn resolve_lib_url(&self, url: Url) -> Result<(Url, Option<String>)> {
        let lib_name = url.host_str()
            .chain_err(|| format!("'lib_name' could not be extracted from the url '{}'", url))?;
        let path_under_lib = url.path().trim_start_matches('/');
        let lib_reference = Some(format!("{}/{}", lib_name, path_under_lib));

        match self.lib_search_path.find(lib_name) {
            Ok(FoundType::File(lib_root_path)) => {
                let lib_path = lib_root_path.join(path_under_lib);
                Ok((Url::from_directory_path(lib_path)
                        .map_err(|_| "Could not convert file: lib_path to Url")?, lib_reference))
            }
            Ok(FoundType::Resource(mut lib_root_url)) => {
                lib_root_url.set_path(&format!("{}/{}", lib_root_url.path(), path_under_lib));
                Ok((lib_root_url, lib_reference))
            }
            _ => bail!("Could not resolve library Url '{}' using library search path", url)
        }
    }
}

impl Provider for MetaProvider {
    /// Takes a Url with a scheme of "http", "https", "file", or "lib" and determine where the content
    /// should be loaded from.
    ///
    /// Url could refer to:
    ///     -  a specific file or flow (that may or may not exist)
    ///     -  a directory - if exists then look for a provider specific default file
    ///     -  a file in a library, transform the reference into a Url where the content can be found
    fn resolve_url(&self, url_str: &str, default_filename: &str, extensions: &[&str]) -> Result<(String, Option<String>)> {
        let mut url = Url::parse(url_str)
            .chain_err(|| format!("Could not convert '{}' to valid Url", url_str))?;
        let scheme = url.scheme().to_string();
        let mut lib_reference = None;

        // resolve a lib reference into either a file: or http: or https: reference
        if scheme == "lib" {
            let resolution = self.resolve_lib_url(url.clone())?;
            url = resolution.0;
            lib_reference = resolution.1;
        }

        let provider = self.get_provider(&url.scheme())?;
        let (resolved_url, _) = provider.resolve_url(&url.to_string(), default_filename, extensions)?;

        Ok((resolved_url, lib_reference))
    }

    /// Takes a Url with a scheme of "http", "https" or "file". Read and return the contents of the
    /// resource at that Url.
    fn get_contents(&self, url_str: &str) -> Result<Vec<u8>> {
        let url = Url::parse(url_str)
            .chain_err(|| format!("Could not convert '{}' to valid Url", url_str))?;
        let scheme = url.scheme().to_string();

        let provider = self.get_provider(&scheme)?;
        let content = provider.get_contents(&url_str)?;
        Ok(content)
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use simpath::Simpath;
    use url::Url;

    use crate::content::provider::{MetaProvider, Provider};

    #[test]
    fn get_invalid_provider() {
        let search_path = Simpath::new("TEST");
        let meta = MetaProvider::new(search_path);

        assert!(meta.get_provider("fake://bla").is_err());
    }

    #[test]
    fn get_http_provider() {
        let search_path = Simpath::new("TEST");
        let meta = MetaProvider::new(search_path);

        assert!(meta.get_provider("http").is_ok());
    }

    #[test]
    fn get_https_provider() {
        let search_path = Simpath::new("TEST");
        let meta = MetaProvider::new(search_path);

        assert!(meta.get_provider("https").is_ok());
    }

    #[test]
    fn get_file_provider() {
        let search_path = Simpath::new("TEST");
        let meta = MetaProvider::new(search_path);

        assert!(meta.get_provider("file").is_ok());
    }

    fn set_lib_search_path() -> Simpath {
        let mut lib_search_path = Simpath::new("lib_search_path");
        let root_str = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("Could not get project root dir");
        lib_search_path.add_directory(root_str.to_str().expect("Could not get root path as string"));
        println!("Lib search path set to '{}'", lib_search_path);
        lib_search_path
    }

    #[test]
    fn resolve_path() {
        let root_str = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("Could not get project root dir");
        let provider: &dyn Provider = &MetaProvider::new(set_lib_search_path());
        let lib_url = "lib://flowstdlib/control/tap";
        match provider.resolve_url(&lib_url, "", &["toml"]) {
            Ok((url, lib_ref)) => {
                assert_eq!(url, format!("file://{}/flowstdlib/control/tap/tap.toml", root_str.display().to_string()));
                assert_eq!(lib_ref, Some("flowstdlib/control/tap".to_string()));
            }
            Err(e) => panic!(e.to_string())
        }
    }

    #[test]
    fn resolve_web_path() {
        let mut search_path = Simpath::new("web_path");
        // `flowstdlib` can be found under the root of the project at `tree/master` on github
        search_path.add_url(&Url::parse(&format!("{}{}", env!("CARGO_PKG_REPOSITORY"), "tree/master/flowstdlib"))
            .expect("Could not parse the url for Simpath"));

        let provider: &dyn Provider = &MetaProvider::new(search_path);

        let lib_url = "lib://flowstdlib/control/tap/tap.toml";
        let resolved_url = provider.resolve_url(&lib_url, "", &["toml"])
            .expect("Couldn't resolve library on the web").0;
        assert_eq!(resolved_url, format!("{}{}", env!("CARGO_PKG_REPOSITORY"),
                                         "tree/master/flowstdlib/control/tap/tap.toml"));
    }
}