#![no_std]

use crate::manifest::IDEISProjectManifest;
use crate::pos_nonfinancial::POSNonFinancialEvent;

pub fn immutable_searchtrace_log(
    _manifest: &IDEISProjectManifest,
    _payload: &POSNonFinancialEvent,
) -> Result<(), ()> {
    Ok(())
}
