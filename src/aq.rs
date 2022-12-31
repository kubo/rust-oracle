//! Oracle Advanced Queuing (available when `aq_unstable` feature is enabled.)
//!
//! **Warning:** Any type in this module is unstable. It may be changed incompatibly by minor version upgrades.
//!
//! # Examples
//!
//! ## Object type queue
//!
//! ```
//! # use oracle::Error;
//! # use oracle::test_util;
//! # use oracle::aq;
//! # use oracle::sql_type::Object;
//! # let conn = test_util::connect()?;
//!
//! // Create a queue
//! let objtype = conn.object_type("UDT_BOOK")?;
//! let mut queue = aq::Queue::<Object>::new(&conn, "BOOK_QUEUE", &objtype)?;
//!
//! // Create a message
//! let mut payload = objtype.new_object()?;
//! payload.set("TITLE", &"Pebble in the Sky")?;
//! payload.set("AUTHORS", &"Isaac Asimov")?;
//! payload.set("PRICE", &17.0)?;
//! let mut msg = aq::MsgProps::<Object>::new(&conn)?;
//! msg.set_payload(&payload);
//!
//! // Enqueue the message to the queue
//! queue.enqueue(&msg)?;
//!
//! // Dequeue a message from the queue
//! let new_msg = queue.dequeue()?;
//! let new_payload = new_msg.payload()?;
//!
//! // Compare message payloads.
//! assert_eq!(payload.get::<String>("TITLE")?, new_payload.get::<String>("TITLE")?);
//! assert_eq!(payload.get::<String>("AUTHORS")?, new_payload.get::<String>("AUTHORS")?);
//! assert_eq!(payload.get::<f32>("PRICE")?, new_payload.get::<f32>("PRICE")?);
//! # Ok::<(), Error>(())
//! ```
//!
//! ## RAW data queue
//!
//! ```
//! # use oracle::Error;
//! # use oracle::test_util;
//! # use oracle::aq;
//! # let conn = test_util::connect()?;
//!
//! // Create a queue
//! let mut queue = aq::Queue::<[u8]>::new(&conn, "RAW_QUEUE", &())?;
//!
//! // Create a message
//! let payload = b"\xde\xad\xbe\xef";
//! let mut msg = aq::MsgProps::<[u8]>::new(&conn)?;
//! msg.set_payload(payload.as_ref());
//!
//! // Enqueue the message to the queue
//! queue.enqueue(&msg)?;
//!
//! // Dequeue a message from the queue
//! let new_msg = queue.dequeue()?;
//! let new_payload = new_msg.payload()?; // returns Vec<u8>
//!
//! // Compare message payloads.
//! assert_eq!(payload, new_payload.as_slice());
//! # Ok::<(), Error>(())
//! ```
//!
//! # Enqueue and dequeue more than one message in one call
//!
//! ```
//! # use oracle::Error;
//! # use oracle::test_util;
//! # use oracle::aq;
//! # let conn = test_util::connect()?;
//!
//! // Create a queue
//! let mut queue = aq::Queue::<[u8]>::new(&conn, "RAW_QUEUE", &())?;
//!
//! // Create messages
//! let payloads = [b"\xde\xad\xbe\xef", b"\xba\xce\xba\x11"];
//! let mut messages = vec![];
//! for payload in &payloads {
//!     let mut msg = aq::MsgProps::<[u8]>::new(&conn)?;
//!     msg.set_payload(payload.as_ref())?;
//!     messages.push(msg);
//! }
//!
//! // Enqueue the messages
//! queue.enqueue_many(&messages)?;
//!
//! // Dequeue messages from the queue
//! let new_messages = queue.dequeue_many(10)?;
//!
//! // Compare message payloads.
//! assert_eq!(new_messages.len(), 2);
//! assert_eq!(new_messages[0].payload()?, payloads[0]);
//! assert_eq!(new_messages[1].payload()?, payloads[1]);
//! # Ok::<(), Error>(())
//! ```

use crate::binding::*;
use crate::chkerr;
use crate::connection::Conn;
use crate::new_odpi_str;
use crate::sql_type::Object;
use crate::sql_type::ObjectType;
use crate::sql_type::OracleType;
use crate::sql_type::Timestamp;
use crate::to_odpi_str;
use crate::to_rust_slice;
use crate::Connection;
use crate::Context;
use crate::DpiMsgProps;
use crate::DpiQueue;
use crate::Error;
use crate::Result;
use std::borrow::ToOwned;
use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr;
use std::time::Duration;

/// A trait for payload type
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
pub trait Payload: ToOwned {
    type TypeInfo;
    fn payload_type(payload_type: &Self::TypeInfo) -> Result<Option<ObjectType>>;
    fn get(props: &MsgProps<Self>) -> Result<Self::Owned>;
    fn set(&self, props: &mut MsgProps<Self>) -> Result<()>;
}

impl Payload for [u8] {
    type TypeInfo = ();

    fn payload_type(_payload_type: &Self::TypeInfo) -> Result<Option<ObjectType>> {
        Ok(None)
    }

    fn get(props: &MsgProps<Self>) -> Result<Vec<u8>> {
        let mut ptr = ptr::null();
        let mut len = 0;
        chkerr!(
            props.ctxt(),
            dpiMsgProps_getPayload(props.handle.raw, ptr::null_mut(), &mut ptr, &mut len)
        );
        Ok(to_rust_slice(ptr, len).to_vec())
    }

    fn set(&self, props: &mut MsgProps<Self>) -> Result<()> {
        chkerr!(
            props.ctxt(),
            dpiMsgProps_setPayloadBytes(
                props.handle.raw,
                self.as_ptr() as *const c_char,
                self.len() as u32
            )
        );
        props.payload_type = None;
        Ok(())
    }
}

impl Payload for Object {
    type TypeInfo = ObjectType;

    fn payload_type(payload_type: &Self::TypeInfo) -> Result<Option<ObjectType>> {
        Ok(Some(payload_type.clone()))
    }

    fn get(props: &MsgProps<Self>) -> Result<Object> {
        let objtype = props.payload_type.as_ref().ok_or(Error::NoDataFound)?;
        let mut obj_handle = ptr::null_mut();
        chkerr!(
            props.ctxt(),
            dpiMsgProps_getPayload(
                props.handle.raw,
                &mut obj_handle,
                ptr::null_mut(),
                ptr::null_mut()
            )
        );
        Ok(Object::new(props.conn.clone(), obj_handle, objtype.clone()))
    }

    fn set(&self, props: &mut MsgProps<Self>) -> Result<()> {
        chkerr!(
            props.ctxt(),
            dpiMsgProps_setPayloadObject(props.handle.raw, self.handle)
        );
        props.payload_type = Some(self.object_type().clone());
        Ok(())
    }
}

/// Advanced Queueing (AQ) queue which may be used to enqueue and dequeue messages
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
pub struct Queue<T>
where
    T: Payload + ?Sized,
{
    conn: Conn,
    handle: DpiQueue,
    payload_type: Option<ObjectType>,
    enq_options: Option<EnqOptions>,
    deq_options: Option<DeqOptions>,
    phantom: PhantomData<T>,
}

impl<'a, T: 'a> Queue<T>
where
    T: Payload + ?Sized,
{
    fn handle(&self) -> *mut dpiQueue {
        self.handle.raw
    }

    fn ctxt(&self) -> &Context {
        self.conn.ctxt()
    }

    /// Creates a new queue which may be used to enqueue and dequeue messages
    /// from Advanced Queuing (AQ) queues.
    pub fn new(
        conn: &Connection,
        queue_name: &str,
        payload_type: &T::TypeInfo,
    ) -> Result<Queue<T>> {
        let mut handle = ptr::null_mut();
        let name = to_odpi_str(queue_name);
        let payload_type = T::payload_type(payload_type)?;
        let objtype = payload_type
            .as_ref()
            .map(|t| t.handle().raw)
            .unwrap_or(ptr::null_mut());
        chkerr!(
            conn.ctxt(),
            dpiConn_newQueue(conn.handle(), name.ptr, name.len, objtype, &mut handle)
        );
        Ok(Queue {
            conn: conn.conn.clone(),
            handle: DpiQueue::new(handle),
            payload_type,
            enq_options: None,
            deq_options: None,
            phantom: PhantomData,
        })
    }

    /// Dequeues a single message from the queue.
    pub fn dequeue(&self) -> Result<MsgProps<T>> {
        let mut props = ptr::null_mut();
        chkerr!(self.ctxt(), dpiQueue_deqOne(self.handle(), &mut props));
        Ok(MsgProps::from_dpi_msg_props(
            self.conn.clone(),
            DpiMsgProps::new(props),
            self.payload_type.clone(),
        ))
    }

    /// Dequeues multiple messages from the queue.
    pub fn dequeue_many(&self, max_size: u32) -> Result<Vec<MsgProps<T>>> {
        let mut num_props = max_size;
        let mut handles = Vec::<DpiMsgProps>::with_capacity(max_size as usize);
        chkerr!(
            self.ctxt(),
            dpiQueue_deqMany(
                self.handle(),
                &mut num_props,
                // The following code works only when
                // the size of `*mut dpiMsgProps` equals to that of `DpiMsgProps`.
                handles.as_mut_ptr() as *mut *mut dpiMsgProps
            )
        );
        let num_props = num_props as usize;
        unsafe {
            handles.set_len(num_props);
        }
        let props: Vec<_> = handles
            .into_iter()
            .map(|handle| {
                MsgProps::from_dpi_msg_props(self.conn.clone(), handle, self.payload_type.clone())
            })
            .collect();
        Ok(props)
    }

    /// Enqueues a single mesasge into the queue.
    pub fn enqueue(&self, props: &MsgProps<T>) -> Result<()> {
        chkerr!(self.ctxt(), dpiQueue_enqOne(self.handle(), props.handle()));
        Ok(())
    }

    /// Enqueues multiple messages into the queue.
    ///
    /// **Warning:** calling this function in parallel on different connections
    /// acquired from the same pool may fail due to Oracle bug 29928074. Ensure
    /// that this function is not run in parallel, use standalone connections or
    /// connections from different pools, or make multiple calls to
    /// [`Queue.enqueue`] instead. The function [`Queue.dequeue_many`]
    /// call is not affected.
    ///
    /// [`Queue.enqueue`]: #method.enqueue
    /// [`Queue.dequeue_many`]: #method.dequeue_many
    pub fn enqueue_many<I>(&self, props: I) -> Result<()>
    where
        I: IntoIterator<Item = &'a MsgProps<T>>,
    {
        let iter = props.into_iter();
        let (lower, _) = iter.size_hint();
        let mut raw_props = Vec::with_capacity(lower);
        for msg in iter {
            let handle = msg.handle();
            raw_props.push(handle);
            unsafe {
                dpiMsgProps_addRef(handle);
            }
        }
        chkerr!(
            self.ctxt(),
            dpiQueue_enqMany(
                self.handle(),
                raw_props.len() as u32,
                raw_props.as_mut_ptr()
            ),
            for handle in raw_props {
                unsafe {
                    dpiMsgProps_release(handle);
                }
            }
        );
        for handle in raw_props {
            unsafe {
                dpiMsgProps_release(handle);
            }
        }
        Ok(())
    }

    /// Returns a reference to the dequeue options associated with the queue. These
    /// options affect how messages are dequeued.
    pub fn deq_options(&mut self) -> Result<&mut DeqOptions> {
        if self.deq_options.is_none() {
            let mut handle = ptr::null_mut();
            chkerr!(
                self.ctxt(),
                dpiQueue_getDeqOptions(self.handle(), &mut handle)
            );
            self.deq_options = Some(DeqOptions::new(self.ctxt().clone(), handle));
        }
        Ok(self.deq_options.as_mut().unwrap())
    }

    /// Returns a reference to the enqueue options associated with the queue. These
    /// options affect how messages are enqueued.
    pub fn enq_options(&mut self) -> Result<&mut EnqOptions> {
        if self.enq_options.is_none() {
            let mut handle = ptr::null_mut();
            chkerr!(
                self.ctxt(),
                dpiQueue_getEnqOptions(self.handle(), &mut handle)
            );
            self.enq_options = Some(EnqOptions::new(self.ctxt().clone(), handle));
        }
        Ok(self.enq_options.as_mut().unwrap())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Delivery mode used for filtering messages when dequeuing messages from a queue
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
pub enum MessageDeliveryMode {
    /// Dequeue only persistent messages from the queue. This is the default mode.
    Persistent,
    /// Dequeue only buffered messages from the queue.
    Buffered,
    /// Dequeue both persistent and buffered messages from the queue.
    PersistentOrBuffered,
}

impl MessageDeliveryMode {
    fn from_dpi_value(val: dpiMessageDeliveryMode) -> Result<MessageDeliveryMode> {
        match val as u32 {
            DPI_MODE_MSG_PERSISTENT => Ok(MessageDeliveryMode::Persistent),
            DPI_MODE_MSG_BUFFERED => Ok(MessageDeliveryMode::Buffered),
            DPI_MODE_MSG_PERSISTENT_OR_BUFFERED => Ok(MessageDeliveryMode::PersistentOrBuffered),
            _ => Err(Error::InternalError(format!(
                "unknown dpiMessageDeliveryMode {}",
                val
            ))),
        }
    }

    fn to_dpi_value(&self) -> dpiMessageDeliveryMode {
        match self {
            MessageDeliveryMode::Persistent => DPI_MODE_MSG_PERSISTENT as dpiMessageDeliveryMode,
            MessageDeliveryMode::Buffered => DPI_MODE_MSG_PERSISTENT as dpiMessageDeliveryMode,
            MessageDeliveryMode::PersistentOrBuffered => {
                DPI_MODE_MSG_PERSISTENT as dpiMessageDeliveryMode
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Possible states for messages in a queue
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
pub enum MessageState {
    /// The message is ready to be processed.
    Ready,
    /// The message is waiting for the delay time to expire.
    Waiting,
    /// The message has already been processed and is retained.
    Processed,
    /// The message has been moved to the exception queue.
    Expired,
}

impl MessageState {
    fn from_dpi_value(val: dpiMessageState) -> Result<MessageState> {
        match val {
            DPI_MSG_STATE_READY => Ok(MessageState::Ready),
            DPI_MSG_STATE_WAITING => Ok(MessageState::Waiting),
            DPI_MSG_STATE_PROCESSED => Ok(MessageState::Processed),
            DPI_MSG_STATE_EXPIRED => Ok(MessageState::Expired),
            _ => Err(Error::InternalError(format!(
                "unknown dpiMessageState {}",
                val
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Modes that are possible when dequeuing messages from a queue
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
pub enum DeqMode {
    /// Read the message without acquiring a lock on the
    ///  message(equivalent to a SELECT statement).
    Browse,
    /// Read the message and obtain a write lock on the
    /// message (equivalent to a SELECT FOR UPDATE
    /// statement).
    Locked,
    /// Read the message and update or delete it. This is
    /// the default mode. Note that the message may be
    /// retained in the queue table based on retention
    /// properties.
    Remove,
    /// Confirms receipt of the message but does not
    /// deliver the actual message content.
    RemoveNoData,
}

impl DeqMode {
    fn from_dpi_value(val: dpiDeqMode) -> Result<DeqMode> {
        match val {
            DPI_MODE_DEQ_BROWSE => Ok(DeqMode::Browse),
            DPI_MODE_DEQ_LOCKED => Ok(DeqMode::Locked),
            DPI_MODE_DEQ_REMOVE => Ok(DeqMode::Remove),
            DPI_MODE_DEQ_REMOVE_NO_DATA => Ok(DeqMode::RemoveNoData),
            _ => Err(Error::InternalError(format!("unknown dpiDeqMode {}", val))),
        }
    }

    fn to_dpi_value(&self) -> dpiDeqMode {
        match self {
            DeqMode::Browse => DPI_MODE_DEQ_BROWSE,
            DeqMode::Locked => DPI_MODE_DEQ_LOCKED,
            DeqMode::Remove => DPI_MODE_DEQ_REMOVE,
            DeqMode::RemoveNoData => DPI_MODE_DEQ_REMOVE_NO_DATA,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// method used for determining which message is to be dequeued from a queue
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
pub enum DeqNavigation {
    /// Retrieves the first available message that
    /// matches the search criteria. This resets the
    /// position to the beginning of the queue.
    FirstMessage,
    /// Skips the remainder of the current transaction
    /// group (if any) and retrieves the first message of
    /// the next transaction group. This option can only
    /// be used if message grouping is enabled for the
    /// queue.
    NextTransaction,
    /// Retrieves the next available message that matches
    /// the search criteria. This is the default method.
    NextMessage,
}

impl DeqNavigation {
    fn from_dpi_value(val: dpiDeqNavigation) -> Result<DeqNavigation> {
        match val {
            DPI_DEQ_NAV_FIRST_MSG => Ok(DeqNavigation::FirstMessage),
            DPI_DEQ_NAV_NEXT_TRANSACTION => Ok(DeqNavigation::NextTransaction),
            DPI_DEQ_NAV_NEXT_MSG => Ok(DeqNavigation::NextMessage),
            _ => Err(Error::InternalError(format!(
                "unknown dpiDeqNavigation {}",
                val
            ))),
        }
    }

    fn to_dpi_value(&self) -> dpiDeqNavigation {
        match self {
            DeqNavigation::FirstMessage => DPI_DEQ_NAV_FIRST_MSG,
            DeqNavigation::NextTransaction => DPI_DEQ_NAV_NEXT_TRANSACTION,
            DeqNavigation::NextMessage => DPI_DEQ_NAV_NEXT_MSG,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// visibility of messages in advanced queuing
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
pub enum Visibility {
    /// The message is not part of the current transaction
    /// but constitutes a transaction of its own.
    Immediate,
    /// The message is part of the current transaction.
    /// This is the default value.
    OnCommit,
}

impl Visibility {
    fn from_dpi_value(val: dpiVisibility) -> Result<Visibility> {
        match val {
            DPI_VISIBILITY_IMMEDIATE => Ok(Visibility::Immediate),
            DPI_VISIBILITY_ON_COMMIT => Ok(Visibility::OnCommit),
            _ => Err(Error::InternalError(format!(
                "unknown dpiVisibility {}",
                val
            ))),
        }
    }

    fn to_dpi_value(&self) -> dpiVisibility {
        match self {
            Visibility::Immediate => DPI_VISIBILITY_IMMEDIATE,
            Visibility::OnCommit => DPI_VISIBILITY_ON_COMMIT,
        }
    }
}

/// Options when dequeuing messages using advanced queueing
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
pub struct DeqOptions {
    ctxt: Context,
    handle: *mut dpiDeqOptions,
}

impl DeqOptions {
    fn new(ctxt: Context, handle: *mut dpiDeqOptions) -> DeqOptions {
        DeqOptions { ctxt, handle }
    }

    fn ctxt(&self) -> &Context {
        &self.ctxt
    }

    /// Returns the condition that must be satisfied in order for a message to be
    /// dequeued.
    ///
    /// See [`set_condition`](#method.set_condition) method for more information
    pub fn condition(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_getCondition(self.handle, &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Returns the name of the consumer that is dequeuing messages.
    ///
    /// see [`set_consumer_name`](#method.set_consumer_name) method for more information.
    pub fn consumer_name(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_getConsumerName(self.handle, &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    ///  Returns the correlation of the message to be dequeued.
    ///
    ///  See [`set_correlation`](#method.set_correlation) method for more information.
    pub fn correlation(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_getCorrelation(self.handle, &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Returns the mode that is to be used when dequeuing messages.
    pub fn mode(&self) -> Result<DeqMode> {
        let mut val = 0;
        chkerr!(self.ctxt(), dpiDeqOptions_getMode(self.handle, &mut val));
        DeqMode::from_dpi_value(val)
    }

    /// Returns the identifier of the specific message that is to be dequeued.
    pub fn message_id(&self) -> Result<Vec<u8>> {
        let mut msg = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_getMsgId(self.handle, &mut msg.ptr, &mut msg.len)
        );
        Ok(msg.to_vec())
    }

    /// Returns the position of the message that is to be dequeued.
    pub fn navigation(&self) -> Result<DeqNavigation> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_getNavigation(self.handle, &mut val)
        );
        DeqNavigation::from_dpi_value(val)
    }

    /// Returns the transformation of the message to be dequeued.
    ///
    /// See [`set_transformation`](#method.set_transformation) method for more information.
    pub fn transformation(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_getTransformation(self.handle, &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Returns whether the message being dequeued is part of the current
    /// transaction or constitutes a transaction on its own.
    pub fn visibility(&self) -> Result<Visibility> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_getVisibility(self.handle, &mut val)
        );
        Visibility::from_dpi_value(val)
    }

    /// Returns the time to wait for a message matching the search
    /// criteria.
    pub fn wait(&self) -> Result<Duration> {
        let mut val = 0;
        chkerr!(self.ctxt(), dpiDeqOptions_getWait(self.handle, &mut val));
        Ok(Duration::from_secs(val as u64))
    }

    /// Sets the condition which must be true for messages to be dequeued.
    ///
    /// The condition must be a valid boolean expression similar to the where clause
    /// of a SQL query. The expression can include conditions on message
    /// properties, user data properties and PL/SQL or SQL functions. User data
    /// properties must be prefixed with tab.user_data as a qualifier to indicate
    /// the specific column of the queue table that stores the message payload.
    pub fn set_condition(&mut self, val: &str) -> Result<()> {
        let val = to_odpi_str(val);
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_setCondition(self.handle, val.ptr, val.len)
        );
        Ok(())
    }

    /// Sets the name of the consumer which will be dequeuing messages. This value
    /// should only be set if the queue is set up for multiple consumers.
    pub fn set_consumer_name(&mut self, val: &str) -> Result<()> {
        let val = to_odpi_str(val);
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_setConsumerName(self.handle, val.ptr, val.len)
        );
        Ok(())
    }

    /// Sets the correlation of the message to be dequeued.
    ///
    /// Special pattern  matching characters such as the percent
    /// sign (`%`) and the underscore (`_`)
    /// can be used. If multiple messages satisfy the pattern, the order of
    /// dequeuing is undetermined.
    pub fn set_correlation(&mut self, val: &str) -> Result<()> {
        let val = to_odpi_str(val);
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_setCorrelation(self.handle, val.ptr, val.len)
        );
        Ok(())
    }

    /// Sets the message delivery mode that is to be used when dequeuing messages.
    pub fn set_delivery_mode(&mut self, val: &MessageDeliveryMode) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_setDeliveryMode(self.handle, val.to_dpi_value())
        );
        Ok(())
    }

    /// Sets the mode that is to be used when dequeuing messages.
    pub fn set_mode(&mut self, val: &DeqMode) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_setMode(self.handle, val.to_dpi_value())
        );
        Ok(())
    }

    /// Sets the identifier of the specific message to be dequeued.
    pub fn set_message_id(&mut self, val: &[u8]) -> Result<()> {
        let ptr = if val.is_empty() {
            ptr::null()
        } else {
            val.as_ptr() as *const c_char
        };
        let len = val.len() as u32;
        chkerr!(self.ctxt(), dpiDeqOptions_setMsgId(self.handle, ptr, len));
        Ok(())
    }

    /// Sets the position in the queue of the message that is to be dequeued.
    pub fn set_navigation(&mut self, val: &DeqNavigation) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_setNavigation(self.handle, val.to_dpi_value())
        );
        Ok(())
    }

    /// Sets the transformation of the message to be dequeued.
    ///
    /// The transformation
    /// is applied after the message is dequeued but before it is returned to the
    /// application. It must be created using DBMS_TRANSFORM.
    pub fn set_transformation(&mut self, val: &str) -> Result<()> {
        let val = to_odpi_str(val);
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_setTransformation(self.handle, val.ptr, val.len)
        );
        Ok(())
    }

    /// Sets whether the message being dequeued is part of the current transaction
    /// or constitutes a transaction on its own.
    pub fn set_visibility(&mut self, val: &Visibility) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiDeqOptions_setVisibility(self.handle, val.to_dpi_value())
        );
        Ok(())
    }

    /// Set the time to wait for a message matching the search
    /// criteria.
    pub fn set_wait(&mut self, val: &Duration) -> Result<()> {
        let secs = val.as_secs();
        let secs = if secs > u32::max_value().into() {
            u32::max_value()
        } else {
            secs as u32
        };
        chkerr!(self.ctxt(), dpiDeqOptions_setWait(self.handle, secs));
        Ok(())
    }
}

impl fmt::Debug for DeqOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DeqOptions {{ handle: {:?} }}", self.handle)
    }
}

/// Options when enqueuing messages using advanced queueing
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
pub struct EnqOptions {
    ctxt: Context,
    handle: *mut dpiEnqOptions,
}

impl EnqOptions {
    fn new(ctxt: Context, handle: *mut dpiEnqOptions) -> EnqOptions {
        EnqOptions { ctxt, handle }
    }

    fn ctxt(&self) -> &Context {
        &self.ctxt
    }

    /// Returns the transformation of the message to be enqueued.
    ///
    /// See [`set_transformation`](#method.set_transformation) method for more information.
    pub fn transformation(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiEnqOptions_getTransformation(self.handle, &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Returns whether the message being enqueued is part of the current
    /// transaction or constitutes a transaction on its own.
    pub fn visibility(&self) -> Result<Visibility> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiEnqOptions_getVisibility(self.handle, &mut val)
        );
        Visibility::from_dpi_value(val)
    }

    /// Sets the message delivery mode that is to be used when enqueuing messages.
    pub fn set_delivery_mode(&mut self, val: &MessageDeliveryMode) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiEnqOptions_setDeliveryMode(self.handle, val.to_dpi_value())
        );
        Ok(())
    }

    /// Sets the transformation of the message to be enqueued.
    ///
    /// The transformation
    /// is applied after the message is enqueued but before it is returned to the
    /// application. It must be created using DBMS_TRANSFORM.
    pub fn set_transformation(&mut self, val: &str) -> Result<()> {
        let val = to_odpi_str(val);
        chkerr!(
            self.ctxt(),
            dpiEnqOptions_setTransformation(self.handle, val.ptr, val.len)
        );
        Ok(())
    }

    /// Sets whether the message being enqueued is part of the current transaction
    /// or constitutes a transaction on its own.
    pub fn set_visibility(&mut self, val: &Visibility) -> Result<()> {
        chkerr!(
            self.ctxt(),
            dpiEnqOptions_setVisibility(self.handle, val.to_dpi_value())
        );
        Ok(())
    }
}

impl fmt::Debug for EnqOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EnqOptions {{ handle: {:?} }}", self.handle)
    }
}

/// Properties of messages that are enqueued and dequeued using advanced queuing
///
/// **Warning:** The type is unstable. It may be changed incompatibly by minor version upgrades.
#[derive(Clone)]
pub struct MsgProps<T>
where
    T: Payload + ?Sized,
{
    conn: Conn,
    handle: DpiMsgProps,
    payload_type: Option<ObjectType>,
    phantom: PhantomData<T>,
}

impl<T> MsgProps<T>
where
    T: Payload + ?Sized,
{
    fn handle(&self) -> *mut dpiMsgProps {
        self.handle.raw()
    }

    fn ctxt(&self) -> &Context {
        self.conn.ctxt()
    }

    /// Creates a new message properties
    pub fn new(conn: &Connection) -> Result<MsgProps<T>> {
        let mut handle = ptr::null_mut();
        chkerr!(conn.ctxt(), dpiConn_newMsgProps(conn.handle(), &mut handle));
        Ok(MsgProps {
            conn: conn.conn.clone(),
            handle: DpiMsgProps::new(handle),
            payload_type: None,
            phantom: PhantomData,
        })
    }

    fn from_dpi_msg_props(
        conn: Conn,
        handle: DpiMsgProps,
        payload_type: Option<ObjectType>,
    ) -> MsgProps<T> {
        MsgProps {
            conn,
            handle,
            payload_type,
            phantom: PhantomData,
        }
    }

    /// Returns the number of attempts that have been made to dequeue a message.
    pub fn num_attempts(&self) -> Result<i32> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiMsgProps_getNumAttempts(self.handle(), &mut val)
        );
        Ok(val)
    }

    /// Returns the correlation supplied by the producer when the message was
    /// enqueued.
    pub fn correlation(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiMsgProps_getCorrelation(self.handle(), &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Returns the duration the enqueued message will be delayed.
    pub fn delay(&self) -> Result<Duration> {
        let mut secs = 0;
        chkerr!(self.ctxt(), dpiMsgProps_getDelay(self.handle(), &mut secs));
        Ok(Duration::from_secs(secs as u64))
    }

    /// Returns the mode that was used to deliver the message.
    pub fn delivery_mode(&self) -> Result<MessageDeliveryMode> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiMsgProps_getDeliveryMode(self.handle(), &mut val)
        );
        MessageDeliveryMode::from_dpi_value(val)
    }

    /// Returns the time that the message was enqueued.
    pub fn enq_time(&self) -> Result<Timestamp> {
        let mut val = Default::default();
        chkerr!(self.ctxt(), dpiMsgProps_getEnqTime(self.handle(), &mut val));
        Ok(Timestamp::from_dpi_timestamp(&val, &OracleType::Date))
    }

    /// Returns the name of the queue to which the message is moved if it cannot be
    /// processed successfully.
    ///
    /// See [`set_exception_queue`](#method.set_exception_queue) method for more information.
    pub fn exception_queue(&self) -> Result<String> {
        let mut s = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiMsgProps_getExceptionQ(self.handle(), &mut s.ptr, &mut s.len)
        );
        Ok(s.to_string())
    }

    /// Returns the duration the message is available to be dequeued.
    ///
    /// See [`set_expiration`](#method.set_expiration) method for more information.
    pub fn expiration(&self) -> Result<Duration> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiMsgProps_getExpiration(self.handle(), &mut val)
        );
        Ok(Duration::from_secs(val as u64))
    }

    /// Returns the id of the message in the queue that generated this message. No
    /// value is available until the message has been enqueued or dequeued.
    pub fn message_id(&self) -> Result<Vec<u8>> {
        let mut msg = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiMsgProps_getMsgId(self.handle(), &mut msg.ptr, &mut msg.len)
        );
        Ok(msg.to_vec())
    }

    /// Returns the id of the message in the last queue that generated this
    /// message.
    ///
    /// See [`set_original_message_id`](#method.set_original_message_id) for more information.
    pub fn original_message_id(&self) -> Result<Vec<u8>> {
        let mut msg = new_odpi_str();
        chkerr!(
            self.ctxt(),
            dpiMsgProps_getOriginalMsgId(self.handle(), &mut msg.ptr, &mut msg.len)
        );
        Ok(msg.to_vec())
    }

    /// Returns the payload associated with the message properties.
    ///
    /// The payload is available after the a call to [`Queue.dequeue`] or
    /// [`Queue.dequeue_many`]
    ///
    /// [`Queue.dequeue`]: Queue#method.dequeue
    /// [`Queue.dequeue_many`]: Queue#method.dequeue_many
    pub fn payload(&self) -> Result<T::Owned> {
        T::get(self)
    }

    /// Returns the priority assigned to the message.
    ///
    /// See [`set_priority`](#method.set_priority) method for more information.
    pub fn priority(&self) -> Result<i32> {
        let mut val = 0;
        chkerr!(
            self.ctxt(),
            dpiMsgProps_getPriority(self.handle(), &mut val)
        );
        Ok(val)
    }

    /// Returns the state of the message at the time of dequeue.
    pub fn state(&self) -> Result<MessageState> {
        let mut val = 0;
        chkerr!(self.ctxt(), dpiMsgProps_getState(self.handle(), &mut val));
        MessageState::from_dpi_value(val)
    }

    /// Sets the correlation of the message to be dequeued.
    ///
    /// Special pattern matching characters such as the percent
    /// sign (`%`) and the underscore (`_`) can be used. If multiple
    /// messages satisfy the pattern, the order of dequeuing is
    /// undetermined.
    pub fn set_correlation(&mut self, val: &str) -> Result<()> {
        let val = to_odpi_str(val);
        chkerr!(
            self.ctxt(),
            dpiMsgProps_setCorrelation(self.handle(), val.ptr, val.len)
        );
        Ok(())
    }

    /// Sets the number of seconds to delay the message before it can be dequeued.
    ///
    /// Messages enqueued with a delay are put into the [`MessageState::Waiting`]
    /// state. When the delay expires the message is put into the
    /// [`MessageState::Ready`] state. Dequeuing directly by message id overrides this
    /// delay specification. Note that delay processing requires the queue monitor
    /// to be started.
    ///
    /// [`MessageState::Waiting`]: MessageState#variant.Waiting
    /// [`MessageState::Ready`]: MessageState#variant.Ready
    pub fn set_delay(&mut self, val: &Duration) -> Result<()> {
        let secs = val.as_secs();
        if secs > i32::max_value() as u64 {
            Err(Error::OutOfRange(format!("too long duration {:?}", val)))
        } else {
            chkerr!(
                self.ctxt(),
                dpiMsgProps_setDelay(self.handle(), secs as i32)
            );
            Ok(())
        }
    }

    /// Sets the name of the queue to which the message is moved if it cannot be
    /// processed successfully.
    ///
    /// Messages are moved if the number of unsuccessful
    /// dequeue attempts has reached the maximum allowed number or if the message
    /// has expired. All messages in the exception queue are in the
    /// [`MessageState::Expired`] state.
    ///
    /// [`MessageState::Expired`]: MessageState#variant.Expired
    pub fn set_exception_queue(&mut self, val: &str) -> Result<()> {
        let val = to_odpi_str(val);
        chkerr!(
            self.ctxt(),
            dpiMsgProps_setExceptionQ(self.handle(), val.ptr, val.len)
        );
        Ok(())
    }

    /// Sets the number of seconds the message is available to be dequeued.
    ///
    /// This value is an offset from the delay. Expiration processing requires the queue
    /// monitor to be running. Until this time elapses, the messages are in the
    /// queue in the state [`MessageState::Ready`]. After this time elapses messages
    /// are moved to the exception queue in the [`MessageState::Expired`] state.
    ///
    /// [`MessageState::Ready`]: MessageState#variant.Ready
    /// [`MessageState::Expired`]: MessageState#variant.Expired
    pub fn set_expiration(&mut self, val: &Duration) -> Result<()> {
        let secs = val.as_secs();
        if secs > i32::max_value() as u64 {
            Err(Error::OutOfRange(format!("too long duration {:?}", val)))
        } else {
            chkerr!(
                self.ctxt(),
                dpiMsgProps_setExpiration(self.handle(), secs as i32)
            );
            Ok(())
        }
    }

    /// Sets the id of the message in the last queue that generated this
    /// message.
    pub fn set_original_message_id(&mut self, val: &[u8]) -> Result<()> {
        let ptr = if val.is_empty() {
            ptr::null()
        } else {
            val.as_ptr() as *const c_char
        };
        let len = val.len() as u32;
        chkerr!(
            self.ctxt(),
            dpiMsgProps_setOriginalMsgId(self.handle(), ptr, len)
        );
        Ok(())
    }

    /// Sets the payload for the message.
    ///
    /// This value will be used when the message is enqueued using
    /// [`Queue.enqueue`] or [`Queue.enqueue_many`].
    ///
    /// [`Queue.enqueue`]: Queue#method.enqueue
    /// [`Queue.enqueue_many`]: Queue#method.enqueue_many
    pub fn set_payload(&mut self, val: &T) -> Result<()> {
        val.set(self)
    }

    /// Sets the priority assigned to the message.
    ///
    /// A smaller number indicates a higher priority. The priority can
    /// be any number, including negative numbers.
    pub fn set_priority(&mut self, val: i32) -> Result<()> {
        chkerr!(self.ctxt(), dpiMsgProps_setPriority(self.handle(), val));
        Ok(())
    }
}

impl<T> fmt::Debug for MsgProps<T>
where
    T: Payload,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MsgProps {{ handle: {:?} }}", self.handle())
    }
}
