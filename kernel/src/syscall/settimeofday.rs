// SPDX-License-Identifier: MPL-2.0

use super::SyscallReturn;
use crate::{
    prelude::*,
    time::{timeval_t, SystemTime},
};

// The use of the timezone structure is obsolete.
// Glibc sets the timezone_addr argument to NULL, so just ignore it.
pub fn sys_settimeofday(
    timeval_addr: Vaddr,
    /* timezone_addr: Vaddr, */ ctx: &Context,
) -> Result<SyscallReturn> {
    unimplemented!();

    Ok(SyscallReturn::Return(0))
}
