#!/bin/bash

set -ev

"$ORACLE_HOME/bin/sqlplus" / as sysdba @tests/SetupTest.sql < /dev/null

