use crate::bencode::Item;
use std::{collections::BTreeMap, path::Path, str::from_utf8, sync::Arc};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    sync::Mutex as TokioMutex,
};

#[derive(Debug, Clone)]
pub struct FileSize {
    file: Arc<TokioMutex<File>>,
    len: usize,
}

pub async fn parse_file(info: &BTreeMap<Vec<u8>, Item>) -> (Arc<Vec<FileSize>>, usize) {
    // single file -> only single file owns length field.
    if let Some(s) = info.get("length".as_bytes()) {
        // file length
        let file_length: usize = s.get_integer();
        // name of the file
        let file_name = info.get("name".as_bytes()).unwrap().get_string();
        // create file and return
        let path = Path::new(std::str::from_utf8(&file_name).unwrap());
        let dest = Arc::new(TokioMutex::new(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(path)
                .await
                .unwrap(),
        ));

        let file_size = FileSize {
            file: dest,
            len: file_length,
        };

        (Arc::new(vec![file_size]), file_length)
    } else {
        // multiple files
        // get parent folder and file dicts.
        let name = info.get("name".as_bytes()).unwrap().get_string();
        let files = info.get("files".as_bytes()).unwrap().get_list();
        let mut ret: Vec<FileSize> = vec![];
        // for each dict
        for f in files {
            let dict = f.get_dict();
            // get file length
            let len = dict.get("length".as_bytes()).unwrap().get_integer();
            // get file path
            let mut path_list = dict.get("path".as_bytes()).unwrap().get_list();

            // get end file name
            let end_file = path_list.pop().unwrap().get_string();
            let file_name = from_utf8(&end_file).unwrap();
            // parent folders to the filename
            let mut base = "./".to_string() + from_utf8(&name).unwrap();
            for folder in path_list {
                let folder_name = "/".to_string() + from_utf8(&folder.get_string()).unwrap();
                base.push_str(&folder_name)
            }

            // crreat parents and file
            create_dir_all(base.clone()).await.unwrap();
            let full_path = base + "/" + file_name;
            let file_path = Path::new(&full_path);
            let file = Arc::new(TokioMutex::new(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(file_path)
                    .await
                    .unwrap(),
            ));
            ret.push(FileSize { file, len })
        }
        // get total length from each file
        let mut total_len: usize = 0;
        for filesize in &ret {
            total_len += filesize.len;
        }
        (Arc::new(ret), total_len)
    }
}
