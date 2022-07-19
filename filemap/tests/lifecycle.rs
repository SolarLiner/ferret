// Copyright (c) 2022 solarliner
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use ferret_filemap::Filemap;

#[tokio::test]
async fn lifecycle() {
    let path = std::env::temp_dir().join("cas-test");
    let cas = Filemap::<u8, u8>::new(&path).await.unwrap();

    cas.insert(10, 11).await.unwrap();

    assert!(cas.contains(&10).await.unwrap());
    assert!(!cas.contains(&11).await.unwrap());

    assert_eq!(cas.get(&10).await.unwrap(), Some(11));

    tokio::fs::remove_dir_all(path).await.unwrap();
}
