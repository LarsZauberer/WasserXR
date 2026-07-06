//! HTTP MCP server for remote scene inspection and editing.
//!
//! Built on the official `rmcp` SDK with the streamable HTTP transport.
//! Tool calls are turned into closures and sent to the scene thread over a
//! channel; `Scene::tick` executes them at the end of each tick and pipes
//! the result back to the waiting tool call.

use std::net::TcpListener;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::JoinHandle;
use std::time::Duration;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData, ServerHandler, schemars, tool, tool_handler, tool_router};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::error::SceneError;
use crate::scene::Scene;

/// TCP port the MCP server listens on.
pub const MCP_PORT: u16 = 4512;

/// How long a tool call waits for the scene to process it before giving up.
const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

type McpAction = Box<dyn FnOnce(&mut Scene) -> Result<Value, String> + Send>;

struct McpRequest {
    action: McpAction,
    reply: tokio::sync::oneshot::Sender<Result<Value, String>>,
}

/// Handle to the running MCP server thread. Dropping it stops the server.
pub(crate) struct McpServer {
    receiver: Receiver<McpRequest>,
    cancel: CancellationToken,
    thread: Option<JoinHandle<()>>,
}

impl Drop for McpServer {
    fn drop(&mut self) {
        self.cancel.cancel();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Scene {
    /// Starts the HTTP MCP server on port 4512 (endpoint `/mcp`).
    ///
    /// Received tool calls are queued and processed at the end of each
    /// `tick`, so the scene must keep ticking for the server to respond.
    pub fn activate_mcp(&mut self) -> Result<(), SceneError> {
        if self.mcp.is_some() {
            crate::warn!(self, "MCP server is already running");
            return Ok(());
        }

        let listener = TcpListener::bind(("127.0.0.1", MCP_PORT))
            .map_err(|error| SceneError::Mcp(error.to_string()))?;
        listener
            .set_nonblocking(true)
            .map_err(|error| SceneError::Mcp(error.to_string()))?;

        let (sender, receiver) = channel();
        let cancel = CancellationToken::new();
        let server_cancel = cancel.clone();
        let thread = std::thread::spawn(move || serve(listener, sender, server_cancel));

        self.mcp = Some(McpServer {
            receiver,
            cancel,
            thread: Some(thread),
        });
        crate::info!(self, "MCP server listening on port {}", MCP_PORT);
        Ok(())
    }

    /// Stops the HTTP MCP server if it is running.
    pub fn deactivate_mcp(&mut self) {
        if self.mcp.take().is_some() {
            crate::info!(self, "MCP server stopped");
        }
    }

    /// Executes queued MCP tool calls. Called at the end of `tick`.
    pub(crate) fn handle_mcp_commands(&mut self) {
        // Take the server out so the actions can borrow the scene mutably.
        // No action touches `self.mcp`, so putting it back is safe.
        let Some(mcp) = self.mcp.take() else {
            return;
        };

        while let Ok(request) = mcp.receiver.try_recv() {
            let result = (request.action)(self);
            let _ = request.reply.send(result);
        }

        self.mcp = Some(mcp);
    }
}

/// Runs the rmcp streamable HTTP server until the token is cancelled.
fn serve(listener: TcpListener, sender: Sender<McpRequest>, cancel: CancellationToken) {
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return;
    };

    runtime.block_on(async move {
        let Ok(listener) = tokio::net::TcpListener::from_std(listener) else {
            return;
        };

        let service: StreamableHttpService<WasserXrMcp, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(WasserXrMcp::new(sender.clone())),
                Default::default(),
                StreamableHttpServerConfig::default().with_cancellation_token(cancel.child_token()),
            );

        let router = axum::Router::new().nest_service("/mcp", service);
        let _ = axum::serve(listener, router)
            .with_graceful_shutdown(cancel.cancelled_owned())
            .await;
    });
}

fn scene_err(error: SceneError) -> String {
    format!("{error:?}")
}

fn parse_uuid(id: &str) -> Result<Uuid, ErrorData> {
    Uuid::parse_str(id)
        .map_err(|error| ErrorData::invalid_params(format!("invalid entity uuid: {error}"), None))
}

#[derive(Deserialize, schemars::JsonSchema)]
struct AddEntityArgs {
    /// Optional display name for the new entity
    name: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct EntityArgs {
    /// Entity uuid
    entity_id: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct RenameArgs {
    /// Entity uuid
    entity_id: String,
    /// New display name
    name: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct ComponentArgs {
    /// Entity uuid
    entity_id: String,
    /// Component id
    component_id: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct SetFieldArgs {
    /// Entity uuid
    entity_id: String,
    /// Component id
    component_id: String,
    /// Field id
    field_id: String,
    /// New value as text
    value: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct AddSystemArgs {
    /// System id
    system_id: String,
    /// Priority, higher runs earlier (default 0)
    priority: Option<usize>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct SystemArgs {
    /// System id
    system_id: String,
}

#[derive(Clone)]
struct WasserXrMcp {
    sender: Sender<McpRequest>,
    tool_router: ToolRouter<Self>,
}

impl WasserXrMcp {
    fn new(sender: Sender<McpRequest>) -> Self {
        Self {
            sender,
            tool_router: Self::tool_router(),
        }
    }

    /// Sends an action to the scene thread and waits for the piped-back result.
    async fn run(
        &self,
        action: impl FnOnce(&mut Scene) -> Result<Value, String> + Send + 'static,
    ) -> Result<CallToolResult, ErrorData> {
        let (reply, replies) = tokio::sync::oneshot::channel();
        self.sender
            .send(McpRequest {
                action: Box::new(action),
                reply,
            })
            .map_err(|_| ErrorData::internal_error("scene command channel is closed", None))?;

        match tokio::time::timeout(REPLY_TIMEOUT, replies).await {
            Ok(Ok(Ok(value))) => Ok(CallToolResult::success(vec![ContentBlock::text(
                value.to_string(),
            )])),
            Ok(Ok(Err(message))) => Ok(CallToolResult::error(vec![ContentBlock::text(message)])),
            _ => Err(ErrorData::internal_error(
                "the scene did not process the command in time (is the scene ticking?)",
                None,
            )),
        }
    }
}

#[tool_router]
impl WasserXrMcp {
    #[tool(description = "List all entities with their ids, names, and attached component ids")]
    async fn list_entities(&self) -> Result<CallToolResult, ErrorData> {
        self.run(|scene| {
            let entities: Vec<Value> = scene
                .get_entities()
                .into_iter()
                .map(|id| {
                    json!({
                        "id": id.to_string(),
                        "name": scene.get_entity_name(id).unwrap_or(""),
                        "components": scene.get_entity_components(id).unwrap_or_default(),
                    })
                })
                .collect();
            Ok(json!(entities))
        })
        .await
    }

    #[tool(description = "Add a new entity and return its id, optionally setting a display name")]
    async fn add_entity(
        &self,
        Parameters(args): Parameters<AddEntityArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run(move |scene| {
            let id = scene.add_entity();
            if let Some(name) = args.name {
                scene.set_entity_name(id, name).map_err(scene_err)?;
            }
            Ok(json!({ "id": id.to_string() }))
        })
        .await
    }

    #[tool(description = "Remove an entity and all of its components")]
    async fn remove_entity(
        &self,
        Parameters(args): Parameters<EntityArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let entity = parse_uuid(&args.entity_id)?;
        self.run(move |scene| {
            scene.remove_entity(entity).map_err(scene_err)?;
            Ok(json!("ok"))
        })
        .await
    }

    #[tool(description = "Set the display name of an entity")]
    async fn set_entity_name(
        &self,
        Parameters(args): Parameters<RenameArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let entity = parse_uuid(&args.entity_id)?;
        self.run(move |scene| {
            scene
                .set_entity_name(entity, args.name)
                .map_err(scene_err)?;
            Ok(json!("ok"))
        })
        .await
    }

    #[tool(description = "Add a component to an entity by component id")]
    async fn add_component(
        &self,
        Parameters(args): Parameters<ComponentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let entity = parse_uuid(&args.entity_id)?;
        self.run(move |scene| {
            scene
                .add_component(entity, args.component_id)
                .map_err(scene_err)?;
            Ok(json!("ok"))
        })
        .await
    }

    #[tool(description = "Remove a component from an entity")]
    async fn remove_component(
        &self,
        Parameters(args): Parameters<ComponentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let entity = parse_uuid(&args.entity_id)?;
        self.run(move |scene| {
            scene
                .remove_component(entity, &args.component_id)
                .map_err(scene_err)?;
            Ok(json!("ok"))
        })
        .await
    }

    #[tool(
        description = "Inspect a component on an entity: its fields with types, mutability, and current values"
    )]
    async fn get_component(
        &self,
        Parameters(args): Parameters<ComponentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let entity = parse_uuid(&args.entity_id)?;
        self.run(move |scene| {
            let component = &args.component_id;
            let fields = scene
                .get_component_fields(entity, component)
                .map_err(scene_err)?;
            let fields: Vec<Value> = fields
                .into_iter()
                .map(|field| {
                    let field_type = scene
                        .get_component_field_type(entity, component, &field)
                        .map(|field_type| format!("{field_type:?}"))
                        .unwrap_or_default();
                    json!({
                        "id": field,
                        "type": field_type,
                        "mutable": scene
                            .is_component_field_mutable(entity, component, &field)
                            .unwrap_or(false),
                        "value": scene
                            .render_field(entity, component, &field)
                            .unwrap_or_else(|error| format!("<{error:?}>")),
                    })
                })
                .collect();
            Ok(json!(fields))
        })
        .await
    }

    #[tool(description = "Write a component field on an entity by parsing a string value")]
    async fn set_component_field(
        &self,
        Parameters(args): Parameters<SetFieldArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let entity = parse_uuid(&args.entity_id)?;
        self.run(move |scene| {
            scene
                .parse_field(entity, &args.component_id, &args.field_id, &args.value)
                .map_err(scene_err)?;
            Ok(json!("ok"))
        })
        .await
    }

    #[tool(description = "List all systems with their priorities and providing plugins")]
    async fn list_systems(&self) -> Result<CallToolResult, ErrorData> {
        self.run(|scene| {
            let systems: Vec<Value> = scene
                .get_systems()
                .into_iter()
                .map(|id| {
                    json!({
                        "id": id,
                        "priority": scene.get_system_priority(&id).unwrap_or(0),
                        "plugin": scene.get_system_plugin_id(&id).unwrap_or(""),
                    })
                })
                .collect();
            Ok(json!(systems))
        })
        .await
    }

    #[tool(description = "Add a system by id with a priority (higher runs earlier, default 0)")]
    async fn add_system(
        &self,
        Parameters(args): Parameters<AddSystemArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run(move |scene| {
            scene
                .add_system(args.system_id, args.priority.unwrap_or(0))
                .map_err(scene_err)?;
            Ok(json!("ok"))
        })
        .await
    }

    #[tool(description = "Remove a system by id")]
    async fn remove_system(
        &self,
        Parameters(args): Parameters<SystemArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.run(move |scene| {
            scene.remove_system(&args.system_id).map_err(scene_err)?;
            Ok(json!("ok"))
        })
        .await
    }

    #[tool(description = "List the paths of all loaded dynamic plugins")]
    async fn list_plugins(&self) -> Result<CallToolResult, ErrorData> {
        self.run(|scene| Ok(json!(scene.get_plugins()))).await
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for WasserXrMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("wasserxr", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "Inspect and edit the running WasserXR scene: entities, components, \
                 component fields, systems, and plugins.",
            )
    }
}
