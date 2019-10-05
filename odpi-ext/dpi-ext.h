#ifndef ODPI_EXT_H
#define ODPI_EXT_H
#include "dpi.h"

#define DPI_OCI_SERVER_NOT_CONNECTED 0x0
#ifndef DPI_OCI_SERVER_NORMAL
#define DPI_OCI_SERVER_NORMAL 0x1
#endif

int dpi_ext_dpiStmt_getFnCode(dpiStmt *stmt, uint16_t *sqlfncode);
int dpi_ext_dpiConn_getServerStatus(dpiConn *conn, uint32_t *status);

#endif
