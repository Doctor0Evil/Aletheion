#![no_std]

use crate::manifest::IDEISProjectManifest;
use crate::pos_nonfinancial::POSNonFinancialEvent;
use crate::session::IDEISSessionToken;

pub fn open_cross_repo_tunnel(
    _manifest: &IDEISProjectManifest,
    _tok: &IDEISSessionToken,
    _payload: &POSNonFinancialEvent,
) -> Result<(), ()> {
    Ok(())
}
