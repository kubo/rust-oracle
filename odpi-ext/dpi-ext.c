#include "dpi-ext.h"
#include "dpiImpl.h"

#define OCI_ATTR_SQLFNCODE 10

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
