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
use crate::error;
use crate::DbError;
use crate::Result;
use lazy_static::lazy_static;
use std::mem::MaybeUninit;
use std::os::raw::c_char;
use std::ptr;
use std::sync::{Arc, Mutex};

//
// Context
//

#[derive(Clone)]
pub(crate) struct Context {
    pub context: *mut dpiContext,
    last_warning: Option<Arc<Mutex<Option<DbError>>>>,
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

enum ContextResult {
    Ok(Context),
    Err(dpiErrorInfo),
}

unsafe impl Sync for ContextResult {}
unsafe impl Send for ContextResult {}

lazy_static! {
    static ref DPI_CONTEXT: ContextResult = {
        let mut ctxt = ptr::null_mut();
        let mut err = MaybeUninit::uninit();
        if unsafe {
            dpiContext_createWithParams(
                DPI_MAJOR_VERSION,
                DPI_MINOR_VERSION,
                ptr::null_mut(),
                &mut ctxt,
                err.as_mut_ptr(),
            )
        } == DPI_SUCCESS as i32
        {
            ContextResult::Ok(Context {
                context: ctxt,
                last_warning: None,
            })
        } else {
            ContextResult::Err(unsafe { err.assume_init() })
        }
    };
}

impl Context {
    pub fn new0() -> Result<Context> {
        match *DPI_CONTEXT {
            ContextResult::Ok(ref ctxt) => Ok(ctxt.clone()),
            ContextResult::Err(ref err) => Err(error::error_from_dpi_error(err)),
        }
    }

    pub fn new() -> Result<Context> {
        let ctxt = Context::new0()?;
        Ok(Context {
            last_warning: Some(Arc::new(Mutex::new(None))),
            ..ctxt
        })
    }

    pub fn last_warning(&self) -> Option<DbError> {
        self.last_warning
            .as_ref()
            .and_then(|mutex| mutex.lock().unwrap().as_ref().cloned())
    }

    pub fn set_warning(&self) {
        if let Some(ref mutex) = self.last_warning {
            *mutex.lock().unwrap() = error::warning(self);
        }
    }

    pub fn common_create_params(&self) -> dpiCommonCreateParams {
        let mut params = MaybeUninit::uninit();
        unsafe {
            dpiContext_initCommonCreateParams(self.context, params.as_mut_ptr());
            let mut params = params.assume_init();
            let driver_name: &'static str = concat!("rust-oracle : ", env!("CARGO_PKG_VERSION"));
            params.createMode |= DPI_MODE_CREATE_THREADED;
            params.driverName = driver_name.as_ptr() as *const c_char;
            params.driverNameLength = driver_name.len() as u32;
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
