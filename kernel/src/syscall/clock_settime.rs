use super::SyscallReturn;
use crate::{
    prelude::*,
    time::{
        clockid_t,
    },
};

pub fn sys_clock_settime(
    clockid: clockid_t,
    timespec_addr: Vaddr,
    ctx: &Context,
) -> Result<SyscallReturn> {
    debug!("clockid = {:?}", clockid);
    unimplemented!();
    Ok(SyscallReturn::Return(0))
}
