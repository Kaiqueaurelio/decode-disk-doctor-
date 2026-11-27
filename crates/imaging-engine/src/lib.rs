use serde::{Deserialize,Serialize};
use safety_policy::{validate_restore,EndpointKind,JobKind,OperationPlan,PlanConfirmation,StorageEndpoint};
use sha2::{Digest,Sha256};use std::{fs::{self,File,OpenOptions},io::{self,Read,Seek,SeekFrom,Write},path::Path};use thiserror::Error;
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]pub struct ImageReport{pub bytes_processed:u64,pub sha256:String}
#[derive(Debug,Error)]pub enum ImagingError{
#[error("Invalid plan: {0}")]Plan(String),#[error("Confirmation: {0}")]Confirmation(String),#[error("Wrong endpoint type.")]WrongType,
#[error("Image and target share a physical device.")]SameDevice,#[error("Target is mounted/system-critical.")]UnsafeTarget,
#[error("Invalid image size or actual disk capacity mismatch.")]InvalidSize,
#[error("Destination read-back verification failed.")]VerificationFailed,#[error("Insufficient free space for a full raw image.")]InsufficientSpace,#[error("The current device serial does not match the selected device identity.")]IdentityMismatch,#[error("I/O: {0}")]Io(#[from]io::Error),#[error("Linux only.")]Unsupported}
fn stream<R:Read,W:Write>(r:&mut R,w:&mut W,len:u64)->Result<ImageReport,ImagingError>{if len==0{return Err(ImagingError::InvalidSize)}
let mut b=vec![0;1024*1024];let mut left=len;let mut done=0;let mut h=Sha256::new();while left>0{let m=usize::try_from(left.min(b.len()as u64)).unwrap_or(b.len());let n=r.read(&mut b[..m])?;
if n==0{return Err(ImagingError::Io(io::Error::new(io::ErrorKind::UnexpectedEof,"short source")))}w.write_all(&b[..n])?;h.update(&b[..n]);done+=n as u64;left-=n as u64;}w.flush()?;Ok(ImageReport{bytes_processed:done,sha256:format!("{:x}",h.finalize())})}
pub fn acquire_image_stream<R:Read,W:Write>(r:&mut R,w:&mut W,n:u64)->Result<ImageReport,ImagingError>{stream(r,w,n)}
pub fn restore_image_stream<R:Read,W:Write>(r:&mut R,w:&mut W,n:u64)->Result<ImageReport,ImagingError>{stream(r,w,n)}
pub fn create_raw_image(s:&StorageEndpoint,path:&Path,destination:&StorageEndpoint,n:u64)->Result<ImageReport,ImagingError>{
if s.kind!=EndpointKind::BlockDevice||destination.kind!=EndpointKind::BlockDevice
||s.stable_identity.as_deref().unwrap_or("").trim().is_empty()
||destination.stable_identity.as_deref().unwrap_or("").trim().is_empty()
||s.stable_identity==destination.stable_identity||n==0{return Err(ImagingError::Plan("Distinct source and destination block devices with stable identities are required.".into()))}
#[cfg(target_os="linux")]{use std::os::unix::fs::{FileTypeExt,MetadataExt};let sm=fs::metadata(&s.path)?;let dm=fs::metadata(&destination.path)?;
if !sm.file_type().is_block_device()||!dm.file_type().is_block_device(){return Err(ImagingError::WrongType)}
verify_identity(sm.rdev(),s.stable_identity.as_deref().unwrap_or(""))?;verify_identity(dm.rdev(),destination.stable_identity.as_deref().unwrap_or(""))?;safe_mount(sm.rdev())?;
let parent=path.parent().filter(|p|!p.as_os_str().is_empty()).unwrap_or(Path::new("."));let pm=fs::metadata(parent)?;
if same_physical_disk(sm.rdev(),dm.rdev())?{return Err(ImagingError::SameDevice)}
if !same_physical_disk(pm.dev(),dm.rdev())?{return Err(ImagingError::Plan("Image path is not on the selected destination disk.".into()))}
let mut src=File::open(&s.path)?;let actual=block_capacity(&src)?;if n!=actual{return Err(ImagingError::InvalidSize)}
if available_bytes(parent)?<actual{return Err(ImagingError::InsufficientSpace)}
let mut dst=OpenOptions::new().write(true).create_new(true).open(path)?;let report=stream(&mut src,&mut dst,actual)?;dst.sync_all()?;
let mut verify=File::open(path)?;let checked=stream(&mut verify,&mut io::sink(),actual)?;if checked.sha256!=report.sha256{return Err(ImagingError::VerificationFailed)}Ok(report)}
#[cfg(not(target_os="linux"))]{let _=(s,path,destination,n);Err(ImagingError::Unsupported)}}
pub fn restore_raw_image(p:&OperationPlan,c:&PlanConfirmation,path:&Path)->Result<ImageReport,ImagingError>{
if p.job_kind!=JobKind::Restore||p.source.kind!=EndpointKind::ImageFile||p.destination.kind!=EndpointKind::BlockDevice{return Err(ImagingError::Plan("Image file to block device required.".into()))}
validate_restore(p).map_err(|e|ImagingError::Plan(e.to_string()))?;c.validate_for(p).map_err(|e|ImagingError::Confirmation(e.to_string()))?;
#[cfg(target_os="linux")]{use std::os::unix::fs::{FileTypeExt,MetadataExt};let im=fs::symlink_metadata(path)?;
if !im.file_type().is_file()||im.file_type().is_symlink()||im.len()==0||im.len()!=p.requested_bytes{return Err(ImagingError::InvalidSize)}
let dm=fs::metadata(&p.destination.path)?;if !dm.file_type().is_block_device(){return Err(ImagingError::WrongType)}
let parent=path.parent().filter(|p|!p.as_os_str().is_empty()).unwrap_or(Path::new("."));let source_storage=fs::metadata(parent)?.dev();verify_identity(source_storage,p.source.stable_identity.as_deref().unwrap_or(""))?;verify_identity(dm.rdev(),p.destination.stable_identity.as_deref().unwrap_or(""))?;if same_physical_disk(source_storage,dm.rdev())?{return Err(ImagingError::SameDevice)}
safe_mount(dm.rdev())?;let mut src=File::open(path)?;let mut dst=OpenOptions::new().read(true).write(true).open(&p.destination.path)?;
if im.len()>block_capacity(&dst)?{return Err(ImagingError::InvalidSize)}
let report=stream(&mut src,&mut dst,im.len())?;dst.sync_all()?;dst.seek(SeekFrom::Start(0))?;
let verified=stream(&mut dst,&mut io::sink(),im.len())?;if report.sha256!=verified.sha256{return Err(ImagingError::VerificationFailed)}Ok(report)}
#[cfg(not(target_os="linux"))]{let _=(p,c,path);Err(ImagingError::Unsupported)}}
#[cfg(target_os="linux")]fn available_bytes(path:&Path)->Result<u64,ImagingError>{use std::{ffi::CString,os::unix::ffi::OsStrExt,mem::MaybeUninit};let p=CString::new(path.as_os_str().as_bytes()).map_err(|e|ImagingError::Plan(e.to_string()))?;let mut stats=MaybeUninit::<libc::statvfs>::zeroed();let rc=unsafe{libc::statvfs(p.as_ptr(),stats.as_mut_ptr())};if rc!=0{return Err(ImagingError::Io(io::Error::last_os_error()))}let s=unsafe{stats.assume_init()};Ok((s.f_bavail as u64).saturating_mul(s.f_frsize as u64))}
#[cfg(target_os="linux")]fn block_capacity(f:&File)->Result<u64,ImagingError>{use std::os::unix::io::AsRawFd;let mut n=0u64;let rc=unsafe{libc::ioctl(f.as_raw_fd(),0x80081272 as libc::c_ulong,&mut n)};if rc<0{return Err(ImagingError::Io(io::Error::last_os_error()))}Ok(n)}
#[cfg(target_os="linux")]fn sysfs_disk_root(raw:u64)->Result<std::path::PathBuf,ImagingError>{
let d=(libc::major(raw as libc::dev_t)as u64,libc::minor(raw as libc::dev_t)as u64);
let p=fs::canonicalize(format!("/sys/dev/block/{}:{}",d.0,d.1))?;
if p.join("partition").exists(){Ok(p.parent().unwrap_or(&p).to_path_buf())}else{Ok(p)}}
#[cfg(target_os="linux")]fn same_physical_disk(a:u64,b:u64)->Result<bool,ImagingError>{Ok(sysfs_disk_root(a)?==sysfs_disk_root(b)?)}
#[cfg(target_os="linux")]fn verify_identity(raw:u64,expected:&str)->Result<(),ImagingError>{let root=sysfs_disk_root(raw)?;let serial=fs::read_to_string(root.join("device/serial")).unwrap_or_default();let vendor=fs::read_to_string(root.join("device/vendor")).unwrap_or_default();let actual=format!("serial:{}:{}",vendor.trim(),serial.trim());if serial.trim().is_empty()||actual!=expected{return Err(ImagingError::IdentityMismatch)}Ok(())}
#[cfg(target_os="linux")]fn safe_mount(raw:u64)->Result<(),ImagingError>{
let target=sysfs_disk_root(raw)?;
for line in fs::read_to_string("/proc/self/mountinfo")?.lines(){let f:Vec<&str>=line.split_whitespace().collect();if f.len()<5{continue}
let Some((a,b))=f[2].split_once(':')else{continue};let(Ok(a),Ok(b))=(a.parse::<u64>(),b.parse::<u64>())else{continue};
let raw_mount=libc::makedev(a as u32,b as u32) as u64;
if let Ok(mounted)=sysfs_disk_root(raw_mount){if mounted==target{return Err(ImagingError::UnsafeTarget)}}}
Ok(())}
#[cfg(test)]mod tests{use super::*;use std::io::Cursor;
#[test]fn image_stream(){let x=vec![7;4096];let mut y=Vec::new();let r=acquire_image_stream(&mut Cursor::new(&x),&mut y,4096).unwrap();assert_eq!(x,y);assert_eq!(r.sha256.len(),64);}
#[test]fn restore_stream(){let x=vec![9;2048];let mut y=Vec::new();restore_image_stream(&mut Cursor::new(&x),&mut y,2048).unwrap();assert_eq!(x,y);}
#[test]fn truncated_errors(){assert!(matches!(restore_image_stream(&mut Cursor::new(b"x"),&mut Vec::new(),2),Err(ImagingError::Io(ref e))if e.kind()==io::ErrorKind::UnexpectedEof));}}
