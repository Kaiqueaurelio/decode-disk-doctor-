use serde::{Deserialize,Serialize};
use safety_policy::{validate_block_clone,OperationPlan,PlanConfirmation};
use sha2::{Digest,Sha256};
use std::io::{self,Read,Seek,SeekFrom,Write};
use thiserror::Error;
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]pub struct CopyReport{pub bytes_copied:u64,pub sha256:String}
#[derive(Debug,Error)]pub enum CloneError{
#[error("Unsafe plan: {0}")]UnsafePlan(String),#[error("Confirmation mismatch: {0}")]Confirmation(String),
#[error("Expected physical block devices.")]NotBlockDevices,#[error("Device is mounted or system-critical.")]MountedOrSystem,
#[error("Same kernel device.")]SameDevice,#[error("Byte count is zero.")]Empty,#[error("Requested length exceeds actual block-device capacity.")]CapacityMismatch,#[error("Destination read-back verification failed.")]VerificationFailed,#[error("The current device serial does not match the selected device identity.")]IdentityMismatch,
#[error("I/O: {0}")]Io(#[from]io::Error),#[error("Linux only.")]Unsupported}
pub fn copy_exact<R:Read,W:Write>(r:&mut R,w:&mut W,len:u64)->Result<CopyReport,CloneError>{
if len==0{return Err(CloneError::Empty)}let mut b=vec![0;1024*1024];let mut left=len;let mut done=0;let mut h=Sha256::new();
while left>0{let m=usize::try_from(left.min(b.len()as u64)).unwrap_or(b.len());let n=r.read(&mut b[..m])?;
if n==0{return Err(CloneError::Io(io::Error::new(io::ErrorKind::UnexpectedEof,"short source")))}
w.write_all(&b[..n])?;h.update(&b[..n]);done+=n as u64;left-=n as u64;}w.flush()?;
Ok(CopyReport{bytes_copied:done,sha256:format!("{:x}",h.finalize())})}
pub fn clone_block_device(p:&OperationPlan,c:&PlanConfirmation)->Result<CopyReport,CloneError>{
validate_block_clone(p).map_err(|e|CloneError::UnsafePlan(e.to_string()))?;
c.validate_for(p).map_err(|e|CloneError::Confirmation(e.to_string()))?;
if p.requested_bytes==0{return Err(CloneError::Empty)}
#[cfg(target_os="linux")]{use std::{fs::{self,OpenOptions},os::unix::fs::{FileTypeExt,MetadataExt},path::Path};
let sp=Path::new(&p.source.path);let dp=Path::new(&p.destination.path);
let sm=fs::metadata(sp)?;let dm=fs::metadata(dp)?;
if !sm.file_type().is_block_device()||!dm.file_type().is_block_device(){return Err(CloneError::NotBlockDevices)}
let sd=dev(sm.rdev());let dd=dev(dm.rdev());if sd==dd{return Err(CloneError::SameDevice)}
verify_identity(sm.rdev(),p.source.stable_identity.as_deref().unwrap_or(""))?;verify_identity(dm.rdev(),p.destination.stable_identity.as_deref().unwrap_or(""))?;
safe_mount(sd)?;safe_mount(dd)?;
let mut src=fs::File::open(sp)?;let src_capacity=block_capacity(&src)?;if p.requested_bytes>src_capacity{return Err(CloneError::CapacityMismatch)}
let mut dst=OpenOptions::new().read(true).write(true).open(dp)?;let dst_capacity=block_capacity(&dst)?;if p.requested_bytes>dst_capacity{return Err(CloneError::CapacityMismatch)}
let copied=copy_exact(&mut src,&mut dst,p.requested_bytes)?;dst.sync_all()?;
dst.seek(SeekFrom::Start(0))?;let verified=copy_exact(&mut dst,&mut io::sink(),p.requested_bytes)?;
if copied.sha256!=verified.sha256{return Err(CloneError::VerificationFailed)}Ok(copied)}
#[cfg(not(target_os="linux"))]{let _=(p,c);Err(CloneError::Unsupported)}}
#[cfg(target_os="linux")]fn block_capacity(file:&std::fs::File)->Result<u64,CloneError>{
use std::os::unix::io::AsRawFd;let mut bytes=0u64;
let rc=unsafe{libc::ioctl(file.as_raw_fd(),0x80081272 as libc::c_ulong,&mut bytes)};
if rc<0{return Err(CloneError::Io(io::Error::last_os_error()))}Ok(bytes)
}
#[cfg(target_os="linux")]fn dev(x:u64)->(u64,u64){(libc::major(x as libc::dev_t)as u64,libc::minor(x as libc::dev_t)as u64)}
#[cfg(target_os="linux")]fn disk_root(raw:u64)->Result<std::path::PathBuf,CloneError>{
let(a,b)=dev(raw);let p=std::fs::canonicalize(format!("/sys/dev/block/{a}:{b}"))?;
if p.join("partition").exists(){Ok(p.parent().unwrap_or(&p).to_path_buf())}else{Ok(p)}}
#[cfg(target_os="linux")]fn verify_identity(raw:u64,expected:&str)->Result<(),CloneError>{
let root=disk_root(raw)?;let serial=std::fs::read_to_string(root.join("device/serial")).unwrap_or_default();
let vendor=std::fs::read_to_string(root.join("device/vendor")).unwrap_or_default();
let actual=format!("serial:{}:{}",vendor.trim(),serial.trim());
if serial.trim().is_empty()||actual!=expected{return Err(CloneError::IdentityMismatch)}Ok(())}
#[cfg(target_os="linux")]fn safe_mount(raw:u64)->Result<(),CloneError>{
let target=disk_root(raw)?;
for l in std::fs::read_to_string("/proc/self/mountinfo")?.lines(){let f:Vec<&str>=l.split_whitespace().collect();if f.len()<5{continue}
let Some((a,b))=f[2].split_once(':')else{continue};let(Ok(a),Ok(b))=(a.parse::<u64>(),b.parse::<u64>())else{continue};
let mounted_raw=libc::makedev(a as u32,b as u32)as u64;
if let Ok(mounted)=disk_root(mounted_raw){if mounted==target{return Err(CloneError::MountedOrSystem)}}}
Ok(())}
#[cfg(test)]mod tests{use super::*;use std::io::Cursor;
#[test]fn copies_and_hashes(){let mut o=Vec::new();let r=copy_exact(&mut Cursor::new(b"12345678"),&mut o,8).unwrap();assert_eq!(o,b"12345678");assert_eq!(r.sha256.len(),64);}
#[test]fn short_read_errors(){assert!(matches!(copy_exact(&mut Cursor::new(b"x"),&mut Vec::new(),2),Err(CloneError::Io(ref e))if e.kind()==io::ErrorKind::UnexpectedEof));}
#[test]fn zero_length_rejected(){assert!(matches!(copy_exact(&mut Cursor::new(b"x"),&mut Vec::new(),0),Err(CloneError::Empty)));}}
