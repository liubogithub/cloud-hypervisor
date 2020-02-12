use crate::fuse;
use std::io;
use std::os::unix::io::RawFd;
use vhost_rs::vhost_user::message::{
    VhostUserFSSlaveMsg, VhostUserFSSlaveMsgFlags, VHOST_USER_FS_SLAVE_ENTRIES,
};
use vhost_rs::vhost_user::{SlaveFsCacheReq, VhostUserMasterReqHandler};

/// Trait for virtio-fs cache requests operations.  This is mainly used to hide
/// vhost-user details from virtio-fs's fuse part.
pub trait FsCacheReqHandler: Send + Sync + 'static {
    /// Setupmapping operation
    fn map(
        &mut self,
        foffset: u64,
        moffset: u64,
        len: u64,
        flags: u64,
        fd: RawFd,
    ) -> io::Result<()>;

    /// Removemapping operation
    fn unmap(&mut self, requests: Vec<fuse::RemovemappingOne>) -> io::Result<()>;
}

impl FsCacheReqHandler for SlaveFsCacheReq {
    fn map(
        &mut self,
        foffset: u64,
        moffset: u64,
        len: u64,
        flags: u64,
        fd: RawFd,
    ) -> io::Result<()> {
        let mut msg: VhostUserFSSlaveMsg = Default::default();
        msg.fd_offset[0] = foffset;
        msg.cache_offset[0] = moffset;
        msg.len[0] = len;
        msg.flags[0] = if (flags & fuse::SetupmappingFlags::WRITE.bits()) != 0 {
            VhostUserFSSlaveMsgFlags::MAP_W | VhostUserFSSlaveMsgFlags::MAP_R
        } else {
            VhostUserFSSlaveMsgFlags::MAP_R
        };

        self.fs_slave_map(&msg, fd)
    }

    fn unmap(&mut self, requests: Vec<fuse::RemovemappingOne>) -> io::Result<()> {
        let mut msg: VhostUserFSSlaveMsg = Default::default();
        let mut i = 0;

        for (ind, req) in requests.iter().enumerate() {
            i = ind % VHOST_USER_FS_SLAVE_ENTRIES;
            msg.len[i] = req.len;
            msg.cache_offset[i] = req.moffset;

            if i == VHOST_USER_FS_SLAVE_ENTRIES - 1 {
                self.fs_slave_unmap(&msg)?;
                msg = Default::default();
            }
        }

        if i < VHOST_USER_FS_SLAVE_ENTRIES - 1 {
            self.fs_slave_unmap(&msg)
        } else {
            Ok(())
        }
    }
}
