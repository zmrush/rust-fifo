use std::{ffi::CString, fs};

use async_fs_stream::AsyncFsStream;
mod async_fs_stream;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{ 
    let path  = "./fifo.sock";
    if std::path::Path::new(path).exists(){
        println!("prepare to remove old fifo file");
        let _ = tokio::fs::remove_file(path).await;
    }   
    let path_c = CString::new(path)?;
    let mod_ = libc::S_IRUSR |libc::S_IWUSR;
    let res = unsafe{
        libc::mkfifo(path_c.as_ptr() as *const libc::c_char, mod_)
    };
    if res!=0{
        println!("error:{}",std::io::Error::last_os_error());
        return Ok(());
    }
    let path_w = path.clone();
    tokio::spawn(async move{
        let fs = std::fs::OpenOptions::new().write(true).open(path_w).unwrap();
        let mut async_write_stream = AsyncFsStream::new(fs).unwrap();
        let mut stdin = tokio::io::stdin();
        let _ = tokio::io::copy(&mut stdin, &mut async_write_stream).await;
    });
    let path_r = path.clone();
    tokio::spawn(async move{
        println!("start to read");
        use std::os::unix::fs::OpenOptionsExt;
        let fs = std::fs::OpenOptions::new().read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path_r).unwrap();
        println!("open success");
        let mut async_read = AsyncFsStream::new(fs).unwrap();
        let mut stdout = tokio::io::stdout();
        let _ = tokio::io::copy(&mut async_read, &mut stdout).await;
        
    });
    loop{

    }
}
