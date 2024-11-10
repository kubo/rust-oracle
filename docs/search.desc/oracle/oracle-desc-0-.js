searchState.loadedDescShard("oracle", 0, "Rust-oracle\nDoes not wait for current calls to complete or users to …\nALTER statement\nStatement batch, which inserts, updates or deletes more …\nA trait implemented by types that can index into bind …\nA builder to create a <code>Batch</code> with various configuration\nError when <code>BatchBuilder::with_batch_errors</code> is set and …\nPL/SQL statement without declare clause\nA trait implemented by types that can index into bind …\nCALL statement\nThe connection has been closed by <code>Connection::close</code>.\nA trait implemented by types that can index into columns …\nColumn information in a select statement\nCOMMIT statement\nConnection status\nConnection to an Oracle database\nBuilder data type to create Connection.\nCREATE statement\nOracle database error or ODPI-C error\nPL/SQL statement with declare clause\nFurther connects are prohibited. Waits for users to …\nDELETE statement\nError from an underlying ODPI-C layer.\nError from an underlying ODPI-C layer.\nDROP statement\nContains the error value\nThe error type for oracle\nA list of error categories.\nEXPLAIN PLAN statement\nShuts down the database. Should be used only in the second …\nShuts down a running instance (if there is any) using …\nDoes not wait for current calls to complete or users to …\nParameters for explicit Oracle client library …\nINSERT statement\nInternal error. When you get this error, please report it …\nInternal error. When you get this error, please report it …\nError when an unacceptable argument is passed\nError when an unacceptable argument is passed\nError when the specified attribute name is not found.\nError when the specified attribute name is not found.\nError when the bind parameter index is out of range. (one …\nError when the bind parameter index is out of range. (one …\nError when the bind parameter name is not in the SQL.\nError when the bind parameter name is not in the SQL.\nError when the column index is out of range. (zero based)\nError when the column index is out of range. (zero based)\nError when the column name is not in the SQL.\nError when the column name is not in the SQL.\nError when invalid method is called such as calling …\nError when invalid method is called such as calling …\nError when conversion from a type to another is not …\nError when conversion from a type to another is not …\nMERGE statement\nError when no more rows exist in the SQL.\nError when no more rows exist in the SQL.\nThe connection is alive. See <code>Connection::status</code> for …\nThe connection has been terminated. See <code>Connection::status</code> …\nError when NULL value is got but the target rust type …\nError when NULL value is got but the target rust type …\nError from an underlying Oracle client library.\nError from an underlying Oracle client library.\nContains the success value\nError when conversion from a type to another fails due to …\nError when conversion from a type to another fails due to …\nError when conversion from a string to an Oracle value …\nError when conversion from a string to an Oracle value …\nAn error when parsing a string into an Oracle type fails. …\nAdministrative privilege\nAllows database access only to users with both the CREATE …\nResult set\nROLLBACK statement\nRow in a result set of a select statement\nA trait to get a row as specified type\nA derive macro to implement the <code>RowValue</code> trait\nSELECT statement\nDatabase shutdown mode\nA type containing an Oracle value\nDatabase startup mode\nStatement\nA builder to create a <code>Statement</code> with various configuration\nStatement type returned by <code>Statement::statement_type</code>.\nConnects as SYSASM (Oracle 12c or later)\nConnects as SYSBACKUP\nConnects as SYSDBA.\nConnects as SYSDG (Oracle 12c or later)\nConnects as SYSKM (Oracle 12c or later)\nConnects as SYSOPER.\nConnects as SYSRAC (Oracle 12c R2 or later)\nFurther connects are prohibited and no new transactions …\nFurther connects are prohibited and no new transactions …\nError when an uninitialized bind value is accessed. Bind …\nError when an uninitialized bind value is accessed. Bind …\nUnknown statement\nUPDATE statement\nOracle version information\nThe internal action that was being performed when the …\nAppends an application context.\nOracle Advanced Queuing (available when <code>aq_unstable</code> …\nGets autocommit mode. It is false by default.\nCreates BatchBuilder\nReturns batch errors. See “Error Handling with batch …\nSet a bind value in the statement.\nReturns the number of bind parameters\nReturns the number of bind variables in the statement.\nReturns an array of bind parameter names\nReturns the names of the unique bind variables in the …\nGets a bind value in the statement.\nCancels execution of running statements in the connection\nGets the current call timeout used for round-trips to the …\nChanges the password for the specified user\nClear the object type cache in the connection.\nReturns the version of Oracle client in use.\nCloses the batch before the end of its lifetime.\nCloses the connection before the end of lifetime.\nCloses the statement before the end of lifetime.\nThe OCI error code if an OCI error has taken place. If no …\nCommits the current active transaction\nType definitions for connection\nConnects to an Oracle server using username, password and …\nConnect an Oracle server using specified parameters\nSets a connection class to restrict sharing DRCP pooled …\nGets current schema associated with the connection\nReturns <code>DbError</code>.\nSets the default driver name to use when creating pools or …\nReturns ODPI-C error code.\nSets the driver name displayed in …\nGets edition associated with the connection\nSpecifies edition of Edition-Based Redefinition.\nReserved for when advanced queuing (AQ) or continuous query\nExcludes the statement from the cache even when …\nCreates a statement, binds values by position and executes …\nBinds values by position and executes the statement. It …\nCreates a statement, binds values by name and executes it …\nBinds values by name and executes the statement. It will …\nUses external authentication such as OS authentication.\nGets external name associated with the connection\nChanges the array size used for performing fetches.\nFormats any SQL value to string using the given formatter. …\nThe public ODPI-C, used by rust-oracle, function name …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGets the column value at the specified index.\nGets the Oracle value. It internally does the followings:\nGets column values as specified type.\nReturns the next implicit result returned by …\nReturns information about the connection\nInitializes Oracle client library.\nGets internal name associated with the connection\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nExecutes the prepared statement and returns a result set …\nExecutes the prepared statement using named parameters and …\nType definitions for I/O in characters\nReturns true when the SQL statement is DDL (data …\nReturns true when the SQL statement is DML (data …\nReturns true when the SQL statement is DML (data …\nReturns <code>true</code> if Oracle client library has initialized …\nReturns <code>true</code> when the connection is a standalone one or a …\nReturns <code>Ok(true)</code> when the SQL value is null. <code>Ok(false)</code> …\nReturns true when the SQL statement is a PL/SQL block.\nReturns true when the SQL statement is a PL/SQL block.\nReturns true when the SQL statement is a query.\nA boolean value indicating if the error is recoverable. …\nReturns true when the SQL statement has a <code>RETURNING INTO</code> …\nA boolean value indicating if the error information is for …\nReturns the corresponding <code>ErrorKind</code> for this error.\nReturns the rowid of the last row that was affected by the …\nGet the warning when connecting to the database or …\nSets the URL that should be provided in the error message …\nEnables lob data types to be fetched or bound as <code>Clob</code>, …\nGets 1st part of Oracle version number\nThe error message\nGets 2nd part of Oracle version number\nGets column name\nCreate a connector\nCreates a new initialization parameter\nCreates a new DbError. Note that its <code>is_recoverable</code> and …\nCreates a new version information\nSets new password during establishing a connection.\nGets whether the column may be NULL. False when the column …\nGets an object type information from name\nRust-oracle is based on ODPI-C using Oracle Call Interface …\nGets an OCI handle attribute corresponding to the …\nGets an OCI handle attribute corresponding to the …\nReturns Oracle error code. For example 1 for “ORA-0001: …\nThe parse error offset (in bytes) when executing a …\nSets the location the Oracle client library will search for\nSets the location from which to load the Oracle Client …\nGets the Oracle type of the SQL value.\nGets Oracle type\nGets 4th part of Oracle version number\nPings the connection to see if it is still alive.\nType definitions for connection pooling\nGets 5th part of Oracle version number\nThe number of rows that will be prefetched by the Oracle …\nSets prelim auth mode to connect to an idle instance.\nSet administrative privilege.\nSets session purity specifying whether an application can …\nExecutes a select statement and returns a result set …\nExecutes the prepared statement and returns a result set …\nExecutes a select statement and returns a result set …\nExecutes the prepared statement and returns a result set …\nExecutes a select statement using named parameters and …\nExecutes the prepared statement using named parameters and …\nExecutes a select statement using named parameters and …\nExecutes the prepared statement using named parameters and …\nGets one row from a query using positoinal bind parameters.\nGets one row from the prepared statement using positoinal …\nGets one row from a query as specified type.\nGets one row from the prepared statement as specified type …\nGets one row from a query with named bind parameters as …\nGets one row from the prepared statement as specified type …\nGets one row from a query using named bind parameters.\nGets one row from the prepared statement using named bind …\nGets values returned by RETURNING INTO clause.\nRolls back the current active transaction\nReturns the number of rows fetched when the SQL statement …\nReturns the number of affected rows\nGets information about the server version\nSet a parameter value\nSets a rust value to the Oracle value. It internally does …\nSets action associated with the connection\nEnables or disables autocommit mode. It is disabled by …\nSets the call timeout to be used for round-trips to the …\nSets client identifier associated with the connection\nSets client info associated with the connection\nSets current schema associated with the connection\nSets name of the database operation to be monitored in the …\nSets external name associated with the connection\nSets internal name associated with the connection\nSets module associated with the connection\nSets null to the SQL value.\nSets an OCI handle attribute corresponding to the …\nSets an OCI handle attribute corresponding to the …\nSets the statement cache size\nSet the data type of a bind parameter\nShuts down a database\nSQL data types\nReturns column values as a vector of SqlValue\nStarts up a database\nCreates <code>StatementBuilder</code> to create a <code>Statement</code>\nReturns statement type\nReturns statement type\nGets the status of the connection.\nGets the statement cache size\nSpecifies the number of statements to retain in the …\nGets the tag of the connection that was acquired from a …\nSpecifies the key to be used for searching for the …\nGets <code>true</code> when the connection is a standalone one or it is …\nGets 3rd part of Oracle version number\nSee “Error Handling”\nSee “Affected Rows”\nRead the message without acquiring a lock on the …\nDequeue only buffered messages from the queue.\nModes that are possible when dequeuing messages from a …\nmethod used for determining which message is to be …\nOptions when dequeuing messages using advanced queueing\nOptions when enqueuing messages using advanced queueing\nThe message has been moved to the exception queue.\nRetrieves the first available message that matches the …\nThe message is not part of the current transaction but …\nRead the message and obtain a write lock on the message …\nDelivery mode used for filtering messages when dequeuing …\nPossible states for messages in a queue\nProperties of messages that are enqueued and dequeued …\nRetrieves the next available message that matches the …\nSkips the remainder of the current transaction group (if …\nThe message is part of the current transaction. This is …\nA trait for payload type\nDequeue only persistent messages from the queue. This is …\nDequeue both persistent and buffered messages from the …\nThe message has already been processed and is retained.\nAdvanced Queueing (AQ) queue which may be used to enqueue …\nThe message is ready to be processed.\nRead the message and update or delete it. This is the …\nConfirms receipt of the message but does not deliver the …\nvisibility of messages in advanced queuing\nThe message is waiting for the delay time to expire.\nReturns the condition that must be satisfied in order for …\nReturns the name of the consumer that is dequeuing …\nReturns the correlation of the message to be dequeued.\nReturns the correlation supplied by the producer when the …\nReturns the duration the enqueued message will be delayed.\nReturns the mode that was used to deliver the message.\nReturns a reference to the dequeue options associated with …\nDequeues a single message from the queue.\nDequeues multiple messages from the queue.\nReturns a reference to the enqueue options associated with …\nReturns the time that the message was enqueued.\nEnqueues a single mesasge into the queue.\nEnqueues multiple messages into the queue.\nReturns the name of the queue to which the message is …\nReturns the duration the message is available to be …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the identifier of the specific message that is to …\nReturns the id of the message in the queue that generated …\nReturns the mode that is to be used when dequeuing …\nReturns the position of the message that is to be dequeued.\nCreates a new queue which may be used to enqueue and …\nCreates a new message properties\nReturns the number of attempts that have been made to …\nReturns the id of the message in the last queue that …\nReturns the payload associated with the message properties.\nReturns the priority assigned to the message.\nSets the condition which must be true for messages to be …\nSets the name of the consumer which will be dequeuing …\nSets the correlation of the message to be dequeued.\nSets the correlation of the message to be dequeued.\nSets the number of seconds to delay the message before it …\nSets the message delivery mode that is to be used when …\nSets the message delivery mode that is to be used when …\nSets the name of the queue to which the message is moved …\nSets the number of seconds the message is available to be …\nSets the identifier of the specific message to be dequeued.\nSets the mode that is to be used when dequeuing messages.\nSets the position in the queue of the message that is to …\nSets the id of the message in the last queue that …\nSets the payload for the message.\nSets the priority assigned to the message.\nSets the transformation of the message to be dequeued.\nSets the transformation of the message to be enqueued.\nSets whether the message being dequeued is part of the …\nSets whether the message being enqueued is part of the …\nSet the time to wait for a message matching the search …\nReturns the state of the message at the time of dequeue.\nReturns the transformation of the message to be dequeued.\nReturns the transformation of the message to be enqueued.\nReturns whether the message being dequeued is part of the …\nReturns whether the message being enqueued is part of the …\nReturns the time to wait for a message matching the search …\nThe mode to use when closing connections to the database\nA dedicated server process is being used with the …\nThe connection is returned to the connection pool for …\nCauses the connection to be dropped from the connection …\nInformation about a connection\nMust use a new session\nA pooled server process (DRCP) is being used with the …\nSession Purity\nCauses the connection to be tagged with the tag …\nReuse a pooled session\nThe type of server process associated with a connection\nA shared server process is being used with the connection.\nThe type of server process is unknown.\nThe name of the Oracle Database Domain name associated …\nThe Oracle Database name associated with the connection\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nThe Oracle Database instance name associated with the …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe maximum length of identifiers (in bytes) supported by …\nThe maximum number of cursors that can be opened\nThe type of server process used by the connection\nThe Oracle Database service name associated with the …\nA cursor which can be moved within a stream of characters.\nSeek to an offset, in characters, in a stream.\nReturns the current seek position from the start of the …\nAttribute number defined in <code>oci.h</code> included in Oracle …\nA type parameter for <code>Connection::oci_attr</code> to get …\nA type parameter for <code>Connection::oci_attr</code> and …\nAttribute data type\nA type parameter for <code>Connection::oci_attr</code> and …\n<code>SvcCtx</code>, <code>Session</code>, <code>Server</code> or <code>Stmt</code>. Other handle and …\nA type parameter for <code>Connection::oci_attr</code> to get …\n<code>Read</code>, <code>Write</code> or <code>ReadWrite</code>\nA type parameter for <code>Statement::oci_attr</code> to get …\nA type parameter for <code>Statement::oci_attr</code> to get …\nA type parameter for <code>Connection::oci_attr</code> to get …\nA type parameter for <code>Connection::oci_attr</code> to get …\nThe module defines types related to the associate type …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nThe module defines types to be set to the associate type …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe module defines types to be set to the associate type …\nAttribute value used in <code>DataType</code>. You have no need to use …\nA trait to get and set OCI attributes as rust types. You …\nA type to get and set u64 microsecond attribute values as …\nThe maximum size is 32767 bytes for <code>VARCHAR2</code>, <code>NVARCHAR</code> and …\nA type corresponding to the <code>init.ora</code> parameter …\nThe maximum size is 4000 bytes for <code>VARCHAR2</code> and <code>NVARCHAR</code>, …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGets a value from the attribute.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nSets a value to the attribute.\nOCI handle type related to <code>Connection</code> to restrict the type …\nOCI handle type to restrict the associate type …\n<code>HandleType</code> for Server Handle Attributes\n<code>HandleType</code> for Authentication Information Handle Attributes…\n<code>HandleType</code> for Statement Handle Attributes\n<code>HandleType</code> for Service Context Handle Attributes\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nAccess mode to restrict the associate type <code>OciAttr::Mode</code>\nRead only mode, which implements <code>ReadMode</code>\nAccess mode to restrict the type parameters of …\nRead write mode, which implements both <code>ReadMode</code> and …\nWrite only mode, which implements <code>WriteMode</code>\nAccess mode to restrict the type parameters of …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe mode to use when closing pools\nIf there are any active connections in the pool an error …\nCauses all of the active connections in the pool to be …\nA new connection should be created if all of the …\nThe mode to use when getting connections from a connection …\nConnections with different authentication contexts can be …\nThe default pool type. All connections in the pool are …\nThe caller should return immediately, regardless of …\nConnection pool\nA bulider to make a connection pool\nAdditional options to get a connection from a pool\nWhether a connection pool is homogeneous or heterogeneous.\nThe caller should block until a connection is available …\nThe caller should block until a connection is available …\nMake a connection pool\nReturns the number of connections in the pool that are …\nCloses the pool and makes it unusable for further activity.\nSpecifies the number of connections that will be created …\nSets the driver name displayed in …\nSpecifies edition of Edition-Based Redefinition.\nReserved for when advanced queuing (AQ) or continuous query\nSpecifies whether external authentication should be used …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGets a connection from the pool with default parameters.\nSpecifies the mode to use when connections are acquired …\nReturns the mode used for acquiring or getting connections …\nAcquires a connection from the specified connection pool.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nSpecifies the maximum number of connections that can be …\nSpecifies the maximum number of connections that can be …\nReturns the maximum connections per shard. This parameter …\nSpecifies the maximum length of time a pooled connection …\nReturns the maximum lifetime a pooled connection may exist.\nSpecifies the minimum number of connections to be created …\nCreates a builder to make a connection pool.\nReturns the number of connections in the pool that are …\nSpecifies the length of time since a connection has last …\nReturns the ping interval duration, which is used to check …\nSpecifies the length of time to wait when performing a …\nSpecifies the name of a PL/SQL procedure in the format …\nSpecifies whether the pool is homogeneous or …\nChanges pool configuration corresponding to …\nSets the mode used for acquiring or getting connections …\nSets the maximum number of connections per shard.\nSets the maximum lifetime a pooled connection may exist.\nSets the ping interval duration which is used to to check …\nSets the default size of the statement cache for …\nSets the amount of time after which idle connections in the\nSpecifies the number of statements to retain in the …\nReturns the default size of the statement cache for …\nSpecifies the length of time after which idle connections …\nReturns the length of time after which idle connections in …\nBFILE\nBLOB\nBINARY_DOUBLE\nBINARY_FLOAT\nA reference to Oracle data type <code>BLOB</code>\nBOOLEAN (not supported)\nCLOB\nCHAR(size)\nA reference to Oracle data type <code>CLOB</code>\nOracle-specific collection data type\nDATE data type\nFLOAT(precision)\nConversion from Oracle values to rust values.\nInteger type in Oracle object type attributes. This will …\nOracle-specific Interval Day to Second data type.\nINTERVAL DAY(lfprec) TO SECOND(fsprec)\nOracle-specific Interval Year to Month data type.\nINTERVAL YEAR(lfprec) TO MONTH\nJSON data type introduced in Oracle 21c\nA trait for LOB types\nLONG\nLONG RAW\nNCLOB\nNCHAR(size)\nNVARCHAR2(size)\nA reference to Oracle data type <code>NCLOB</code>\nNUMBER(precision, scale)\nOracle-specific object data type\nObject\nType information about Object or Collection data type\nObject type attribute information\nOracle data type\nRAW(size)\nResult set output by or returned by a PL/SQL block or a …\nREF CURSOR (not supported)\nROWID\nOracle-specific Datetime data type\nTIMESTAMP(fsprec)\nTIMESTAMP(fsprec) WITH LOCAL TIME ZONE\nTIMESTAMP(fsprec) WITH TIME ZONE\nConversion from rust values to Oracle values.\nA trait specifying Oracle type to bind a null value.\nNot an Oracle type, used only internally to bind/define …\nVARCHAR2(size)\nXML\nCreates a new IntervalDS with precisions.\nCreates a new IntervalYM with precision.\nCreates a timestamp with precision.\nCreates a timestamp with time zone.\nCreates a timestamp with time zone.\nGets a vector of attribute information if it isn’t a …\nReturns the chunk size, in bytes, of the internal LOB. …\nCloses the LOB.\nCloses the LOB.\nCloses the LOB.\nCloses the LOB resource. This should be done when a batch …\nReturns the day number from 1 to 31.\nReturns days component.\nGets the Oracle type of elements if it is a collection. …\nReturns whether an element exists at the specified index.\nReturns the first index.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns fractional second precision.\nReturns the value of the element at the specified index.\nGets an value at the specified attribute.\nReturns the hour number from 0 to 23.\nReturns hours component.\nReturns an iterator visiting all indices in the collection.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nTrue when it is a collectoin. Otherwise false.\nReturns a boolean value indicating if the LOB resource has …\nReturns an iterator visiting all values with indices in …\nReturns the last index.\nReturns leading field precision.\nReturns the minute number from 0 to 59.\nReturns minutes component.\nReturns the month number from 1 to 12.\nReturns months component.\nGets the attribute name\nGets object name\nReturns the nanosecond number from 0 to 999,999,999.\nReturns nanoseconds component.\nCreates a new IntervalDS.\nCreates a new IntervalYM.\nReturns a reference to a new temporary LOB which may …\nReturns a reference to a new temporary CLOB which may …\nReturns a reference to a new temporary NCLOB which may …\nCreates a timestamp.\nCreate a new collection.\nCreate a new Oracle object.\nReturns the next index following the specified index.\nGets the number of attributes if it isn’t a collection. …\nReturns type information.\nReturns type information.\nOpens the LOB resource for writing. This will improve …\nGets the attribute type\nGets package name if it is a PL/SQL type. Otherwise, <code>None</code>.\nReturns precision.\nReturns precision\nReturns the previous index following the specified index.\nAppends an element to the end of the collection.\nGets rows as an iterator of <code>Row</code>s.\nGets rows as an itertor of the specified type.\nGets one row as <code>Row</code>.\nGets one row as the specified type.\nRemove the element at the specified index. Note that the …\nGets schema name\nReturns the second number from 0 to 59.\nReturns seconds component.\nSets the value to the element at the specified index.\nSets the value to the specified attribute.\nReturns the size of the data stored in the LOB.\nReturns the number of elements.\nTrims a number of elements from the end of a collection.\nShortens the data in the LOB so that it only contains the …\nReturns hour component of time zone.\nReturns minute component of time zone.\nReturns total time zone offset from UTC in seconds.\nReturns an iterator visiting all values in the collection.\nReturns true when the timestamp’s text representation …\nReturns the year number from -4713 to 9999.\nReturns years component.\nAn iterator over the indices of a Collection.\nAn iterator over the elements of a Collection.\nAn iterator over the values of a Collection.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.")