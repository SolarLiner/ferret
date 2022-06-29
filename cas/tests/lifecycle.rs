// Copyright (c) 2022 solarliner
// 
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

use ferret_cas::Cas;

#[tokio::test]
async fn lifecycle() {
    let path = std::env::temp_dir().join("cas-test");
    let cas = Cas::new(&path).await.unwrap();

    cas.set(&10);
    cas.set(&11);
    cas.set(&12);

    println!("Path: {}", path.display());
}
