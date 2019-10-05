#include "dpiImpl.h"
#include "dpi-ext.h"

#define OCI_ATTR_SQLFNCODE 10

/* Copied from dpiConn.c */
static int dpiConn__check(dpiConn *conn, const char *fnName, dpiError *error)
{
    if (dpiGen__startPublicFn(conn, DPI_HTYPE_CONN, fnName, error) < 0)
        return DPI_FAILURE;
    return dpiConn__checkConnected(conn, error);
}

int dpi_ext_dpiStmt_getFnCode(dpiStmt *stmt, uint16_t *sqlfncode)
{
    dpiError error;
    int status;

    if (dpiGen__startPublicFn(stmt, DPI_HTYPE_STMT, __func__, &error) < 0)
        return dpiGen__endPublicFn(stmt, DPI_FAILURE, &error);
    status = dpiOci__attrGet(stmt->handle, DPI_OCI_HTYPE_STMT, sqlfncode, 0,
            OCI_ATTR_SQLFNCODE, "get sql function code", &error);
    return dpiGen__endPublicFn(stmt, status, &error);
}

int dpi_ext_dpiConn_getServerStatus(dpiConn *conn, uint32_t *server_status)
{
    dpiError error;
    int status;

    if (dpiConn__check(conn, __func__, &error) < 0)
        return dpiGen__endPublicFn(conn, DPI_FAILURE, &error);
    status = dpiOci__attrGet(conn->serverHandle, DPI_OCI_HTYPE_SERVER, server_status, 0,
            DPI_OCI_ATTR_SERVER_STATUS, "get server status", &error);
    return dpiGen__endPublicFn(conn, status, &error);
}

