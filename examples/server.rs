use std::{
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

use anyhow::bail;
use open62541::{ua, Attribute, ObjectNode, Server, VariableNode};
use open62541_sys::{
    UA_NS0ID_BASEDATAVARIABLETYPE, UA_NS0ID_FOLDERTYPE, UA_NS0ID_OBJECTSFOLDER, UA_NS0ID_ORGANIZES,
    UA_NS0ID_STRING,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (server, runner) = Server::new();

    println!("Adding server nodes");

    let object_node = ObjectNode {
        requested_new_node_id: Some(ua::NodeId::string(1, "the.folder")),
        parent_node_id: ua::NodeId::ns0(UA_NS0ID_OBJECTSFOLDER),
        reference_type_id: ua::NodeId::ns0(UA_NS0ID_ORGANIZES),
        browse_name: ua::QualifiedName::new(1, "the folder"),
        type_definition: ua::NodeId::ns0(UA_NS0ID_FOLDERTYPE),
        attributes: ua::ObjectAttributes::default(),
    };
    let object_node_id = server.add_object_node(object_node)?;

    let variable_node = VariableNode {
        requested_new_node_id: Some(ua::NodeId::string(1, "the.answer")),
        parent_node_id: object_node_id.clone(),
        reference_type_id: ua::NodeId::ns0(UA_NS0ID_ORGANIZES),
        browse_name: ua::QualifiedName::new(1, "the answer"),
        type_definition: ua::NodeId::ns0(UA_NS0ID_BASEDATAVARIABLETYPE),
        attributes: ua::VariableAttributes::default()
            .with_data_type(&ua::NodeId::ns0(UA_NS0ID_STRING)),
    };
    let variable_node_id = server.add_variable_node(variable_node)?;

    server.write_value(
        &variable_node_id,
        &ua::Variant::scalar(ua::String::new("foobar")?),
    )?;

    read_attribute(&server, &variable_node_id, ua::AttributeId::NODEID_T)?;
    read_attribute(&server, &variable_node_id, ua::AttributeId::NODECLASS_T)?;
    read_attribute(&server, &variable_node_id, ua::AttributeId::BROWSENAME_T)?;
    read_attribute(&server, &variable_node_id, ua::AttributeId::DISPLAYNAME_T)?;

    for attribute in [ua::AttributeId::DESCRIPTION, ua::AttributeId::WRITEMASK] {
        read_attribute(&server, &variable_node_id, &attribute)?;
    }

    let (cancel_tx, cancel_rx) = mpsc::channel();

    // Start background task that simulates changing variable values.
    let server_task_handle = thread::spawn(move || -> anyhow::Result<()> {
        println!("Simulating values");
        loop {
            for value in ["foo", "bar", "baz"] {
                match cancel_rx.recv_timeout(Duration::from_millis(1000)) {
                    Ok(()) => return Ok(()),
                    Err(RecvTimeoutError::Timeout) => {
                        // Continue and simulate next updated value below, then repeat loop.
                    }
                    Err(RecvTimeoutError::Disconnected) => bail!("main task should be running"),
                }
                server.write_value(
                    &variable_node_id,
                    &ua::Variant::scalar(ua::String::new(value)?),
                )?;
            }
        }
    });

    // Start runner task that handles incoming connections (events).
    let runner_task_handle = thread::spawn(|| -> anyhow::Result<()> {
        println!("Running server");
        runner.run()?;
        Ok(())
    });

    // Wait for runner task to finish eventually (SIGINT/Ctrl+C).
    if let Err(err) = runner_task_handle
        .join()
        .expect("runner task should not panic")
    {
        println!("Runner task failed: {err}");
    }

    println!("Exiting");

    cancel_tx.send(()).expect("server task should be running");

    // Wait for simulation task to shut down after cancelling.
    if let Err(err) = server_task_handle
        .join()
        .expect("server task should not panic")
    {
        println!("Server task failed: {err}");
    }

    println!("Done");

    Ok(())
}

fn read_attribute(
    server: &Server,
    node_id: &ua::NodeId,
    attribute: impl Attribute,
) -> anyhow::Result<()> {
    let value = server.read_attribute(node_id, attribute)?;

    println!("- attribute {attribute:?} has value {value:?}");

    Ok(())
}
