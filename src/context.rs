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
use crate::DbError;
use crate::Error;
use crate::Result;
use once_cell::sync::OnceCell;
use std::mem::{self, MaybeUninit};
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};

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
