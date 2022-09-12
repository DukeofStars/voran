use voran::{
    package::{InstallInfo, Package, PackageType},
    packages::{GetPackages, Packages},
};

use tokio::fs;

#[tokio::test]
async fn get_packages_works() {
    // Initialise files
    fs::create_dir("packages/").await.unwrap();
    fs::create_dir("packages/my-package").await.unwrap();
    fs::create_dir("packages/my-package/0.1.0").await.unwrap();
    let res_package = Package {
        name: "my-package".to_string(),
        friendly_name: "My Package".to_string(),
        version: "0.1.0".to_string(),
        install: InstallInfo {
            url: "https://google.com/index.html".to_string(),
            type_: PackageType::JellyFish,
        },
    };
    fs::write(
        "packages/my-package/0.1.0/package.toml",
        toml::to_string(&res_package).unwrap(),
    )
    .await
    .unwrap();

    let get_packages = GetPackages::new("packages/");

    let lazy = get_packages.lazy().await.unwrap();
    let mut my_package = lazy
        .get_package("my-package")
        .expect("This package does exist");
    let other_package = lazy.get_package("other-package");
    if other_package.is_some() {
        panic!("This package should not exist");
    }
    let package = my_package.package();
    if package.is_some() {
        panic!("There is no package.toml there");
    }
    my_package.version("0.1.0").unwrap();
    let package = my_package.package().unwrap();
    if package != res_package {
        panic!("Packages should be the same");
    }

    fs::remove_dir_all("packages/").await.unwrap();
}
