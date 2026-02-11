use async_trait::async_trait;

#[async_trait]
pub trait PackageManager {
    async fn install(packages: Vec<String>);
}

macro_rules! package_managers {
    ($( $name:ident => $struct:ty ),* $(,)?) => {
        #[derive(serde::Deserialize, schemars::JsonSchema, Debug, Clone)]
        #[serde(rename_all = "lowercase")]
        pub enum PackageManagerName {
            $($name),*
        }

        pub async fn install(manager: PackageManagerName, packages: Vec<String>) {
            match manager {
                $(
                    PackageManagerName::$name => <$struct as PackageManager>::install(packages).await
                ),*
            }
        }
    };
}

// struct Brew;

// #[async_trait]
// impl PackageManager for Brew {
//     async fn install(packages: Vec<String>) {}
// }

// package_managers!(
//     Brew => Brew,
// );
