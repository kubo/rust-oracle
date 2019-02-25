#!/bin/bash

set -ev

"$ORACLE_HOME/bin/sqlplus" / as sysdba @odpi/test/sql/SetupTest.sql < /dev/null

