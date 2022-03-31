#![allow(unused)]
extern crate dirs;
extern crate dotenv;

use std::{error::Error, str};
use std::env;
use std::fs::File;
use std::fs::create_dir;
use std::path::Path;
use std::io::prelude::*;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::Region;
use s3::BucketConfiguration;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut args = env::args().skip(1);
    let action = args.next().unwrap();
    let bucket_name = args.next().unwrap();

    let bucket_instance = BucketInstance::new(bucket_name);
    
    match action.as_str() {
        "create" => {
            bucket_instance.create_bucket().await;
        },
        "delete" => {
            bucket_instance.delete_bucket().await;
        },
        "delete-file" => {
            let file_path = args.next().unwrap();
            bucket_instance.delete_file_from_bucket(&file_path).await;
        },
        "send" => {
            let file_path = args.next().unwrap();
            bucket_instance.send_file_to_bucket(&file_path).await;
        },
        "get" => {
            let file_path = args.next().unwrap();
            bucket_instance.get_file_from_bucket(&file_path).await;
        },
        _ => println!("Comando não existente.")
    }
    
    Ok(())
}

struct BucketInstance {
    bucket: s3::bucket::Bucket
}

impl BucketInstance {
    fn new(bucket_name: String) -> BucketInstance {
        dotenv().ok();

        let access_key = env::var("ACCESS_KEY").unwrap();
        let secret_key = env::var("SECRET_KEY").unwrap();

        let bucket = Bucket::new_with_path_style(
            &bucket_name,
            Region::SaEast1,
            Credentials {
                access_key: Some(access_key),
                secret_key: Some(secret_key),
                security_token: None,
                session_token: None
            }
        ).unwrap();

        BucketInstance{ bucket }
    }

    fn create_app_folder(&self) {
        let mut folder_path = self.get_home_path();
        folder_path.push_str("/s3-cloud");

        let dir_exists = Path::new(&folder_path).is_dir();

        if !dir_exists {
            create_dir(folder_path).unwrap();
        }
    }

    fn get_home_path(&self) -> String {
        let home_path = match dirs::home_dir() {
            Some(dir) => dir.to_str().unwrap().to_owned(),
            None => {
                panic!("Caminho inacessível");
            }
        };

        home_path
    }

    async fn create_bucket(self) -> Result<(), Box<dyn std::error::Error>> {
        let (_, code) = self.bucket.head_object("/").await?;

        match code {
            404 => {
                let create_result = Bucket::create_with_path_style(
                    self.bucket.name.as_str(),
                    self.bucket.region(),
                    self.bucket.credentials().clone(),
                    BucketConfiguration::default()
                ).await?;

                println!("Bucket criado com sucesso!");
            },
            200 => {
                println!("Bucket já existe.");
            },
            301 => {
                println!("Nome já registrado.");
            },
            _ => println!("Erro desconhecido")
        }

        Ok(())
    }

    async fn delete_bucket(self) -> Result<(), Box<dyn std::error::Error>> {
        let delete_result = self.bucket.delete().await?;
        if delete_result == 204 {
            println!("Bucket excluído com sucesso.");
        } else {
            println!("Possível erro na exclusão do bucket.");
        }

        Ok(())
    }

    async fn send_file_to_bucket(self, file_path: &String) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = "/".to_owned();
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();

        path.push_str(&file_path);

        file.read_to_end(&mut buffer)?;
    
        let (_, code) = self.bucket.put_object(path, &buffer).await?;

        if code == 200 {
            println!("Arquivo enviado com sucesso para nuvem!");
        } else {
            println!("Possível erro ao enviar o arquivo para nuvem!");
        }

        Ok(())
    }

    async fn delete_file_from_bucket(self, file_path: &String) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = "/".to_owned();
        path.push_str(&file_path);

        let (_, code) = self.bucket.delete_object(path).await?;

        if code == 204 {
            println!("Arquivo excluído da nuvem com sucesso!");
        } else {
            println!("Possível erro na exclusão do arquivo em nuvem!");
            println!("{}", code);
        }

        Ok(())
    }

    async fn get_file_from_bucket(self, file_path: &String) -> Result<(), Box<dyn std::error::Error>> {
        self.create_app_folder();

        let mut to_save_path = self.get_home_path();
        to_save_path.push_str("/s3-cloud/");
        to_save_path.push_str(file_path);


        let (data, code) = self.bucket.get_object(file_path).await?;
        let mut file = File::create(to_save_path)?;
        file.write_all(&data);

        Ok(())
    }
}