#![allow(dead_code)]

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use shaku::{module, Component, HasComponent, Interface};

pub trait DatabaseConnection: Interface {
    fn connect(&self) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = DatabaseConnection)]
pub struct DatabaseConnectionImpl {
    connection_string: String,
}

impl DatabaseConnection for DatabaseConnectionImpl {
    fn connect(&self) -> Result<()> {
        println!("connect: {}", self.connection_string);
        Ok(())
    }
}

#[derive(Debug)]
pub struct User {
    name: String,
}

#[async_trait]
pub trait UserRepository: Interface {
    async fn find_user(&self, id: String) -> Result<Option<User>>;
    fn update(&self, user: User) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = UserRepository)]
pub struct UserRepositoryImpl {
    #[shaku(inject)]
    connection: Arc<dyn DatabaseConnection>,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_user(&self, id: String) -> Result<Option<User>> {
        self.connection.connect().unwrap();

        Ok(Some(User {
            name: format!("test_user_{}", id),
        }))
    }

    fn update(&self, _user: User) -> Result<()> {
        todo!()
    }
}

#[async_trait]
pub trait UserService: Interface {
    async fn find_user(&self, id: String) -> Result<Option<User>>;
    fn deactivate_user(&self, id: String) -> Result<()>;
}

#[derive(Component)]
#[shaku(interface = UserService)]
pub struct UserServiceImpl {
    #[shaku(inject)]
    user_repository: Arc<dyn UserRepository>,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn find_user(&self, id: String) -> Result<Option<User>> {
        self.user_repository.find_user(id).await
    }

    fn deactivate_user(&self, _id: String) -> Result<()> {
        todo!()
    }
}

module! {
    pub AppModule {
        components = [UserServiceImpl, UserRepositoryImpl, DatabaseConnectionImpl],
        providers = []
    }
}

#[tokio::main]
async fn main() {
    let module = AppModule::builder()
        .with_component_parameters::<DatabaseConnectionImpl>(DatabaseConnectionImplParameters {
            connection_string: "User ID=user;Password=password;Host=localhost;Port=5432;Database=myDataBase;Pooling=true;Min Pool Size=0;Max Pool Size=100;Connection Lifetime=0;".to_owned()
        })
    .build();

    let user_service: &dyn UserService = module.resolve_ref();
    let user = user_service.find_user("id001".to_owned()).await.unwrap();

    println!("result: {:?}", user);
}
