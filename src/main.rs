use std::{ffi::CString, fs, io::Read, os::unix::fs::OpenOptionsExt};

use async_fs_stream::AsyncFsStream;
use futures::FutureExt;
use tokio::io::AsyncReadExt;
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
    //write 设置成nonblock，如果read没有打开的话，就会报错 { code: 6, kind: Uncategorized, message: "Device not configured" }
    // let fs = std::fs::OpenOptions::new().write(true).custom_flags(libc::O_NONBLOCK).open(path_w).unwrap();
    // 1 配合下面的4进行异常写 导致下面读取 抛出异常 { code: 35, kind: WouldBlock, message: "Resource temporarily unavailable" }
    tokio::spawn(async move{
        let fs = std::fs::OpenOptions::new().write(true).open(path_w).unwrap();
        let mut async_write_stream = AsyncFsStream::new(fs).unwrap();
        async_write_stream.write(b"hello world ping ping pong").await;
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    });
    //2
    //正常写
    // tokio::spawn(async move{
    //     let fs = std::fs::OpenOptions::new().write(true).open(path_w).unwrap();
    //     let mut async_write_stream = AsyncFsStream::new(fs).unwrap();
    //     let mut stdin = tokio::io::stdin();
    //     let _ = tokio::io::copy(&mut stdin, &mut async_write_stream).await;
    // });
    let path_r = path.clone();
    //3 
    //正常读
    // tokio::spawn(async move{
    //     println!("start to read");
    //     use std::os::unix::fs::OpenOptionsExt;
    //     let fs = std::fs::OpenOptions::new().read(true)
    //     .custom_flags(libc::O_NONBLOCK)
    //     .open(path_r).unwrap();
    //     println!("open success");
    //     let mut async_read = AsyncFsStream::new(fs).unwrap();
    //     let mut stdout = tokio::io::stdout();
    //     let _ = tokio::io::copy(&mut async_read, &mut stdout).await;
    //     println!("end to read");
        
    // });
    //4 
    tokio::spawn(async move{
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        // 不管是用 tokio fs还是用std fs两者本质都是 block读的方式，而不是采用 select/epoll这种 non block的方式，所以报错都是一样的
        //这个是为了配合 1 进行异常读 抛出异常 { code: 35, kind: WouldBlock, message: "Resource temporarily unavailable" }
        // 出错的原因是 这里因为设置了 文件描述符 是 nonblock，所以 这里的读总是直接返回的，但是当 上面1 的 write的文件符 是存活的时候，要么能读取到写的东西，要么就抛出 { code: 35, kind: WouldBlock, message: "Resource temporarily unavailable" }
        // 但是 当上面1 的 write的 描述符 的生命周期结束关闭后，这里又变成读取到 0字节的bytes，所以 我这里的示例 后面就会再次输出 eof
        let mut fs = tokio::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(path_r).await.unwrap();
        let mut buffer = [0; 10];
        loop{
            let n = fs.read(&mut buffer).await;
            if n.is_err(){
                println!("read error:{:?}",n.err());
                println!("pppp");
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }else {
                let n = n.unwrap();
                if n >0{
                    println!("read:{}",String::from_utf8_lossy(&buffer[0..n]).into_owned());
                }else{
                    println!("read eof");
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    // break;
                }
                
            }
        }
        // let mut fs = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK).open(path_r).unwrap();
        // let mut buffer = [0; 10];
        // loop{
        //     let n = fs.read(&mut buffer);
        //     if n.is_err(){
        //         println!("pppp");
        //         println!("read error:{:?}",n.err());
        //         // break;
        //         tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        //     }else {
        //         let n = n.unwrap();
        //         if n >0{
        //             println!("read:{}",String::from_utf8_lossy(&buffer[0..n]).into_owned());
        //         }else{
        //             println!("read eof");
        //             // break;
        //             tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        //         }
                
        //     }
        // }
        
    });
    loop{

    }
}
