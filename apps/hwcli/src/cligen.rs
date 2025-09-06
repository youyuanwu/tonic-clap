use tonic_clap_tests::helloworld::{self, EnumOk, HelloRequest};

#[derive(clap::Subcommand, Debug)]
pub enum CommandServices {
    /// greeter service
    #[command(subcommand)]
    Greeter(GreeterCommands),
    #[command(subcommand)]
    Greeter2(Greeter2Commands),
}

#[derive(clap::Subcommand, Debug)]
pub enum GreeterCommands {
    SayHello(HelloRequestArg),
    SayHello2(HelloRequest2Arg),
}

#[derive(clap::Subcommand, Debug)]
pub enum Greeter2Commands {
    SayHello(HelloRequestArg),
    SayHello2(HelloRequest2Arg),
}

impl GreeterCommands {
    async fn execute(
        &self,
        ch: tonic::transport::Channel,
        json_data: Option<String>,
    ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {
        let mut c = helloworld::greeter_client::GreeterClient::new(ch);
        match self {
            GreeterCommands::SayHello(args) => {
                let mut request: HelloRequest = match json_data {
                    Some(data) => serde_json::from_str(&data).unwrap(),
                    None => Default::default(),
                };
                args.apply(&mut request);
                Ok(Box::new(c.say_hello(request).await?.into_inner()))
            }
            GreeterCommands::SayHello2(args) => {
                let request = tonic::Request::new((*args).clone().into());
                Ok(Box::new(c.say_hello2(request).await?.into_inner()))
            }
        }
    }
}

impl Greeter2Commands {
    async fn execute(
        &self,
        ch: tonic::transport::Channel,
    ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {
        let mut c = helloworld::greeter2_client::Greeter2Client::new(ch);
        match self {
            Greeter2Commands::SayHello(args) => {
                let mut request = HelloRequest::default();
                args.apply(&mut request);
                Ok(Box::new(c.say_hello(request).await?.into_inner()))
            }
            Greeter2Commands::SayHello2(args) => {
                let request = tonic::Request::new((*args).clone().into());
                Ok(Box::new(c.say_hello2(request).await?.into_inner()))
            }
        }
    }
}

impl CommandServices {
    pub async fn execute(
        &self,
        ch: tonic::transport::Channel,
        json_data: Option<String>,
    ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {
        match self {
            CommandServices::Greeter(cmd) => cmd.execute(ch, json_data).await,
            CommandServices::Greeter2(cmd) => cmd.execute(ch).await,
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct HelloRequestArg {
    #[arg(short, long)]
    name: Option<String>,
}

impl HelloRequestArg {
    pub fn apply(&self, r: &mut HelloRequest) {
        self.name.clone().inspect(|opt| r.name = opt.to_owned());
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct HelloRequest2Arg {
    #[arg(long, default_value = "")]
    name: String,

    #[command(flatten)]
    field1: Option<Field1Arg>,

    #[arg(long, default_value = "")]
    field2: Vec<String>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct Field1Arg {
    #[arg(long, default_value = "")]
    fname: String,
    #[arg(long, default_value = "0")]
    fcount: i32,
}

impl From<HelloRequest2Arg> for helloworld::HelloRequest2 {
    fn from(value: HelloRequest2Arg) -> Self {
        Self {
            name: value.name,
            field1: value.field1.map(|f| f.into()),
            field2: value.field2,
            field3: EnumOk::Ok1.into(),
            opt_string: None,
            one_of_field: Some(helloworld::hello_request2::OneOfField::OneOf1(
                helloworld::OneOf1 {
                    one_of_str: "one_of_str_value".to_string(),
                },
            )),
        }
    }
}

impl From<Field1Arg> for helloworld::Field1 {
    fn from(value: Field1Arg) -> Self {
        Self {
            fname: value.fname,
            fcount: value.fcount,
        }
    }
}
