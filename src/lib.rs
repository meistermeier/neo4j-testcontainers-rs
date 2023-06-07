#![doc = include_str!("../doc/lib.md")]
#![warn(clippy::all, clippy::nursery)]
#![allow(clippy::cargo, clippy::pedantic)]
#![warn(
    bad_style,
    dead_code,
    improper_ctypes,
    missing_copy_implementations,
    missing_debug_implementations,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2021_incompatible_or_patterns,
    rust_2021_incompatible_closure_captures,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unsafe_op_in_unsafe_fn,
    unused_allocation,
    unused_comparisons,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    unused,
    while_true
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::bool_assert_comparison,
    clippy::missing_const_for_fn
)]

use std::collections::HashMap;
use testcontainers::{core::WaitFor, Container, Image};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Neo4j {
    version: String,
    user: String,
    pass: String,
    env_vars: HashMap<String, String>,
}

#[doc = include_str!("../doc/lib.md")]
impl Neo4j {
    /// Create a new instance of a Neo4j 5 image with the default user and password.
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(None, None, None)
    }

    // Create a new instance of a Neo4j image of the given version with the default user and password.
    #[must_use]
    pub fn from_version(version: &str) -> Self {
        Self::new(None, None, Some(version.to_owned()))
    }

    /// Create a new instance of a Neo4j image with the version and given user and password.
    #[must_use]
    pub fn from_auth_and_version(version: &str, user: &str, pass: &str) -> Self {
        Self::new(
            Some(user.to_owned()),
            Some(pass.to_owned()),
            Some(version.to_owned()),
        )
    }

    fn new(user: Option<String>, pass: Option<String>, version: Option<String>) -> Self {
        const USER_VAR: &str = "NEO4J_TEST_USER";
        const PASS_VAR: &str = "NEO4J_TEST_PASS";
        const VERSION_VAR: &str = "NEO4J_VERSION_TAG";

        const DEFAULT_USER: &str = "neo4j";
        const DEFAULT_PASS: &str = "neo";
        const DEFAULT_VERSION_TAG: &str = "5";

        use std::env::var;

        let user = user
            .or_else(|| var(USER_VAR).ok())
            .unwrap_or_else(|| DEFAULT_USER.to_owned());
        let pass = pass
            .or_else(|| var(PASS_VAR).ok())
            .unwrap_or_else(|| DEFAULT_PASS.to_owned());
        let version = version
            .or_else(|| var(VERSION_VAR).ok())
            .unwrap_or_else(|| DEFAULT_VERSION_TAG.to_owned());

        let mut env_vars = HashMap::new();
        env_vars.insert("NEO4J_AUTH".to_owned(), format!("{}/{}", user, pass));

        if pass.len() < 8 {
            env_vars.insert(
                "NEO4J_dbms_security_auth__minimum__password__length".to_owned(),
                pass.len().to_string(),
            );
        }

        Self {
            version,
            user,
            pass,
            env_vars,
        }
    }

    // Return the version of the Neo4j image.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    // Return the user of the Neo4j server.
    #[must_use]
    pub fn user(&self) -> &str {
        &self.user
    }

    // Return the password of the Neo4j server.
    #[must_use]
    pub fn pass(&self) -> &str {
        &self.pass
    }

    // Return the connection URI to connect to the Neo4j server over IPv4.
    #[must_use]
    pub fn uri_ipv4(container: &Container<'_, Self>) -> String {
        let bolt_port = container
            .ports()
            .map_to_host_port_ipv4(7687)
            .expect("Image exposes 7687 by default");
        format!("bolt://127.0.0.1:{}", bolt_port)
    }

    // Return the connection URI to connect to the Neo4j server over IPv6.
    #[must_use]
    pub fn uri_ipv6(container: &Container<'_, Self>) -> String {
        let bolt_port = container
            .ports()
            .map_to_host_port_ipv6(7687)
            .expect("Image exposes 7687 by default");
        format!("bolt://[::1]:{}", bolt_port)
    }
}

impl Default for Neo4j {
    fn default() -> Self {
        Self::from_env()
    }
}

impl Image for Neo4j {
    type Args = ();

    fn name(&self) -> String {
        "neo4j".to_owned()
    }

    fn tag(&self) -> String {
        self.version.clone()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stdout("Bolt enabled on"),
            WaitFor::message_on_stdout("Started."),
        ]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}