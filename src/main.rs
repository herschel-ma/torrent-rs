use std::error::Error;

mod bencode;
mod field;
mod file;
mod hash;
mod tcp_bt;
mod torrent;
mod tracker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // get torrent file from command line
    let args = std::env::args().collect::<Vec<String>>();
    let arg = if let Some(s) = args.get(1) {
        s
    } else {
        panic!("no torrent file specified");
    };

    // read bytes from torrent file
    let bytes: Vec<u8> = tokio::fs::read(arg).await?;

    // create torrent object to parse torrent file
    let client = torrent::Client::new(&bytes).await;
    // download torrent
    client.start().await;
    Ok(())
}

#[cfg(test)]
mod client_test {
    #[test]
    #[ignore]
    fn test_read_torrent_file() {
        use super::*;
        use tokio_test;
        let file_path = "./Elewder.torrent";
        tokio_test::block_on(async {
            let content_bytes = tokio::fs::read(&file_path).await.unwrap();
            let client = torrent::Client::new(&content_bytes).await;
            dbg!(client);
        })
    }

    #[test]
    #[ignore]
    fn test_print_0x_to_decimal() {
        let hex_str = "0x4000";
        let dec = 16384;
        assert_eq!(format!("{:#x}", dec), hex_str);
        let mut msg = vec![1, 2, 3];
        assert_eq!(msg.drain(0..2).collect::<Vec<_>>(), &[1, 2]);
    }
}
