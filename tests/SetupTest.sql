whenever sqlerror exit failure

-- Get the path of this file.
-- See https://dba.stackexchange.com/a/168154/142186
set termout off
spool _SetupTest.tmp
@@_nonexistent_script.sql
spool off;
var path varchar2(100);
set serverout on
declare
  output varchar2(1000) := regexp_replace(replace(q'{
@_SetupTest.tmp
}',chr(10)),'.*"(.*)".*','\1');
begin
  if length(output) > 24 then
    :path:=substr(output,1,length(output)-24);
  else
    :path:='.';
  end if;
end;
/
col path new_val path
select :path path from dual;
set termout on

@&path/../odpi/test/sql/SetupTest.sql

create type &main_user..udt_issue19_obj as object (
    FloatCol                            float
);
/
create type &main_user..udt_issue19_col as varray(10) of float;
/
