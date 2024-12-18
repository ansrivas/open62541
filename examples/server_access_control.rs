use anyhow::Context as _;
use open62541::{ua, DefaultAccessControl, ServerBuilder, DEFAULT_PORT_NUMBER};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("Building server");

    // These files have been created with `server_ssl.sh`.
    let certificate_pem = include_str!("server_certificate.pem");
    let private_key_pem = include_str!("server_private_key.pem");

    let certificate = pem::parse(certificate_pem).context("parse PEM certificate")?;
    let private_key = pem::parse(private_key_pem).context("parse PEM private key")?;

    let (_, runner) = ServerBuilder::default_with_security_policies(
        DEFAULT_PORT_NUMBER,
        certificate.contents(),
        private_key.contents(),
    )
    .context("get server builder")?
    .access_control(DefaultAccessControl::new(
        false,
        &[
            (
                &ua::String::new("lorem").expect("create first username"),
                &ua::String::new("lorem123").expect("create first password"),
            ),
            (
                &ua::String::new("ipsum").expect("create second username"),
                &ua::String::new("ipsum123").expect("create second password"),
            ),
        ],
    ))
    .context("set access control")?
    .accept_all()
    .build();

    println!("Running server");

    runner.run()?;

    println!("Exiting");

    Ok(())
}
