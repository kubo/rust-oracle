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

-- Get compatible initialization parameter
var compat_ver number
begin
select to_number(regexp_replace(value, '^([[:digit:]]+).*', '\1')) * 100 +
       to_number(regexp_replace(value, '^[[:digit:]]+\.([[:digit:]]+).*', '\1'))
       into :compat_ver
  from v$parameter where name = 'compatible';
end;
/

create type &main_user..udt_issue19_obj as object (
    FloatCol                            float
);
/
create type &main_user..udt_issue19_col as varray(10) of float;
/
begin
  if :compat_ver >= 2300 then
     execute immediate 'drop table if exists &main_user..test_vector_type purge';
     execute immediate 'create table &main_user..test_vector_type(id integer, vec vector, fixed_dim vector(2, *), f32 vector(*, float32), f64 vector(4, float64), i8 vector(*, int8))';
  end if;
  if :compat_ver >= 2305 then
     execute immediate 'drop table if exists &main_user..test_binary_vector purge';
     execute immediate 'create table &main_user..test_binary_vector(id integer, vec vector(*, binary))';
  end if;
end;
/
