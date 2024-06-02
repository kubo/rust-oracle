// Rust-oracle - Rust binding for Oracle database
//
// URL: https://github.com/kubo/rust-oracle
//
//-----------------------------------------------------------------------------
// Copyright (c) 2017-2023 Kubo Takehiro <kubo@jiubao.org>. All rights reserved.
// This program is free software: you can modify it and/or redistribute it
// under the terms of:
//
// (i)  the Universal Permissive License v 1.0 or at your option, any
//      later version (http://oss.oracle.com/licenses/upl); and/or
//
// (ii) the Apache License v 2.0. (http://www.apache.org/licenses/LICENSE-2.0)
//-----------------------------------------------------------------------------

use crate::binding::*;
#[cfg(doc)]
use crate::pool::PoolBuilder;
use crate::util::{os_string_into_ansi_c_string, string_into_c_string};
#[cfg(doc)]
use crate::Connector;
use crate::DbError;
use crate::Error;
use crate::Result;
use once_cell::sync::OnceCell;
use std::ffi::{CString, OsString};
use std::mem::{self, MaybeUninit};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};

/// Parameters for explicit Oracle client library initialization
///
/// Note:
/// 1. Any method that invokes C functions in the Oracle client library will implicitly initialize it.
/// 2. Regardless of whether it is initialized explicitly or implicitly, it is only once per process.
///
/// # Examples
///
/// Initialize explicitly twice
///
/// ```
/// # use oracle::*;
/// // init() returns Ok(true) on the first call.
/// assert_eq!(InitParams::new().init()?, true);
///
/// // It returns Ok(false) when Oracle client library has initialized already.
/// assert_eq!(InitParams::new().init()?, false);
/// # Ok::<(), Error>(())
/// ```
///
/// Initialize implicitly then explicitly
///
/// ```
/// # use oracle::*;
/// // Oracle client library isn't initialzied at first.
/// assert_eq!(InitParams::is_initialized(), false);
///
/// // It is initialized by any method that invokes C functions in it.
/// Connection::connect("dummy", "dummy", "");
///
/// // Parameters have no effect on the already initialized one.
/// assert_eq!(
///     InitParams::new()
///         .oracle_client_lib_dir("/another/oracle/client/location")?
///         .init()?,
///     false
/// );
/// # Ok::<(), Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct InitParams {
    default_driver_name: Option<CString>,
    load_error_url: Option<CString>,
    oracle_client_lib_dir: Option<CString>,
    oracle_client_config_dir: Option<CString>,
    soda_use_json_desc: bool,
    use_json_id: bool,
}

impl InitParams {
    /// Creates a new initialization parameter
    pub fn new() -> InitParams {
        InitParams {
            default_driver_name: None,
            load_error_url: None,
            oracle_client_lib_dir: None,
            oracle_client_config_dir: None,
            soda_use_json_desc: false,
            use_json_id: false,
        }
    }

    /// Sets the default driver name to use when creating pools or standalone connections.
    ///
    /// The standard is to set this value to `"<name> : <version>"`, where `<name>`
    /// is the name of the driver and `<version>` is its version. There should be a
    /// single space character before and after the colon.
    ///
    /// This value is shown in database views that give information about
    /// connections. For example, it is in the `CLIENT_DRIVER` column
    /// of [`V$SESSION_CONNECT_INFO`].
    ///
    /// If this member isn't set, then the default value is `"rust-oracle : <version>"`,
    /// where `<version>` is the oracle crate version.
    ///
    /// This value is propagated to the default value of [`Connector::driver_name`]
    /// and [`PoolBuilder::driver_name`].
    ///
    /// # Errors
    ///
    /// If `name` contains null characters, an error will be returned.
    ///
    /// [`V$SESSION_CONNECT_INFO`]: https://www.oracle.com/pls/topic/lookup?ctx=dblatest&id=GUID-9F0DCAEA-A67E-4183-89E7-B1555DC591CE
    pub fn default_driver_name<T>(&mut self, name: T) -> Result<&mut InitParams>
    where
        T: Into<String>,
    {
        self.default_driver_name = Some(string_into_c_string(name.into(), "default_driver_name")?);
        Ok(self)
    }

    /// Sets the URL that should be provided in the error message returned
    /// when the Oracle Client library cannot be loaded.
    ///
    /// This URL should direct the user to the installation instructions for
    /// the application or driver using ODPI-C. If this value isn't set then
    /// the default ODPI-C URL is provided in the error message instead.
    ///
    /// # Errors
    ///
    /// If `url` contains null characters, an error will be returned.
    pub fn load_error_url<T>(&mut self, url: T) -> Result<&mut InitParams>
    where
        T: Into<String>,
    {
        self.load_error_url = Some(string_into_c_string(url.into(), "load_error_url")?);
        Ok(self)
    }

    /// Sets the location from which to load the Oracle Client library.
    ///
    /// If this value is set it is the only location that is searched;
    /// otherwise, if this value isn't set the Oracle Client library is
    /// searched for in the usual fashion as noted in [Oracle Client Library Loading][clientlibloading].
    /// Also see that section for limitations on using this.
    ///
    /// # Errors
    ///
    /// If `dir` contains null characters, an error will be returned.
    ///
    /// On windows, `dir` must consist with characters convertible to [ANSI code page],
    /// which is, for example, [CP1252] in English, [CP932] in Japanese.
    /// Otherwise, an error will be returned.
    ///
    /// [clientlibloading]: https://odpi-c.readthedocs.io/en/latest/user_guide/installation.html#oracle-client-library-loading
    /// [ANSI code page]: https://en.wikipedia.org/wiki/Windows_code_page#ANSI_code_page
    /// [CP1252]: https://en.wikipedia.org/wiki/Windows-1252
    /// [CP932]: https://en.wikipedia.org/wiki/Code_page_932_(Microsoft_Windows)
    pub fn oracle_client_lib_dir<T>(&mut self, dir: T) -> Result<&mut InitParams>
    where
        T: Into<OsString>,
    {
        self.oracle_client_lib_dir = Some(os_string_into_ansi_c_string(
            dir.into(),
            "oracle_client_lib_dir",
        )?);
        Ok(self)
    }

    /// Sets the location the Oracle client library will search for
    /// configuration files.
    ///
    /// This is equivalent to setting the environment variable `TNS_ADMIN`.
    /// If this value is set, it overrides any value set by the environment
    /// variable `TNS_ADMIN`.
    ///
    /// # Errors
    ///
    /// If `dir` contains null characters, an error will be returned.
    ///
    /// On windows, `dir` must consist with characters convertible to [ANSI code page],
    /// which is, for example, [CP1252] in English, [CP932] in Japanese.
    /// Otherwise, an error will be returned.
    ///
    /// [ANSI code page]: https://en.wikipedia.org/wiki/Windows_code_page#ANSI_code_page
    /// [CP1252]: https://en.wikipedia.org/wiki/Windows-1252
    /// [CP932]: https://en.wikipedia.org/wiki/Code_page_932_(Microsoft_Windows)
    pub fn oracle_client_config_dir<T>(&mut self, dir: T) -> Result<&mut InitParams>
    where
        T: Into<OsString>,
    {
        self.oracle_client_config_dir = Some(os_string_into_ansi_c_string(
            dir.into(),
            "oracle_client_config_dir",
        )?);
        Ok(self)
    }

    // SODA has not been supported yet.
    #[doc(hidden)]
    pub fn soda_use_json_desc(&mut self, value: bool) -> &mut InitParams {
        self.soda_use_json_desc = value;
        self
    }

    // JSON has not been supported yet.
    #[doc(hidden)]
    pub fn use_json_id(&mut self, value: bool) -> &mut InitParams {
        self.use_json_id = value;
        self
    }

    /// Initializes Oracle client library.
    ///
    /// It returns `Ok(true)` when Oracle client library hasn't been initialized
    /// yet and it is initialized successfully.
    ///
    /// It returns `Ok(false)` when Oracle client library has been initialized
    /// already. Parameter values in `self` affect nothing.
    ///
    /// Otherwise, it retruns an error.
    pub fn init(&self) -> Result<bool> {
        let mut initialized_here = false;
        GLOBAL_CONTEXT.get_or_try_init(|| {
            let mut params = unsafe { mem::zeroed::<dpiContextCreateParams>() };
            if let Some(ref name) = self.default_driver_name {
                params.defaultDriverName = name.as_ptr();
            }
            if let Some(ref url) = self.load_error_url {
                params.loadErrorUrl = url.as_ptr();
            }
            if let Some(ref dir) = self.oracle_client_lib_dir {
                params.oracleClientLibDir = dir.as_ptr()
            }
            if let Some(ref dir) = self.oracle_client_config_dir {
                params.oracleClientConfigDir = dir.as_ptr()
            }
            params.sodaUseJsonDesc = self.soda_use_json_desc.into();
            params.useJsonId = self.use_json_id.into();
            let result = Context::from_params(&mut params);
            initialized_here = true;
            result
        })?;
        Ok(initialized_here)
    }

    /// Returns `true` if Oracle client library has initialized already.
    ///
    /// # Examples
    ///
    /// ```
    /// # use oracle::*;
    ///
    /// // `false` at first
    /// assert_eq!(InitParams::is_initialized(), false);
    ///
    /// // Use any method that invokes C functions in the Oracle client library.
    /// Connection::connect("dummy", "dummy", "");
    ///
    /// // `true` here
    /// assert_eq!(InitParams::is_initialized(), true);
    /// # Ok::<(), Error>(())
    /// ```
    pub fn is_initialized() -> bool {
        GLOBAL_CONTEXT.get().is_some()
    }
}

//
// Context
//

// Context is created for each connection.
// On the other hand, the context member (*mut dpiContext) is created only once in the process.
//
// It is used to share information between Connection and structs created from the Connection.
#[derive(Clone)]
pub(crate) struct Context {
    pub context: *mut dpiContext,
    last_warning: Option<Arc<Mutex<Option<DbError>>>>,
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

static GLOBAL_CONTEXT: OnceCell<Context> = OnceCell::new();

impl Context {
    // Use this only inside of GLOBAL_CONTEXT.get_or_try_init().
    fn from_params(params: &mut dpiContextCreateParams) -> Result<Context> {
        if params.defaultDriverName.is_null() {
            let driver_name: &'static str =
                concat!("rust-oracle : ", env!("CARGO_PKG_VERSION"), "\0");
            params.defaultDriverName = driver_name.as_ptr() as *const c_char;
        }
        let mut ctxt = ptr::null_mut();
        let mut err = MaybeUninit::uninit();
        if unsafe {
            dpiContext_createWithParams(
                DPI_MAJOR_VERSION,
                DPI_MINOR_VERSION,
                params,
                &mut ctxt,
                err.as_mut_ptr(),
            )
        } == DPI_SUCCESS as i32
        {
            Ok(Context {
                context: ctxt,
                last_warning: None,
            })
        } else {
            Err(Error::from_dpi_error(&unsafe { err.assume_init() }))
        }
    }

    pub fn new0() -> Result<Context> {
        Ok(GLOBAL_CONTEXT
            .get_or_try_init(|| {
                let mut params = unsafe { mem::zeroed() };
                Context::from_params(&mut params)
            })?
            .clone())
    }

    pub fn new() -> Result<Context> {
        let ctxt = Context::new0()?;
        Ok(Context {
            last_warning: Some(Arc::new(Mutex::new(None))),
            ..ctxt
        })
    }

    // called by Connection::last_warning
    pub fn last_warning(&self) -> Option<DbError> {
        self.last_warning
            .as_ref()
            .and_then(|mutex| mutex.lock().unwrap().as_ref().cloned())
    }

    // called by Connection, Statement, Batch and Pool to set a warning
    // referred by `Connection::last_warning` later.
    pub fn set_warning(&self) {
        if let Some(ref mutex) = self.last_warning {
            *mutex.lock().unwrap() = DbError::to_warning(self);
        }
    }

    pub fn common_create_params(&self) -> dpiCommonCreateParams {
        let mut params = MaybeUninit::uninit();
        unsafe {
            dpiContext_initCommonCreateParams(self.context, params.as_mut_ptr());
            let mut params = params.assume_init();
            params.createMode |= DPI_MODE_CREATE_THREADED;
            params
        }
    }

    pub fn conn_create_params(&self) -> dpiConnCreateParams {
        let mut params = MaybeUninit::uninit();
        unsafe {
            dpiContext_initConnCreateParams(self.context, params.as_mut_ptr());
            params.assume_init()
        }
    }

    pub fn pool_create_params(&self) -> dpiPoolCreateParams {
        let mut params = MaybeUninit::uninit();
        unsafe {
            dpiContext_initPoolCreateParams(self.context, params.as_mut_ptr());
            params.assume_init()
        }
    }
}
